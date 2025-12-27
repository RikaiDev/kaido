use anyhow::Result;
use std::io::Write;

mod agent;
mod ai;
mod audit;
mod commands;
mod config;
mod error;
mod kubectl;
mod safety;
mod shell;
mod tools;
mod ui;
mod utils;

use shell::run_agent_repl;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to file
    init_file_logger();
    
    log::info!("=== Kaido AI Agent Starting ===");
    
    // Run agent REPL
    let result = run_agent_repl().await;
    
    if let Err(e) = &result {
        log::error!("Agent error: {e:?}");
    }
    
    log::info!("=== Kaido AI Agent Exiting ===");
    
    result
}

/// Initialize env_logger to write to file
fn init_file_logger() {
    use std::fs::OpenOptions;
    
    // Create logs directory if not exists
    let log_dir = std::path::Path::new("logs");
    if !log_dir.exists() {
        let _ = std::fs::create_dir(log_dir);
    }
    
    // Open log file in append mode
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/kaido.log")
        .expect("Failed to open log file");
    
    // Initialize env_logger with custom format
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {} {}:{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .filter_level(log::LevelFilter::Debug)
        .init();
}
