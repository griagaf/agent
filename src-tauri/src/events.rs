use agent::agent::AgentEvent;
use log::warn;
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter};

const CHANNEL: &str = "agent:event";

#[derive(Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Message<'a> {
    Said { text: &'a str },
    ToolStarted { name: &'a str, input: &'a Value },
    ToolFinished { name: &'a str, chars: usize },
    ToolFailed { name: &'a str, error: &'a str },
}

pub fn send(app: &AppHandle, event: AgentEvent) {
    let message = match event {
        AgentEvent::Said(text) => Message::Said { text },
        AgentEvent::ToolStarted { name, input } => Message::ToolStarted { name, input },
        AgentEvent::ToolFinished { name, output } => Message::ToolFinished {
            name,
            chars: output.chars().count(),
        },
        AgentEvent::ToolFailed { name, error } => Message::ToolFailed { name, error },
    };

    if let Err(error) = app.emit(CHANNEL, message) {
        warn!("[send] событие не дошло до окна: {error}");
    }
}
