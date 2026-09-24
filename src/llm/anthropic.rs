use anyhow::{Result, anyhow, bail};
use serde_json::{Value, json};

use super::http;
use super::{AssistantTurn, ToolCall, ToolSpec, Turn};
use crate::config::Config;

const API_VERSION: &str = "2023-06-01";
const MAX_TOKENS: u32 = 16000;

/// Нативный Messages API Anthropic (https://api.anthropic.com/v1/messages).
pub struct AnthropicProvider {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl AnthropicProvider {
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
        let body = json!({
            "model": self.model,
            "max_tokens": MAX_TOKENS,
            "system": system,
            "tools": tools.iter().map(tool_to_json).collect::<Vec<_>>(),
            "messages": history.iter().map(turn_to_json).collect::<Vec<_>>(),
        });

        let request = self
            .http
            .post(format!("{}/v1/messages", self.base_url))
            .header("content-type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .json(&body);

        let response = http::send(request).await?;

        let status = response.status();
        let payload: Value = response.json().await?;
        if !status.is_success() {
            bail!("Anthropic API вернул {status}: {payload}");
        }

        parse_message(&payload)
    }
}

fn tool_to_json(tool: &ToolSpec) -> Value {
    json!({
        "name": tool.name,
        "description": tool.description,
        "input_schema": tool.input_schema,
    })
}

fn turn_to_json(turn: &Turn) -> Value {
    match turn {
        Turn::User(text) => json!({ "role": "user", "content": text }),
        Turn::Assistant(turn) => json!({ "role": "assistant", "content": turn.raw }),
        Turn::ToolResults(outcomes) => {
            let blocks: Vec<Value> = outcomes
                .iter()
                .map(|outcome| {
                    json!({
                        "type": "tool_result",
                        "tool_use_id": outcome.call_id,
                        "content": outcome.content,
                        "is_error": outcome.is_error,
                    })
                })
                .collect();
            json!({ "role": "user", "content": blocks })
        }
    }
}

fn parse_message(payload: &Value) -> Result<AssistantTurn> {
    if payload["stop_reason"] == "refusal" {
        let explanation = payload["stop_details"]["explanation"]
            .as_str()
            .unwrap_or("модель отклонила запрос");
        bail!("{explanation}");
    }

    let content = payload["content"]
        .as_array()
        .ok_or_else(|| anyhow!("в ответе нет поля content: {payload}"))?;

    let mut text = String::new();
    let mut tool_calls = Vec::new();

    for block in content {
        match block["type"].as_str() {
            Some("text") => {
                if let Some(chunk) = block["text"].as_str() {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(chunk);
                }
            }
            Some("tool_use") => tool_calls.push(ToolCall {
                id: block["id"].as_str().unwrap_or_default().to_string(),
                name: block["name"].as_str().unwrap_or_default().to_string(),
                input: block["input"].clone(),
            }),
            _ => {}
        }
    }

    Ok(AssistantTurn {
        text,
        tool_calls,
        raw: payload["content"].clone(),
    })
}
