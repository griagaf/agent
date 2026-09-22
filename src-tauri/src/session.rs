use agent::agent::{Agent, AgentEvent};
use log::error;
use tauri::{AppHandle, State};

use crate::error::Error;
use crate::events;

#[tauri::command]
pub async fn ask(
    app: AppHandle,
    agent: State<'_, Agent>,
    question: String,
) -> Result<String, Error> {
    let question = question.trim();
    if question.is_empty() {
        return Err(Error::EmptyQuestion);
    }

    let mut on_event = move |event: AgentEvent<'_>| events::send(&app, event);

    agent.ask(question, &mut on_event).await.map_err(|e| {
        error!("[ask] {e:#}");
        Error::AskFailed
    })
}
