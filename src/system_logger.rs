use std::fs::{File, OpenOptions};
use std::io::{self, Write};
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
    Info,
    Warning,
    Error,
    Success,
    Security,
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

    pub fn log_command(&mut self, command: &str, success: bool, details: Option<String>, error: Option<String>) {
        if !self.enabled {
            return;
        }

        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        let level = if success { LogLevel::Success } else { LogLevel::Error };

        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            command: command.to_string(),
            user,
            success,
            details,
            error,
        };

        self.write_log_entry(&entry);
    }

    pub fn log_security_event(&mut self, event: &str, details: Option<String>) {
        if !self.enabled {
            return;
        }

        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Security,
            command: event.to_string(),
            user,
            success: true,
            details,
            error: None,
        };

        self.write_log_entry(&entry);
    }

    pub fn log_info(&mut self, message: &str, details: Option<String>) {
        if !self.enabled {
            return;
        }

        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

        let entry = LogEntry {
            timestamp: Utc::now(),
            level: LogLevel::Info,
            command: message.to_string(),
            user,
            success: true,
            details,
            error: None,
        };

        self.write_log_entry(&entry);
    }

    fn write_log_entry(&mut self, entry: &LogEntry) {
        if let Some(ref mut file) = self.log_file {
            if let Ok(json) = serde_json::to_string(entry) {
                let _ = writeln!(file, "{}", json);
                let _ = file.flush();
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_log_path(&self) -> Option<String> {
        if Path::new("/var/log/linux-distro-agent/agent.log").exists() {
            Some("/var/log/linux-distro-agent/agent.log".to_string())
        } else {
            let home = std::env::var("HOME").ok()?;
            Some(format!("{}/.local/share/linux-distro-agent/agent.log", home))
        }
    }
}

impl Default for SystemLogger {
    fn default() -> Self {
        Self::new()
    }
}
