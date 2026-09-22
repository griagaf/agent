use std::io::{self, Write};

use agent::agent::{Agent, AgentEvent};
use agent::config::Config;
use agent::tools::ToolBox;
use anyhow::{Result, bail};
use serde_json::{Value, json};

#[tokio::main]
async fn main() -> Result<()> {
    announce_dpi_awareness();

    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest = args.get(1..).unwrap_or_default().join(" ");

    // Отладочные режимы: дёргаем инструмент напрямую, без модели и без ключа.
    match args.first().map(String::as_str) {
        Some("--find") => return call_tool("find_file", json!({ "query": rest })),
        Some("--element") => {
            let (name, window) = match rest.split_once(" --in ") {
                Some((name, window)) => (name, Some(window)),
                None => (rest.as_str(), None),
            };
            return call_tool("find_element", json!({ "name": name, "window": window }));
        }
        _ => {}
    }

    let question = args.join(" ");
    let question = if question.trim().is_empty() {
        ask_user()?
    } else {
        question
    };

    let config = Config::from_env()?;
    println!("Модель: {} ({})\n", config.model, config.base_url);

    let agent = Agent::new(&config)?;
    let answer = agent.ask(&question, &mut print_event).await?;

    println!("\n{answer}");
    Ok(())
}

// Пока процесс не объявлен DPI-осведомлённым, UI Automation делит координаты на масштаб экрана.
#[cfg(windows)]
fn announce_dpi_awareness() {
    use windows::Win32::UI::HiDpi::{
        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
    };

    // Безопасно: вызов без указателей, аргумент — константа самой системы.
    if let Err(reason) =
        unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) }
    {
        println!("Внимание: координаты элементов будут неточными ({reason})");
    }
}

#[cfg(not(windows))]
fn announce_dpi_awareness() {}

fn call_tool(name: &str, input: Value) -> Result<()> {
    let tools = ToolBox::with_defaults();
    let Some(tool) = tools.find(name) else {
        bail!("инструмент {name} не зарегистрирован");
    };

    println!("{}", tool.call(&input)?);
    Ok(())
}

fn ask_user() -> Result<String> {
    print!("Что найти? ");
    io::stdout().flush()?;

    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

fn print_event(event: AgentEvent) {
    match event {
        AgentEvent::Said(text) => println!("{text}"),
        AgentEvent::ToolStarted { name, input } => println!("[инструмент] {name} {input}"),
        AgentEvent::ToolFinished { name, output } => println!(
            "[готово] {name}: {} символов результата",
            output.chars().count()
        ),
        AgentEvent::ToolFailed { name, error } => println!("[ошибка] {name}: {error}"),
    }
}
