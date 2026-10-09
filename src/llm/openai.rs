use anyhow::{Result, anyhow, bail};
use serde_json::{Value, json};

use super::http;
use super::{AssistantTurn, ToolCall, ToolSpec, Turn};
use crate::config::Config;

/// Формат /chat/completions. Его понимают Ollama, LM Studio, llama.cpp,
/// Groq, OpenRouter и сам OpenAI — меняется только base_url и модель.
pub struct OpenAiProvider {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(http: reqwest::Client, config: &Config) -> Self {
        Self {
            http,
            base_url: config.base_url.trim_end_matches('/').to_string(),
            api_key: config.api_key.clone(),
            model: config.model.clone(),
        }
    }

    pub async fn respond(
        &self,
        system: &str,
        history: &[Turn],
        tools: &[ToolSpec],
    ) -> Result<AssistantTurn> {
        let mut messages = vec![json!({ "role": "system", "content": system })];
        for turn in history {
            append_turn(&mut messages, turn);
        }

        let body = json!({
            "model": self.model,
            "messages": messages,
            "tools": tools.iter().map(tool_to_json).collect::<Vec<_>>(),
            "tool_choice": "auto",
        });

        let request = self
            .http
            .post(format!("{}/chat/completions", self.base_url))
            .header("content-type", "application/json")
            .bearer_auth(&self.api_key)
            .json(&body);

        let response = http::send(request).await?;

        let status = response.status();
        let payload: Value = response.json().await?;
        if !status.is_success() {
            // Список моделей у сервисов меняется, поэтому сразу говорим, где смотреть актуальный.
            if payload["error"]["code"] == "model_not_found" {
                bail!(
                    "модели '{}' нет у этого сервиса. Доступные смотрите здесь: {}/models — \
                     нужную пропишите в AGENT_MODEL",
                    self.model,
                    self.base_url
                );
            }
            bail!("LLM API вернул {status}: {payload}");
        }

        parse_message(&payload)
    }
}

fn tool_to_json(tool: &ToolSpec) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": tool.name,
            "description": tool.description,
            "parameters": tool.input_schema,
        },
    })
}

fn append_turn(messages: &mut Vec<Value>, turn: &Turn) {
    match turn {
        Turn::User(text) => messages.push(json!({ "role": "user", "content": text })),
        Turn::Assistant(turn) => messages.push(turn.raw.clone()),
        // Здесь, в отличие от Anthropic, каждый результат — отдельное сообщение.
        Turn::ToolResults(outcomes) => messages.extend(outcomes.iter().map(|outcome| {
            json!({
                "role": "tool",
                "tool_call_id": outcome.call_id,
                "content": outcome.content,
            })
        })),
    }
}

fn parse_message(payload: &Value) -> Result<AssistantTurn> {
    let message = payload["choices"]
        .get(0)
        .map(|choice| &choice["message"])
        .ok_or_else(|| anyhow!("в ответе нет choices: {payload}"))?;

    let text = message["content"].as_str().unwrap_or_default().to_string();

    let mut tool_calls = Vec::new();
    if let Some(calls) = message["tool_calls"].as_array() {
        for call in calls {
            let arguments = call["function"]["arguments"].as_str().unwrap_or("{}");
            // Аргументы приходят строкой с JSON внутри, а мелкие модели иногда шлют пустую.
            let input = if arguments.trim().is_empty() {
                json!({})
            } else {
                serde_json::from_str(arguments).map_err(|e| {
                    anyhow!("модель прислала некорректные аргументы ({e}): {arguments}")
                })?
            };

            tool_calls.push(ToolCall {
                id: call["id"].as_str().unwrap_or_default().to_string(),
                name: call["function"]["name"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                input,
            });
        }
    }

    Ok(AssistantTurn {
        text,
        tool_calls,
        raw: message.clone(),
    })
}
