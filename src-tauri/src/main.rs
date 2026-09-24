#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod events;
mod overlay;
mod session;
mod steps;

use agent::agent::Agent;
use agent::config::Config;
use anyhow::Result;
use env_logger::Env;
use log::{error, info};

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Окно поднимаем даже со сломанной настройкой, иначе человек не увидит причину.
    let agent = build_agent()
        .inspect_err(|reason| error!("[main] агент не собрался: {reason:#}"))
        .ok();

    let started = tauri::Builder::default()
        .manage(agent)
        .setup(|app| {
            overlay::create(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            session::ask,
            overlay::hide_pointer,
            overlay::raise_pointer,
            steps::show_step,
            steps::check_step,
            steps::perform_step
        ])
        .run(tauri::generate_context!());

    if let Err(reason) = started {
        error!("[main] окно не запустилось: {reason}");
        std::process::exit(1);
    }
}

fn build_agent() -> Result<Agent> {
    let config = Config::from_env()?;
    info!(
        "[build_agent] модель {} ({})",
        config.model, config.base_url
    );

    Agent::new(&config)
}
