use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub agent: AgentConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub distro_builder: DistroBuilderConfig,
    pub remote: RemoteConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub run_as_root: bool,
    pub allowed_users: Vec<String>,
    pub allowed_groups: Vec<String>,
    pub enable_history: bool,
    pub cache_duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_audit_log: bool,
    pub require_sudo_for_installs: bool,
    pub allowed_package_managers: Vec<String>,
    pub max_concurrent_operations: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub log_level: String,
    pub log_file: String,
    pub max_log_size_mb: u64,
    pub rotate_logs: bool,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroBuilderConfig {
    pub max_build_time_hours: u32,
    pub max_iso_size_gb: u32,
    pub allowed_base_systems: Vec<String>,
    pub default_work_dir: String,
    pub cleanup_after_build: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub enable_ssh_support: bool,
    pub ssh_key_path: Option<String>,
    pub allowed_hosts: Vec<String>,
    pub default_user: String,
    pub connection_timeout: u64,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            agent: AgentConfig {
                run_as_root: false,
                allowed_users: vec!["root".to_string()],
                allowed_groups: vec!["wheel".to_string(), "sudo".to_string()],
                enable_history: true,
                cache_duration: 300,
            },
            security: SecurityConfig {
                enable_audit_log: true,
                require_sudo_for_installs: true,
                allowed_package_managers: vec![
                    "pacman".to_string(),
                    "apt".to_string(),
                    "dnf".to_string(),
                    "zypper".to_string(),
                    "portage".to_string(),
                    "nix".to_string(),
                    "apk".to_string(),
                ],
                max_concurrent_operations: 5,
                timeout_seconds: 300,
            },
            logging: LoggingConfig {
                log_level: "info".to_string(),
                log_file: "/var/log/linux-distro-agent/agent.log".to_string(),
                max_log_size_mb: 100,
                rotate_logs: true,
                retention_days: 30,
            },
            distro_builder: DistroBuilderConfig {
                max_build_time_hours: 4,
                max_iso_size_gb: 8,
                allowed_base_systems: vec![
                    "Arch".to_string(),
                    "Debian".to_string(),
                    "Ubuntu".to_string(),
                ],
                default_work_dir: "/var/lib/linux-distro-agent/builds".to_string(),
                cleanup_after_build: true,
            },
            remote: RemoteConfig {
                enable_ssh_support: false,
                ssh_key_path: None,
                allowed_hosts: vec![],
                default_user: "root".to_string(),
                connection_timeout: 30,
            },
        }
    }
}

impl SystemConfig {
    pub fn load() -> Result<Self> {
        let config_paths = [
            "/etc/linux-distro-agent/config.toml",
            "/etc/linux-distro-agent.toml",
        ];

        for path in &config_paths {
            if Path::new(path).exists() {
                let content = fs::read_to_string(path)?;
                return Ok(toml::from_str(&content)?);
            }
        }

        // Return default config if no system config found
        Ok(Self::default())
    }



    pub fn generate_sample_config() -> String {
        let config = Self::default();
        toml::to_string_pretty(&config).unwrap_or_else(|_| "# Error generating config".to_string())
    }
}
