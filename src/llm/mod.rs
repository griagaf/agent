pub mod anthropic;
pub mod http;
pub mod openai;

use anyhow::Result;
use serde_json::Value;

use crate::config::{Config, ProviderKind};

/// Описание инструмента в том виде, в каком его получает модель.
#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Запрос модели на вызов инструмента.
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: Value,
}

/// Результат выполнения инструмента, который уходит обратно модели.
#[derive(Debug, Clone)]
pub struct ToolOutcome {
    pub call_id: String,
    pub content: String,
    pub is_error: bool,
}

#[derive(Debug, Clone)]
pub struct AssistantTurn {
    pub text: String,
    pub tool_calls: Vec<ToolCall>,
    /// Ответ провайдера как есть. Anthropic требует возвращать блоки дословно,
    /// иначе рвётся связка thinking -> tool_use, поэтому историю храним в сыром виде.
    pub raw: Value,
}

#[derive(Debug, Clone)]
pub enum Turn {
    User(String),
    Assistant(AssistantTurn),
    ToolResults(Vec<ToolOutcome>),
}

/// Провайдер модели. Диспетчеризация через enum, а не dyn: реализаций две,
/// и обе должны уметь async без лишних крейтов.
pub enum Provider {
    Anthropic(anthropic::AnthropicProvider),
    OpenAiCompat(openai::OpenAiProvider),
}

impl Provider {
    pub fn from_config(config: &Config) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        Ok(match config.provider {
            ProviderKind::Anthropic => {
                Provider::Anthropic(anthropic::AnthropicProvider::new(http, config))
            }
            ProviderKind::OpenAiCompat => {
                Provider::OpenAiCompat(openai::OpenAiProvider::new(http, config))
            }
        })
    }

    pub async fn respond(
        &self,
        system: &str,
        history: &[Turn],
        tools: &[ToolSpec],
    ) -> Result<AssistantTurn> {
        match self {
            Provider::Anthropic(p) => p.respond(system, history, tools).await,
            Provider::OpenAiCompat(p) => p.respond(system, history, tools).await,
        }
    }
}
