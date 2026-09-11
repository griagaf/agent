use std::io::{self, Write};

use agent::agent::{Agent, AgentEvent};
use agent::config::Config;
use agent::tools::ToolBox;
use anyhow::{Result, bail};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Отладочный режим: дёргаем инструмент напрямую, без модели и без ключа.
    if args.first().is_some_and(|arg| arg == "--find") {
        return find_directly(&args[1..].join(" "));
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

fn find_directly(query: &str) -> Result<()> {
    if query.trim().is_empty() {
        bail!("укажите, что искать: cargo run -- --find <часть имени файла>");
    }

    let tools = ToolBox::with_defaults();
    let Some(tool) = tools.find("find_file") else {
        bail!("инструмент find_file не зарегистрирован");
    };

    println!("{}", tool.call(&json!({ "query": query }))?);
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
