use std::sync::Arc;

use anyhow::{Result, bail};
use serde_json::Value;

use crate::config::Config;
use crate::llm::{Provider, ToolOutcome, Turn};
use crate::tools::ToolBox;

const MAX_STEPS: usize = 14;

const SYSTEM_PROMPT: &str = "Ты — помощник на компьютере пользователя. \
Пользователь плохо разбирается в компьютерах, поэтому отвечай простыми словами \
по-русски, без технического жаргона.

Твоя задача — не рассказать, а провести: человек должен сделать всё сам, своими \
руками. Поэтому, когда просят научить чему-то на экране, не перечисляй шаги текстом, \
а строй план инструментами.

1. Найди нужные элементы через find_element. Сам ты экран не видишь, подписи \
выдумывать нельзя. Всегда указывай window: поиск по всему экрану идёт несколько \
секунд и цепляет чужие подписи.
2. Собери план через propose_plan. Один шаг — одно действие. Подсказку пиши так, \
как объяснил бы человеку рядом: куда смотреть и что нажать.
3. После плана скажи одной фразой, что будете делать. Сами шаги человек увидит \
в окне, повторять их в ответе не нужно.

Признак выполнения шага — то, чего до шага на экране не было: открылось меню, \
появилось окно, в поле появился нужный текст. Кнопка, которая видна и так, \
признаком быть не может.

Если нужной программы на экране нет, не выдумывай план — попроси её открыть.

Когда нужно найти файл, вызывай find_file: придумывать пути нельзя. Если подошло \
несколько файлов, покажи списком и объясни, чем они отличаются. В ответе про файл \
всегда пиши полный путь и размер.";

/// Что происходит внутри шага агента — чтобы CLI печатал ход работы,
/// а окно Tauri потом слало те же события во фронтенд.
pub enum AgentEvent<'a> {
    Said(&'a str),
    ToolStarted { name: &'a str, input: &'a Value },
    ToolFinished { name: &'a str, output: &'a str },
    ToolFailed { name: &'a str, error: &'a str },
}

pub struct Agent {
    provider: Provider,
    tools: Arc<ToolBox>,
}

impl Agent {
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            provider: Provider::from_config(config)?,
            tools: Arc::new(ToolBox::with_defaults()),
        })
    }

    /// Гоняет цикл «модель -> инструмент -> модель», пока модель не ответит текстом.
    pub async fn ask(
        &self,
        question: &str,
        on_event: &mut (dyn FnMut(AgentEvent) + Send),
    ) -> Result<String> {
        let specs = self.tools.specs();
        let mut history = vec![Turn::User(question.to_string())];

        for _ in 0..MAX_STEPS {
            let turn = self
                .provider
                .respond(SYSTEM_PROMPT, &history, &specs)
                .await?;

            // Финальный ответ возвращаем наверх, а не шлём событием, иначе он печатается дважды.
            if turn.tool_calls.is_empty() {
                return Ok(turn.text);
            }
            if !turn.text.is_empty() {
                on_event(AgentEvent::Said(&turn.text));
            }

            let calls = turn.tool_calls.clone();
            history.push(Turn::Assistant(turn));

            let mut outcomes = Vec::with_capacity(calls.len());
            for call in &calls {
                on_event(AgentEvent::ToolStarted {
                    name: &call.name,
                    input: &call.input,
                });

                // Ошибку инструмента отдаём модели, а не наверх: пусть исправится сама.
                let outcome = match self.run_tool(&call.name, &call.input).await {
                    Ok(output) => {
                        on_event(AgentEvent::ToolFinished {
                            name: &call.name,
                            output: &output,
                        });
                        ToolOutcome {
                            call_id: call.id.clone(),
                            content: output,
                            is_error: false,
                        }
                    }
                    Err(error) => {
                        let message = error.to_string();
                        on_event(AgentEvent::ToolFailed {
                            name: &call.name,
                            error: &message,
                        });
                        ToolOutcome {
                            call_id: call.id.clone(),
                            content: message,
                            is_error: true,
                        }
                    }
                };

                outcomes.push(outcome);
            }

            history.push(Turn::ToolResults(outcomes));
        }

        bail!("агент не уложился в {MAX_STEPS} шагов и остановился");
    }

    async fn run_tool(&self, name: &str, input: &Value) -> Result<String> {
        let Some(tool) = self.tools.find(name) else {
            bail!("инструмента '{name}' не существует");
        };

        // Поиск по диску блокирует поток надолго — уводим его с рантайма.
        let input = input.clone();
        tokio::task::spawn_blocking(move || tool.call(&input)).await?
    }
}
