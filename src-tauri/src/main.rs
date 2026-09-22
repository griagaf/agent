#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod events;
mod session;

use agent::agent::Agent;
use agent::config::Config;
use anyhow::Result;
use env_logger::Env;
use log::{error, info};

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let agent = match build_agent() {
        Ok(agent) => agent,
        Err(reason) => {
            error!("[main] агент не собрался: {reason:#}");
            std::process::exit(1);
        }
    };

    let started = tauri::Builder::default()
        .manage(agent)
        .invoke_handler(tauri::generate_handler![session::ask])
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
