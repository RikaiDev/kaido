use chrono::Utc;
use log::{Level, LevelFilter, Metadata, Record};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Custom logger implementation for Kaido AI Shell
pub struct KaidoLogger {
    log_file: Option<PathBuf>,
    max_file_size: u64,
    max_files: usize,
}

impl KaidoLogger {
    pub fn new(log_file: Option<PathBuf>, max_file_size: u64, max_files: usize) -> Self {
        Self {
            log_file,
            max_file_size,
            max_files,
        }
    }

    pub fn init(
        log_file: Option<PathBuf>,
        max_file_size: u64,
        max_files: usize,
    ) -> Result<(), log::SetLoggerError> {
        let logger = Self::new(log_file, max_file_size, max_files);
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(LevelFilter::Info);
        Ok(())
    }
}

impl log::Log for KaidoLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
            let log_entry = format!(
                "{} [{}] {}: {}\n",
                timestamp,
                record.level(),
                record.target(),
                record.args()
            );

            // Print to stderr
            eprintln!("{}", log_entry.trim());

            // Write to file if configured
            if let Some(ref log_file) = self.log_file {
                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_file) {
                    let _ = file.write_all(log_entry.as_bytes());
                }
            }
        }
    }

    fn flush(&self) {}
}

/// Command execution logger for audit trail
pub struct CommandLogger {
    log_file: Mutex<Option<PathBuf>>,
}

impl CommandLogger {
    pub fn new(log_file: Option<PathBuf>) -> Self {
        Self {
            log_file: Mutex::new(log_file),
        }
    }

    pub fn log_command_execution(
        &self,
        command: &str,
        working_directory: &str,
        status: &str,
        exit_code: Option<i32>,
        execution_time_ms: u64,
        stdout: &str,
        stderr: &str,
    ) {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let log_entry = format!(
            "COMMAND_EXECUTION: {} | CMD: {} | DIR: {} | STATUS: {} | EXIT: {:?} | TIME: {}ms | STDOUT: {} | STDERR: {}\n",
            timestamp,
            command,
            working_directory,
            status,
            exit_code,
            execution_time_ms,
            stdout.chars().take(200).collect::<String>(),
            stderr.chars().take(200).collect::<String>()
        );

        if let Ok(log_file) = self.log_file.lock() {
            if let Some(ref path) = *log_file {
                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                    let _ = file.write_all(log_entry.as_bytes());
                }
            }
        }
    }

    pub fn log_ai_interaction(
        &self,
        user_input: &str,
        ai_response: &str,
        command_executed: Option<&str>,
    ) {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let log_entry = format!(
            "AI_INTERACTION: {} | INPUT: {} | RESPONSE: {} | CMD: {:?}\n",
            timestamp,
            user_input.chars().take(100).collect::<String>(),
            ai_response.chars().take(100).collect::<String>(),
            command_executed
        );

        if let Ok(log_file) = self.log_file.lock() {
            if let Some(ref path) = *log_file {
                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                    let _ = file.write_all(log_entry.as_bytes());
                }
            }
        }
    }

    pub fn log_safety_event(&self, command: &str, rule_id: &str, severity: &str, action: &str) {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let log_entry = format!(
            "SAFETY_EVENT: {timestamp} | CMD: {command} | RULE: {rule_id} | SEVERITY: {severity} | ACTION: {action}\n"
        );

        if let Ok(log_file) = self.log_file.lock() {
            if let Some(ref path) = *log_file {
                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                    let _ = file.write_all(log_entry.as_bytes());
                }
            }
        }
    }
}

/// Initialize logging system
pub fn init_logging(
    log_file: Option<PathBuf>,
    max_file_size: u64,
    max_files: usize,
) -> Result<(), log::SetLoggerError> {
    KaidoLogger::init(log_file, max_file_size, max_files)
}

/// Create command logger instance
pub fn create_command_logger(log_file: Option<PathBuf>) -> CommandLogger {
    CommandLogger::new(log_file)
}

/// Simple command logging function for compatibility
pub fn log_command(command: &str, status: &str, output: &str, error_message: Option<&str>) {
    log::info!("COMMAND_LOG: {{ \"command\": \"{command}\", \"status\": \"{status}\", \"output\": \"{output}\", \"error\": \"{error_message:?}\" }}");
}
