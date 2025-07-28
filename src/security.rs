use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityLevel::Low => write!(f, "Low"),
            SecurityLevel::Medium => write!(f, "Medium"),
            SecurityLevel::High => write!(f, "High"),
            SecurityLevel::Critical => write!(f, "Critical"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: SecurityLevel,
    pub category: SecurityCategory,
    pub recommendation: String,
    pub references: Vec<String>,
    pub affected_files: Vec<String>,
    pub cve_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityCategory {
    FilePermissions,
    NetworkSecurity,
    UserAccounts,
    SystemConfiguration,
    Services,
    Vulnerabilities,
    Compliance,
    AccessControl,
}

impl std::fmt::Display for SecurityCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityCategory::FilePermissions => write!(f, "FilePermissions"),
            SecurityCategory::NetworkSecurity => write!(f, "NetworkSecurity"),
            SecurityCategory::UserAccounts => write!(f, "UserAccounts"),
            SecurityCategory::SystemConfiguration => write!(f, "SystemConfiguration"),
            SecurityCategory::Services => write!(f, "Services"),
            SecurityCategory::Vulnerabilities => write!(f, "Vulnerabilities"),
            SecurityCategory::Compliance => write!(f, "Compliance"),
            SecurityCategory::AccessControl => write!(f, "AccessControl"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub timestamp: DateTime<Utc>,
    pub hostname: String,
    pub distribution: String,
    pub kernel_version: String,
    pub findings: Vec<SecurityFinding>,
    pub summary: SecuritySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySummary {
    pub total_findings: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub security_score: f64, // 0-100 scale
}

pub struct SecurityAuditor {
    findings: Vec<SecurityFinding>,
}

impl SecurityAuditor {
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
        }
    }

    pub fn run_full_audit(&mut self) -> Result<SecurityReport> {
        self.findings.clear();

        // Run various security checks
        self.check_file_permissions()?;
        self.check_user_accounts()?;
        self.check_network_configuration()?;
        self.check_system_services()?;
        self.check_system_configuration()?;
        self.check_installed_packages()?;
        self.check_ssh_configuration()?;
        self.check_firewall_status()?;

        self.generate_report()
    }

    fn check_file_permissions(&mut self) -> Result<()> {
        let sensitive_files = vec![
            "/etc/passwd",
            "/etc/shadow",
            "/etc/sudoers",
            "/etc/ssh/sshd_config",
            "/boot/grub/grub.cfg",
        ];

        for file_path in sensitive_files {
            if let Ok(metadata) = fs::metadata(file_path) {
                let _mode = metadata.permissions();
                
                // Check for world-writable files
                if self.is_world_writable(&metadata) {
                    self.findings.push(SecurityFinding {
                        id: format!("FILE_PERM_{}", file_path.replace('/', "_")),
                        title: format!("World-writable sensitive file: {}", file_path),
                        description: format!("The file {} is world-writable, which poses a security risk.", file_path),
                        severity: SecurityLevel::High,
                        category: SecurityCategory::FilePermissions,
                        recommendation: format!("Remove world-write permissions: chmod o-w {}", file_path),
                        references: vec!["CIS Controls".to_string()],
                        affected_files: vec![file_path.to_string()],
                        cve_ids: vec![],
                    });
                }

                // Check shadow file permissions
                if file_path == "/etc/shadow" && self.get_file_mode(&metadata) != 0o640 {
                    self.findings.push(SecurityFinding {
                        id: "SHADOW_PERM".to_string(),
                        title: "Incorrect shadow file permissions".to_string(),
                        description: "/etc/shadow file has incorrect permissions".to_string(),
                        severity: SecurityLevel::High,
                        category: SecurityCategory::FilePermissions,
                        recommendation: "Set correct permissions: chmod 640 /etc/shadow".to_string(),
                        references: vec!["CIS Benchmark".to_string()],
                        affected_files: vec!["/etc/shadow".to_string()],
                        cve_ids: vec![],
                    });
                }
            }
        }

        Ok(())
    }

    fn check_user_accounts(&mut self) -> Result<()> {
        // Check for users with empty passwords
        if let Ok(shadow_content) = fs::read_to_string("/etc/shadow") {
            for line in shadow_content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 && (parts[1].is_empty() || parts[1] == "!") {
                    // Skip system accounts that should be locked
                    if !self.is_system_user(parts[0]) {
                        self.findings.push(SecurityFinding {
                            id: format!("EMPTY_PASS_{}", parts[0]),
                            title: format!("User {} has empty or locked password", parts[0]),
                            description: "User account without proper password protection".to_string(),
                            severity: SecurityLevel::Medium,
                            category: SecurityCategory::UserAccounts,
                            recommendation: "Set a strong password or disable the account".to_string(),
                            references: vec!["Security Best Practices".to_string()],
                            affected_files: vec!["/etc/shadow".to_string()],
                            cve_ids: vec![],
                        });
                    }
                }
            }
        }

        // Check for root login capability
        if let Ok(passwd_content) = fs::read_to_string("/etc/passwd") {
            for line in passwd_content.lines() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 && parts[0] == "root" && parts[2] == "0" {
                    // Check if root has a shell
                    if parts.len() >= 7 && !parts[6].contains("nologin") && !parts[6].contains("false") {
                        self.findings.push(SecurityFinding {
                            id: "ROOT_LOGIN".to_string(),
                            title: "Root account has login shell".to_string(),
                            description: "Root account can be used for direct login".to_string(),
                            severity: SecurityLevel::Medium,
                            category: SecurityCategory::UserAccounts,
                            recommendation: "Consider disabling direct root login and using sudo".to_string(),
                            references: vec!["Security Hardening Guide".to_string()],
                            affected_files: vec!["/etc/passwd".to_string()],
                            cve_ids: vec![],
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn check_network_configuration(&mut self) -> Result<()> {
        // Check for open ports
        if let Ok(output) = Command::new("ss").args(&["-tuln"]).output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut listening_ports = Vec::new();
            
            for line in output_str.lines().skip(1) {
                if line.contains("LISTEN") {
                    if let Some(port_info) = self.extract_port_info(line) {
                        listening_ports.push(port_info);
                    }
                }
            }

            // Check for potentially dangerous open ports
            let dangerous_ports = vec![
                (21, "FTP"),
                (23, "Telnet"),
                (53, "DNS"),
                (69, "TFTP"),
                (135, "RPC"),
                (139, "NetBIOS"),
                (445, "SMB"),
                (1433, "SQL Server"),
                (3389, "RDP"),
            ];

            for (port, service) in dangerous_ports {
                if listening_ports.iter().any(|p| p.contains(&port.to_string())) {
                    self.findings.push(SecurityFinding {
                        id: format!("OPEN_PORT_{}", port),
                        title: format!("Potentially dangerous port {} ({}) is open", port, service),
                        description: format!("Port {} ({}) is listening and may pose security risks", port, service),
                        severity: SecurityLevel::Medium,
                        category: SecurityCategory::NetworkSecurity,
                        recommendation: format!("Review if {} service on port {} is necessary", service, port),
                        references: vec!["Network Security Guidelines".to_string()],
                        affected_files: vec![],
                        cve_ids: vec![],
                    });
                }
            }
        }

        Ok(())
    }

    fn check_system_services(&mut self) -> Result<()> {
        // Check for unnecessary running services
        let potentially_dangerous_services = vec![
            "telnet",
            "rsh",
            "rlogin",
            "vsftpd",
            "apache2",
            "nginx",
            "mysql",
            "postgresql",
        ];

        for service in potentially_dangerous_services {
            if let Ok(output) = Command::new("systemctl")
                .args(&["is-active", service])
                .output()
            {
                if output.status.success() {
                    let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if status == "active" {
                        self.findings.push(SecurityFinding {
                            id: format!("SERVICE_{}", service.to_uppercase()),
                            title: format!("Potentially unnecessary service {} is running", service),
                            description: format!("Service {} is active and may not be needed", service),
                            severity: SecurityLevel::Low,
                            category: SecurityCategory::Services,
                            recommendation: format!("Review if {} service is necessary", service),
                            references: vec!["Service Hardening Guide".to_string()],
                            affected_files: vec![],
                            cve_ids: vec![],
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn check_system_configuration(&mut self) -> Result<()> {
        // Check kernel parameters
        let security_params = vec![
            ("net.ipv4.ip_forward", "0"),
            ("net.ipv4.conf.all.send_redirects", "0"),
            ("net.ipv4.conf.default.send_redirects", "0"),
            ("net.ipv4.conf.all.accept_redirects", "0"),
            ("net.ipv4.conf.default.accept_redirects", "0"),
        ];

        for (param, expected_value) in security_params {
            if let Ok(output) = Command::new("sysctl")
                .args(&["-n", param])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let current_value = output_str.trim();
                if current_value != expected_value {
                    self.findings.push(SecurityFinding {
                        id: format!("SYSCTL_{}", param.replace('.', "_").to_uppercase()),
                        title: format!("Insecure kernel parameter: {}", param),
                        description: format!("Kernel parameter {} has value {} but should be {}", param, current_value, expected_value),
                        severity: SecurityLevel::Medium,
                        category: SecurityCategory::SystemConfiguration,
                        recommendation: format!("Set {} = {} in /etc/sysctl.conf", param, expected_value),
                        references: vec!["Kernel Hardening Guide".to_string()],
                        affected_files: vec!["/etc/sysctl.conf".to_string()],
                        cve_ids: vec![],
                    });
                }
            }
        }

        Ok(())
    }

    fn check_installed_packages(&mut self) -> Result<()> {
        // This would typically integrate with vulnerability databases
        // For now, we'll check for some commonly problematic packages
        let problematic_packages = vec![
            "telnet",
            "rsh-client",
            "rsh-redone-client",
            "talk",
            "ntalk",
        ];

        // This is a simplified check - in reality you'd query the package manager
        for package in problematic_packages {
            // Simulate package check (would use actual package manager queries)
            self.findings.push(SecurityFinding {
                id: format!("PKG_{}", package.to_uppercase()),
                title: format!("Potentially insecure package {} may be installed", package),
                description: format!("Package {} is known to have security issues", package),
                severity: SecurityLevel::Low,
                category: SecurityCategory::Vulnerabilities,
                recommendation: format!("Consider removing package {}", package),
                references: vec!["Package Security Advisory".to_string()],
                affected_files: vec![],
                cve_ids: vec![],
            });
        }

        Ok(())
    }

    fn check_ssh_configuration(&mut self) -> Result<()> {
        let ssh_config_path = "/etc/ssh/sshd_config";
        if let Ok(config_content) = fs::read_to_string(ssh_config_path) {
            let mut permit_root_login = true;
            let mut _password_auth = true;
            let mut permit_empty_passwords = false;

            for line in config_content.lines() {
                let line = line.trim();
                if line.starts_with("PermitRootLogin") && line.contains("no") {
                    permit_root_login = false;
                }
                if line.starts_with("PasswordAuthentication") && line.contains("no") {
                    _password_auth = false;
                }
                if line.starts_with("PermitEmptyPasswords") && line.contains("yes") {
                    permit_empty_passwords = true;
                }
            }

            if permit_root_login {
                self.findings.push(SecurityFinding {
                    id: "SSH_ROOT_LOGIN".to_string(),
                    title: "SSH root login is enabled".to_string(),
                    description: "SSH allows direct root login".to_string(),
                    severity: SecurityLevel::High,
                    category: SecurityCategory::AccessControl,
                    recommendation: "Set 'PermitRootLogin no' in /etc/ssh/sshd_config".to_string(),
                    references: vec!["SSH Hardening Guide".to_string()],
                    affected_files: vec![ssh_config_path.to_string()],
                    cve_ids: vec![],
                });
            }

            if permit_empty_passwords {
                self.findings.push(SecurityFinding {
                    id: "SSH_EMPTY_PASSWORDS".to_string(),
                    title: "SSH allows empty passwords".to_string(),
                    description: "SSH configuration permits empty passwords".to_string(),
                    severity: SecurityLevel::Critical,
                    category: SecurityCategory::AccessControl,
                    recommendation: "Set 'PermitEmptyPasswords no' in /etc/ssh/sshd_config".to_string(),
                    references: vec!["SSH Security Best Practices".to_string()],
                    affected_files: vec![ssh_config_path.to_string()],
                    cve_ids: vec![],
                });
            }
        }

        Ok(())
    }

    fn check_firewall_status(&mut self) -> Result<()> {
        // Check if firewall is active
        let firewall_tools = vec!["ufw", "firewalld", "iptables"];
        let mut firewall_active = false;

        for tool in firewall_tools {
            if let Ok(output) = Command::new(tool).args(&["status"]).output() {
                if output.status.success() {
                    let status = String::from_utf8_lossy(&output.stdout);
                    if status.contains("active") || status.contains("running") {
                        firewall_active = true;
                        break;
                    }
                }
            }
        }

        if !firewall_active {
            self.findings.push(SecurityFinding {
                id: "FIREWALL_INACTIVE".to_string(),
                title: "No active firewall detected".to_string(),
                description: "System does not appear to have an active firewall".to_string(),
                severity: SecurityLevel::High,
                category: SecurityCategory::NetworkSecurity,
                recommendation: "Enable and configure a firewall (ufw, firewalld, or iptables)".to_string(),
                references: vec!["Network Security Guidelines".to_string()],
                affected_files: vec![],
                cve_ids: vec![],
            });
        }

        Ok(())
    }

    fn generate_report(&self) -> Result<SecurityReport> {
        let summary = self.calculate_summary();
        
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| {
                Command::new("hostname")
                    .output()
                    .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            })
            .unwrap_or_else(|_| "unknown".to_string());

        let distribution = std::env::var("DISTRIB_ID")
            .unwrap_or_else(|_| "unknown".to_string());

        let kernel_version = Command::new("uname")
            .args(&["-r"])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(SecurityReport {
            timestamp: Utc::now(),
            hostname,
            distribution,
            kernel_version,
            findings: self.findings.clone(),
            summary,
        })
    }

    fn calculate_summary(&self) -> SecuritySummary {
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;

        for finding in &self.findings {
            match finding.severity {
                SecurityLevel::Critical => critical_count += 1,
                SecurityLevel::High => high_count += 1,
                SecurityLevel::Medium => medium_count += 1,
                SecurityLevel::Low => low_count += 1,
            }
        }

        // Calculate security score (0-100, higher is better)
        let total_issues = self.findings.len() as f64;
        let weighted_score = if total_issues > 0.0 {
            let penalty = (critical_count as f64 * 25.0) + 
                         (high_count as f64 * 15.0) + 
                         (medium_count as f64 * 8.0) + 
                         (low_count as f64 * 3.0);
            (100.0 - penalty).max(0.0)
        } else {
            100.0
        };

        SecuritySummary {
            total_findings: self.findings.len(),
            critical_count,
            high_count,
            medium_count,
            low_count,
            security_score: weighted_score,
        }
    }

    // Helper methods
    fn is_world_writable(&self, metadata: &std::fs::Metadata) -> bool {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o002 != 0
    }

    fn get_file_mode(&self, metadata: &std::fs::Metadata) -> u32 {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o777
    }

    fn is_system_user(&self, username: &str) -> bool {
        let system_users = vec![
            "root", "daemon", "bin", "sys", "sync", "games", "man", "lp",
            "mail", "news", "uucp", "proxy", "www-data", "backup", "list",
            "irc", "gnats", "nobody", "_apt", "systemd-network", "systemd-resolve",
        ];
        system_users.contains(&username)
    }

    fn extract_port_info(&self, line: &str) -> Option<String> {
        // Simple port extraction from ss output
        if let Some(parts) = line.split_whitespace().nth(4) {
            Some(parts.to_string())
        } else {
            None
        }
    }
}

impl SecurityAuditor {
    pub fn get_security_report_json(&self) -> Result<String> {
        // This method should generate a current security report and return as JSON
        let mut auditor = Self::new();
        let report = auditor.run_full_audit()?;
        Ok(serde_json::to_string_pretty(&report)?)
    }
}

impl SecurityReport {
    pub fn filter_by(&self, severity: Option<String>, category: Option<String>) -> Vec<&SecurityFinding> {
        self.findings.iter().filter(|finding| {
            let severity_match = if let Some(sev) = &severity {
                match sev.to_lowercase().as_str() {
                    "low" => matches!(finding.severity, SecurityLevel::Low),
                    "medium" => matches!(finding.severity, SecurityLevel::Medium),
                    "high" => matches!(finding.severity, SecurityLevel::High),
                    "critical" => matches!(finding.severity, SecurityLevel::Critical),
                    _ => true,
                }
            } else {
                true
            };
            
            let category_match = if let Some(cat) = &category {
                match cat.to_lowercase().as_str() {
                    "filepermissions" => matches!(finding.category, SecurityCategory::FilePermissions),
                    "networksecurity" => matches!(finding.category, SecurityCategory::NetworkSecurity),
                    "useraccounts" => matches!(finding.category, SecurityCategory::UserAccounts),
                    "systemconfiguration" => matches!(finding.category, SecurityCategory::SystemConfiguration),
                    "services" => matches!(finding.category, SecurityCategory::Services),
                    "vulnerabilities" => matches!(finding.category, SecurityCategory::Vulnerabilities),
                    "compliance" => matches!(finding.category, SecurityCategory::Compliance),
                    "accesscontrol" => matches!(finding.category, SecurityCategory::AccessControl),
                    _ => true,
                }
            } else {
                true
            };
            
            severity_match && category_match
        }).collect()
    }
}

impl Default for SecurityAuditor {
    fn default() -> Self {
        Self::new()
    }
}
