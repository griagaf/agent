use agent::agent::AgentEvent;
use agent::scenario::Plan;
use agent::tools::propose_plan::{self, Accepted};
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
    Planned { plan: Plan },
}

pub fn send(app: &AppHandle, event: AgentEvent) {
    let message = match event {
        AgentEvent::Said(text) => Message::Said { text },
        AgentEvent::ToolStarted { name, input } => Message::ToolStarted { name, input },
        AgentEvent::ToolFinished { name, output } => finished(name, output),
        AgentEvent::ToolFailed { name, error } => Message::ToolFailed { name, error },
    };

    if let Err(error) = app.emit(CHANNEL, message) {
        warn!("[send] событие не дошло до окна: {error}");
    }
}

// План — единственный результат инструмента, который окну нужен целиком.
fn finished<'a>(name: &'a str, output: &'a str) -> Message<'a> {
    if name != propose_plan::NAME {
        return Message::ToolFinished {
            name,
            chars: output.chars().count(),
        };
    }

    match serde_json::from_str::<Accepted>(output) {
        Ok(accepted) => Message::Planned {
            plan: accepted.plan,
        },
        Err(reason) => {
            warn!("[finished] план не разобрался: {reason}");
            Message::ToolFailed {
                name,
                error: "план не удалось прочитать",
            }
        }
    }
}
