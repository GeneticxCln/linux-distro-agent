use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub status: ServiceStatus,
    pub enabled: bool,
    pub active: bool,
    pub running: bool,
    pub failed: bool,
    pub description: String,
    pub main_pid: Option<u32>,
    pub memory_usage: Option<String>,
    pub cpu_usage: Option<String>,
    pub uptime: Option<String>,
    pub restart_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceStatus {
    Active,
    Inactive,
    Failed,
    Activating,
    Deactivating,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCommand {
    pub command: String,
    pub description: String,
    pub requires_root: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub status: String,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub memory_usage: String,
    pub user: String,
    pub command: String,
}

pub struct ServiceManager {
    services_cache: HashMap<String, ServiceInfo>,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            services_cache: HashMap::new(),
        }
    }

    pub fn list_services(&mut self, filter: Option<&str>) -> Result<Vec<ServiceInfo>> {
        let mut services = Vec::new();
        
        // Get list of all systemd services
        let output = Command::new("systemctl")
            .args(&["list-units", "--type=service", "--all", "--no-pager", "--plain"])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to list systemd services"));
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines().skip(1) { // Skip header
            if line.trim().is_empty() || line.contains("LOAD") {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let service_name = parts[0];
                
                // Apply filter if provided
                if let Some(filter) = filter {
                    if !service_name.contains(filter) && !line.contains(filter) {
                        continue;
                    }
                }
                
                if let Ok(service_info) = self.get_service_info(service_name) {
                    services.push(service_info);
                }
            }
        }
        
        // Update cache
        for service in &services {
            self.services_cache.insert(service.name.clone(), service.clone());
        }
        
        Ok(services)
    }

    pub fn get_service_info(&self, service_name: &str) -> Result<ServiceInfo> {
        // Get service status
        let status_output = Command::new("systemctl")
            .args(&["status", service_name, "--no-pager", "--lines=0"])
            .output()?;
        
        // Get service properties
        let props_output = Command::new("systemctl")
            .args(&["show", service_name, "--no-pager"])
            .output()?;
        
        let status_str = String::from_utf8_lossy(&status_output.stdout);
        let props_str = String::from_utf8_lossy(&props_output.stdout);
        
        let mut service_info = ServiceInfo {
            name: service_name.to_string(),
            status: ServiceStatus::Unknown,
            enabled: false,
            active: false,
            running: false,
            failed: false,
            description: String::new(),
            main_pid: None,
            memory_usage: None,
            cpu_usage: None,
            uptime: None,
            restart_count: None,
        };
        
        // Parse properties
        for line in props_str.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "Description" => service_info.description = value.to_string(),
                    "ActiveState" => {
                        service_info.status = match value {
                            "active" => ServiceStatus::Active,
                            "inactive" => ServiceStatus::Inactive,
                            "failed" => ServiceStatus::Failed,
                            "activating" => ServiceStatus::Activating,
                            "deactivating" => ServiceStatus::Deactivating,
                            _ => ServiceStatus::Unknown,
                        };
                        service_info.active = value == "active";
                    },
                    "SubState" => {
                        service_info.running = value == "running";
                    },
                    "UnitFileState" => {
                        service_info.enabled = value == "enabled";
                    },
                    "MainPID" => {
                        if let Ok(pid) = value.parse::<u32>() {
                            if pid > 0 {
                                service_info.main_pid = Some(pid);
                            }
                        }
                    },
                    "MemoryCurrent" => {
                        if let Ok(bytes) = value.parse::<u64>() {
                            service_info.memory_usage = Some(self.format_bytes(bytes));
                        }
                    },
                    "NRestarts" => {
                        if let Ok(count) = value.parse::<u32>() {
                            service_info.restart_count = Some(count);
                        }
                    },
                    _ => {}
                }
            }
        }
        
        // Check if failed
        service_info.failed = matches!(service_info.status, ServiceStatus::Failed);
        
        // Get uptime if service is running
        if service_info.running {
            if let Some(pid) = service_info.main_pid {
                service_info.uptime = self.get_process_uptime(pid);
            }
        }
        
        Ok(service_info)
    }

    pub fn get_service_command(&self, action: &str, service_name: &str) -> Option<ServiceCommand> {
        match action.to_lowercase().as_str() {
            "start" => Some(ServiceCommand {
                command: format!("systemctl start {}", service_name),
                description: format!("Start {} service", service_name),
                requires_root: true,
            }),
            "stop" => Some(ServiceCommand {
                command: format!("systemctl stop {}", service_name),
                description: format!("Stop {} service", service_name),
                requires_root: true,
            }),
            "restart" => Some(ServiceCommand {
                command: format!("systemctl restart {}", service_name),
                description: format!("Restart {} service", service_name),
                requires_root: true,
            }),
            "reload" => Some(ServiceCommand {
                command: format!("systemctl reload {}", service_name),
                description: format!("Reload {} service configuration", service_name),
                requires_root: true,
            }),
            "enable" => Some(ServiceCommand {
                command: format!("systemctl enable {}", service_name),
                description: format!("Enable {} service to start at boot", service_name),
                requires_root: true,
            }),
            "disable" => Some(ServiceCommand {
                command: format!("systemctl disable {}", service_name),
                description: format!("Disable {} service from starting at boot", service_name),
                requires_root: true,
            }),
            "status" => Some(ServiceCommand {
                command: format!("systemctl status {}", service_name),
                description: format!("Show {} service status", service_name),
                requires_root: false,
            }),
            _ => None,
        }
    }

    pub fn list_failed_services(&mut self) -> Result<Vec<ServiceInfo>> {
        let output = Command::new("systemctl")
            .args(&["list-units", "--type=service", "--state=failed", "--no-pager", "--plain"])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to list failed services"));
        }
        
        let mut failed_services = Vec::new();
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines().skip(1) {
            if line.trim().is_empty() || line.contains("LOAD") {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 1 {
                let service_name = parts[0];
                if let Ok(service_info) = self.get_service_info(service_name) {
                    failed_services.push(service_info);
                }
            }
        }
        
        Ok(failed_services)
    }

    pub fn list_active_services(&mut self) -> Result<Vec<ServiceInfo>> {
        let output = Command::new("systemctl")
            .args(&["list-units", "--type=service", "--state=active", "--no-pager", "--plain"])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to list active services"));
        }
        
        let mut active_services = Vec::new();
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines().skip(1) {
            if line.trim().is_empty() || line.contains("LOAD") {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 1 {
                let service_name = parts[0];
                if let Ok(service_info) = self.get_service_info(service_name) {
                    active_services.push(service_info);
                }
            }
        }
        
        Ok(active_services)
    }

    pub fn get_system_services_summary(&mut self) -> Result<HashMap<String, usize>> {
        let output = Command::new("systemctl")
            .args(&["list-units", "--type=service", "--all", "--no-pager", "--plain"])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to get services summary"));
        }
        
        let mut summary = HashMap::new();
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines().skip(1) {
            if line.trim().is_empty() || line.contains("LOAD") {
                continue;
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let state = parts[2]; // ACTIVE state
                *summary.entry(state.to_string()).or_insert(0) += 1;
            }
        }
        
        Ok(summary)
    }

    pub fn get_top_processes(&self, limit: usize) -> Result<Vec<ProcessInfo>> {
        let output = Command::new("ps")
            .args(&["aux", "--sort=-%cpu"])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to get process information"));
        }
        
        let mut processes = Vec::new();
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for (i, line) in output_str.lines().enumerate() {
            if i == 0 || processes.len() >= limit {
                continue; // Skip header or if we have enough processes
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 11 {
                if let (Ok(pid), Ok(cpu), Ok(mem)) = (
                    parts[1].parse::<u32>(),
                    parts[2].parse::<f64>(),
                    parts[3].parse::<f64>(),
                ) {
                    let command = parts[10..].join(" ");
                    processes.push(ProcessInfo {
                        pid,
                        name: parts[10].to_string(),
                        status: parts[7].to_string(),
                        cpu_percent: cpu,
                        memory_percent: mem,
                        memory_usage: parts[5].to_string(),
                        user: parts[0].to_string(),
                        command,
                    });
                }
            }
        }
        
        Ok(processes)
    }

    pub fn search_services(&mut self, query: &str) -> Result<Vec<ServiceInfo>> {
        self.list_services(Some(query))
    }

    pub fn get_service_logs(&self, service_name: &str, lines: usize) -> Result<String> {
        let output = Command::new("journalctl")
            .args(&["-u", service_name, "--no-pager", "-n", &lines.to_string()])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to get service logs"));
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn format_bytes(&self, bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        format!("{:.1}{}", size, UNITS[unit_index])
    }

    fn get_process_uptime(&self, pid: u32) -> Option<String> {
        let output = Command::new("ps")
            .args(&["-o", "etime=", "-p", &pid.to_string()])
            .output()
            .ok()?;
        
        if output.status.success() {
            let uptime = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !uptime.is_empty() {
                return Some(uptime);
            }
        }
        
        None
    }

    pub fn is_service_running(&self, service_name: &str) -> Result<bool> {
        let output = Command::new("systemctl")
            .args(&["is-active", service_name, "--quiet"])
            .output()?;
        
        Ok(output.status.success())
    }

    pub fn is_service_enabled(&self, service_name: &str) -> Result<bool> {
        let output = Command::new("systemctl")
            .args(&["is-enabled", service_name, "--quiet"])
            .output()?;
        
        Ok(output.status.success())
    }

    pub fn get_service_dependencies(&self, service_name: &str) -> Result<Vec<String>> {
        let output = Command::new("systemctl")
            .args(&["list-dependencies", service_name, "--no-pager", "--plain"])
            .output()?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to get service dependencies"));
        }
        
        let mut dependencies = Vec::new();
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines().skip(1) {
            let trimmed = line.trim_start_matches(['●', '○', '├', '└', '─', ' ']);
            if !trimmed.is_empty() && trimmed != service_name {
                dependencies.push(trimmed.to_string());
            }
        }
        
        Ok(dependencies)
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}
