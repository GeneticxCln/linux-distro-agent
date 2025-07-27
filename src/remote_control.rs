use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::process::Command as AsyncCommand;
use crate::system_config::RemoteConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteHost {
    pub hostname: String,
    pub user: String,
    pub port: Option<u16>,
    pub key_path: Option<String>,
    pub sudo_password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteTask {
    pub id: String,
    pub command: String,
    pub hosts: Vec<String>,
    pub parallel: bool,
    pub timeout: Option<Duration>,
    pub become_root: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteResult {
    pub host: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

pub struct RemoteController {
    config: RemoteConfig,
    hosts: HashMap<String, RemoteHost>,
}

impl RemoteController {
    pub fn new(config: RemoteConfig) -> Self {
        Self {
            config,
            hosts: HashMap::new(),
        }
    }

    pub fn add_host(&mut self, name: String, host: RemoteHost) {
        self.hosts.insert(name, host);
    }

    pub fn load_hosts_from_file(&mut self, path: &str) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read hosts file")?;
        
        let hosts: HashMap<String, RemoteHost> = toml::from_str(&content)
            .context("Failed to parse hosts file")?;
        
        self.hosts.extend(hosts);
        Ok(())
    }

    pub async fn execute_task(&self, task: &RemoteTask) -> Result<Vec<RemoteResult>> {
        if !self.config.enable_ssh_support {
            anyhow::bail!("SSH support is disabled in configuration");
        }

        let mut results = Vec::new();

        if task.parallel {
            results = self.execute_parallel(task).await?;
        } else {
            results = self.execute_sequential(task).await?;
        }

        Ok(results)
    }

    async fn execute_parallel(&self, task: &RemoteTask) -> Result<Vec<RemoteResult>> {
        let mut handles = Vec::new();

        for host_name in &task.hosts {
            if let Some(host) = self.hosts.get(host_name) {
                let host = host.clone();
                let task = task.clone();
                let config = self.config.clone();

                let handle = tokio::spawn(async move {
                    Self::execute_on_host(&config, &host, &task).await
                });

                handles.push(handle);
            }
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result?),
                Err(e) => eprintln!("Task execution error: {}", e),
            }
        }

        Ok(results)
    }

    async fn execute_sequential(&self, task: &RemoteTask) -> Result<Vec<RemoteResult>> {
        let mut results = Vec::new();

        for host_name in &task.hosts {
            if let Some(host) = self.hosts.get(host_name) {
                let result = Self::execute_on_host(&self.config, host, task).await?;
                results.push(result);
            }
        }

        Ok(results)
    }

    async fn execute_on_host(
        config: &RemoteConfig,
        host: &RemoteHost,
        task: &RemoteTask,
    ) -> Result<RemoteResult> {
        let start_time = std::time::Instant::now();
        
        // Build SSH command
        let mut ssh_cmd = AsyncCommand::new("ssh");
        
        // Add SSH options
        ssh_cmd.arg("-o").arg("StrictHostKeyChecking=no");
        ssh_cmd.arg("-o").arg(format!("ConnectTimeout={}", config.connection_timeout));
        
        if let Some(key_path) = &host.key_path.as_ref().or(config.ssh_key_path.as_ref()) {
            ssh_cmd.arg("-i").arg(key_path);
        }

        if let Some(port) = host.port {
            ssh_cmd.arg("-p").arg(port.to_string());
        }

        // Add user and hostname
        let user_host = format!("{}@{}", host.user, host.hostname);
        ssh_cmd.arg(&user_host);

        // Prepare command
        let mut command = task.command.clone();
        if task.become_root && host.user != "root" {
            command = format!("sudo {}", command);
        }

        ssh_cmd.arg(&command);

        // Set timeout if specified
        if let Some(timeout) = task.timeout {
            ssh_cmd.kill_on_drop(true);
        }

        // Execute command
        let output = ssh_cmd.output().await
            .context("Failed to execute SSH command")?;

        let duration = start_time.elapsed();

        Ok(RemoteResult {
            host: host.hostname.clone(),
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration,
        })
    }

    pub fn generate_inventory_template() -> String {
        r#"# Linux Distribution Agent - Remote Hosts Inventory
# Save this as ~/.config/linux-distro-agent/hosts.toml

[web-server]
hostname = "192.168.1.100"
user = "admin"
port = 22
key_path = "~/.ssh/id_rsa"

[database-server]
hostname = "db.example.com"
user = "root"
port = 2222
key_path = "/etc/ssh/admin_key"

[backup-server]
hostname = "backup.local"
user = "backup"
"#.to_string()
    }

    // Predefined common tasks
    pub fn create_system_update_task(hosts: Vec<String>) -> RemoteTask {
        RemoteTask {
            id: "system-update".to_string(),
            command: "linux-distro-agent update --execute".to_string(),
            hosts,
            parallel: true,
            timeout: Some(Duration::from_secs(600)),
            become_root: true,
        }
    }

    pub fn create_package_install_task(package: &str, hosts: Vec<String>) -> RemoteTask {
        RemoteTask {
            id: format!("install-{}", package),
            command: format!("linux-distro-agent install {} --execute", package),
            hosts,
            parallel: true,
            timeout: Some(Duration::from_secs(300)),
            become_root: true,
        }
    }

    pub fn create_info_gathering_task(hosts: Vec<String>) -> RemoteTask {
        RemoteTask {
            id: "gather-info".to_string(),
            command: "linux-distro-agent info --pretty".to_string(),
            hosts,
            parallel: true,
            timeout: Some(Duration::from_secs(30)),
            become_root: false,
        }
    }

    pub async fn test_connectivity(&self, host_name: &str) -> Result<bool> {
        if let Some(host) = self.hosts.get(host_name) {
            let test_task = RemoteTask {
                id: "connectivity-test".to_string(),
                command: "echo 'Connection successful'".to_string(),
                hosts: vec![host_name.to_string()],
                parallel: false,
                timeout: Some(Duration::from_secs(10)),
                become_root: false,
            };

            let results = self.execute_task(&test_task).await?;
            Ok(results.first().map_or(false, |r| r.success))
        } else {
            anyhow::bail!("Host '{}' not found", host_name);
        }
    }
}
