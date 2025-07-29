use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;
use chrono::{DateTime, Utc};

pub struct SystemLogger {
    log_file: Option<File>,
    enabled: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    command: String,
    user: String,
    success: bool,
    details: Option<String>,
    error: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub enum LogLevel {
    #[allow(dead_code)]
    Debug,
}

impl SystemLogger {
    pub fn new() -> Self {
        let mut logger = SystemLogger {
            log_file: None,
            enabled: false,
        };
        
        logger.initialize();
        logger
    }

    fn initialize(&mut self) {
        // Try to open system log file
        if let Ok(file) = self.open_system_log() {
            self.log_file = Some(file);
            self.enabled = true;
        } else {
            // Fallback to user log directory
            if let Ok(file) = self.open_user_log() {
                self.log_file = Some(file);
                self.enabled = true;
            }
        }
    }

    fn open_system_log(&self) -> io::Result<File> {
        // Try to create system log directory if it doesn't exist
        if !Path::new("/var/log/linux-distro-agent").exists() {
            std::fs::create_dir_all("/var/log/linux-distro-agent")?;
        }

        OpenOptions::new()
            .create(true)
            .append(true)
            .open("/var/log/linux-distro-agent/agent.log")
    }

    fn open_user_log(&self) -> io::Result<File> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let log_dir = format!("{}/.local/share/linux-distro-agent", home);
        
        std::fs::create_dir_all(&log_dir)?;
        
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("{}/agent.log", log_dir))
    }

}

impl Default for SystemLogger {
    fn default() -> Self {
        Self::new()
    }
}
