// Repository Management System - v4.3.0
// Universal repository management across all Linux distributions

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use reqwest::Client;
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub priority: Option<u32>,
    pub gpg_key: Option<String>,
    pub architecture: Option<String>,
    pub components: Vec<String>,
    pub repo_type: RepositoryType,
    pub distribution: String,
    pub trusted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RepositoryType {
    Main,
    Universe,
    Multiverse,
    Restricted,
    Security,
    Updates,
    Backports,
    Proposed,
    Devel,
    Testing,
    Unstable,
    Contrib,
    NonFree,
    Snap,
    Flatpak,
    AppImage,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mirror {
    pub url: String,
    pub country: String,
    pub speed: Option<f64>,  // MB/s
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub repositories: Vec<Repository>,
    pub mirrors: HashMap<String, Vec<Mirror>>,
    pub auto_optimize: bool,
    pub check_signatures: bool,
    pub cache_duration: u64,  // seconds
}

impl Repository {
    pub fn new(name: &str, url: &str, distribution: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            enabled: true,
            priority: None,
            gpg_key: None,
            architecture: None,
            components: Vec::new(),
            repo_type: RepositoryType::Custom,
            distribution: distribution.to_string(),
            trusted: false,
        }
    }

    pub fn with_components(mut self, components: Vec<String>) -> Self {
        self.components = components;
        self
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn with_gpg_key(mut self, gpg_key: String) -> Self {
        self.gpg_key = Some(gpg_key);
        self.trusted = true;
        self
    }

    pub fn with_type(mut self, repo_type: RepositoryType) -> Self {
        self.repo_type = repo_type;
        self
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn format_sources_list_entry(&self) -> String {
        let enabled_prefix = if self.enabled { "" } else { "# " };
        let components_str = self.components.join(" ");
        
        match self.distribution.as_str() {
            "ubuntu" | "debian" => {
                format!(
                    "{}deb {} {} {}",
                    enabled_prefix,
                    self.url,
                    self.distribution,
                    components_str
                )
            }
            _ => {
                format!(
                    "{}[{}]\nname={}\nbaseurl={}\nenabled={}\ngpgcheck={}",
                    enabled_prefix,
                    self.name,
                    self.name,
                    self.url,
                    if self.enabled { 1 } else { 0 },
                    if self.trusted { 1 } else { 0 }
                )
            }
        }
    }
}

impl Mirror {
    pub fn new(url: &str, country: &str) -> Self {
        Self {
            url: url.to_string(),
            country: country.to_string(),
            speed: None,
            last_sync: None,
            active: true,
        }
    }

    pub async fn test_speed(&mut self, client: &Client) -> Result<f64> {
        let start = std::time::Instant::now();
        
        // Test download of a small file (1MB) to measure speed
        let test_url = format!("{}/ls-lR.gz", self.url.trim_end_matches('/'));
        
        let response = timeout(
            Duration::from_secs(30),
            client.get(&test_url).send()
        ).await??;
        
        if response.status().is_success() {
            let content_length = response.content_length().unwrap_or(1024 * 1024);
            let elapsed = start.elapsed();
            let speed = (content_length as f64) / elapsed.as_secs_f64() / 1024.0 / 1024.0;
            self.speed = Some(speed);
            Ok(speed)
        } else {
            Err(anyhow!("Mirror test failed: HTTP {}", response.status()))
        }
    }
}

pub struct RepositoryManager {
    config: RepositoryConfig,
    distro_type: String,
    config_path: PathBuf,
    client: Client,
}

impl RepositoryManager {
    pub fn new(distro_type: &str) -> Result<Self> {
        let config_path = Self::get_config_path(distro_type)?;
        let config = Self::load_or_create_config(&config_path)?;
        let client = Client::new();

        Ok(Self {
            config,
            distro_type: distro_type.to_string(),
            config_path,
            client,
        })
    }

    fn get_config_path(distro_type: &str) -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not find config directory"))?
            .join("lda");
        
        fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join(format!("repositories_{}.json", distro_type)))
    }

    fn load_or_create_config(config_path: &Path) -> Result<RepositoryConfig> {
        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(RepositoryConfig {
                repositories: Vec::new(),
                mirrors: HashMap::new(),
                auto_optimize: true,
                check_signatures: true,
                cache_duration: 3600, // 1 hour
            })
        }
    }

    pub fn save_config(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }

    pub fn add_repository(&mut self, repository: Repository) -> Result<()> {
        // Check if repository already exists
        if self.config.repositories.iter().any(|r| r.name == repository.name) {
            return Err(anyhow!("Repository '{}' already exists", repository.name));
        }

        self.config.repositories.push(repository.clone());
        self.save_config()?;
        self.apply_repository_changes()?;
        Ok(())
    }

    pub fn remove_repository(&mut self, repo_name: &str) -> Result<()> {
        let initial_len = self.config.repositories.len();
        self.config.repositories.retain(|r| r.name != repo_name);
        
        if self.config.repositories.len() == initial_len {
            return Err(anyhow!("Repository '{}' not found", repo_name));
        }

        self.save_config()?;
        self.apply_repository_changes()?;
        Ok(())
    }

    pub fn enable_repository(&mut self, repo_name: &str) -> Result<()> {
        if let Some(repo) = self.config.repositories.iter_mut().find(|r| r.name == repo_name) {
            repo.enable();
            self.save_config()?;
            self.apply_repository_changes()?;
            Ok(())
        } else {
            Err(anyhow!("Repository '{}' not found", repo_name))
        }
    }

    pub fn disable_repository(&mut self, repo_name: &str) -> Result<()> {
        if let Some(repo) = self.config.repositories.iter_mut().find(|r| r.name == repo_name) {
            repo.disable();
            self.save_config()?;
            self.apply_repository_changes()?;
            Ok(())
        } else {
            Err(anyhow!("Repository '{}' not found", repo_name))
        }
    }

    pub fn list_repositories(&self) -> Vec<&Repository> {
        self.config.repositories.iter().collect()
    }

    pub fn get_enabled_repositories(&self) -> Vec<&Repository> {
        self.config.repositories.iter().filter(|r| r.enabled).collect()
    }

    pub fn update_repository_lists(&self) -> Result<()> {
        match self.distro_type.as_str() {
            "ubuntu" | "debian" => {
                let output = Command::new("sudo")
                    .args(&["apt", "update"])
                    .output()?;
                
                if !output.status.success() {
                    return Err(anyhow!("Failed to update repository lists: {}", 
                        String::from_utf8_lossy(&output.stderr)));
                }
            }
            "fedora" | "rhel" | "centos" => {
                let output = Command::new("sudo")
                    .args(&["dnf", "makecache"])
                    .output()?;
                
                if !output.status.success() {
                    return Err(anyhow!("Failed to update repository cache: {}", 
                        String::from_utf8_lossy(&output.stderr)));
                }
            }
            "arch" => {
                let output = Command::new("sudo")
                    .args(&["pacman", "-Sy"])
                    .output()?;
                
                if !output.status.success() {
                    return Err(anyhow!("Failed to sync package databases: {}", 
                        String::from_utf8_lossy(&output.stderr)));
                }
            }
            "opensuse" => {
                let output = Command::new("sudo")
                    .args(&["zypper", "refresh"])
                    .output()?;
                
                if !output.status.success() {
                    return Err(anyhow!("Failed to refresh repositories: {}", 
                        String::from_utf8_lossy(&output.stderr)));
                }
            }
            _ => {
                return Err(anyhow!("Unsupported distribution: {}", self.distro_type));
            }
        }
        
        Ok(())
    }

    pub async fn optimize_mirrors(&mut self) -> Result<()> {
        for (repo_name, mirrors) in &mut self.config.mirrors {
            for mirror in mirrors.iter_mut() {
                if let Err(e) = mirror.test_speed(&self.client).await {
                    eprintln!("Failed to test mirror {}: {}", mirror.url, e);
                    mirror.active = false;
                }
            }
            
            // Sort mirrors by speed (fastest first)
            mirrors.sort_by(|a, b| {
                b.speed.unwrap_or(0.0).partial_cmp(&a.speed.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        
        self.save_config()?;
        Ok(())
    }

    pub fn add_mirror(&mut self, repo_name: &str, mirror: Mirror) -> Result<()> {
        self.config.mirrors
            .entry(repo_name.to_string())
            .or_insert_with(Vec::new)
            .push(mirror);
        
        self.save_config()?;
        Ok(())
    }

    pub fn get_best_mirror(&self, repo_name: &str) -> Option<&Mirror> {
        self.config.mirrors
            .get(repo_name)?
            .iter()
            .filter(|m| m.active)
            .max_by(|a, b| {
                a.speed.unwrap_or(0.0).partial_cmp(&b.speed.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn import_system_repositories(&mut self) -> Result<()> {
        match self.distro_type.as_str() {
            "ubuntu" | "debian" => self.import_apt_repositories(),
            "fedora" | "rhel" | "centos" => self.import_yum_repositories(),
            "arch" => self.import_pacman_repositories(),
            "opensuse" => self.import_zypper_repositories(),
            _ => Err(anyhow!("Unsupported distribution: {}", self.distro_type))
        }
    }

    fn import_apt_repositories(&mut self) -> Result<()> {
        let sources_list_path = Path::new("/etc/apt/sources.list");
        if sources_list_path.exists() {
            let content = fs::read_to_string(sources_list_path)?;
            for line in content.lines() {
                if let Ok(repo) = self.parse_apt_line(line) {
                    self.config.repositories.push(repo);
                }
            }
        }
        
        // Also check sources.list.d directory
        let sources_d_path = Path::new("/etc/apt/sources.list.d");
        if sources_d_path.exists() {
            for entry in fs::read_dir(sources_d_path)? {
                let entry = entry?;
                if entry.path().extension().map_or(false, |ext| ext == "list") {
                    let content = fs::read_to_string(entry.path())?;
                    for line in content.lines() {
                        if let Ok(repo) = self.parse_apt_line(line) {
                            self.config.repositories.push(repo);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    fn parse_apt_line(&self, line: &str) -> Result<Repository> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return Err(anyhow!("Invalid or commented line"));
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(anyhow!("Invalid sources.list format"));
        }

        let url = parts[1].to_string();
        let distribution = parts[2].to_string();
        let components = if parts.len() > 3 {
            parts[3..].iter().map(|&s| s.to_string()).collect()
        } else {
            vec!["main".to_string()]
        };

        let name = format!("{}-{}", distribution, components.join("-"));
        
        Ok(Repository {
            name,
            url,
            enabled: true,
            priority: None,
            gpg_key: None,
            architecture: None,
            components,
            repo_type: RepositoryType::Main,
            distribution,
            trusted: false,
        })
    }

    fn import_yum_repositories(&mut self) -> Result<()> {
        // Implementation for DNF/YUM repositories
        let yum_repos_d = Path::new("/etc/yum.repos.d");
        if yum_repos_d.exists() {
            for entry in fs::read_dir(yum_repos_d)? {
                let entry = entry?;
                if entry.path().extension().map_or(false, |ext| ext == "repo") {
                    // Parse .repo files (basic implementation)
                    // This would need more sophisticated INI parsing
                }
            }
        }
        Ok(())
    }

    fn import_pacman_repositories(&mut self) -> Result<()> {
        // Implementation for Pacman repositories
        let pacman_conf = Path::new("/etc/pacman.conf");
        if pacman_conf.exists() {
            // Parse pacman.conf (basic implementation)
            // This would need more sophisticated parsing
        }
        Ok(())
    }

    fn import_zypper_repositories(&mut self) -> Result<()> {
        // Implementation for Zypper repositories
        let output = Command::new("zypper")
            .args(&["lr", "-u"])
            .output()?;
        
        if output.status.success() {
            // Parse zypper repository list output
            // This would need proper parsing of the output format
        }
        Ok(())
    }

    fn apply_repository_changes(&self) -> Result<()> {
        match self.distro_type.as_str() {
            "ubuntu" | "debian" => self.apply_apt_changes(),
            "fedora" | "rhel" | "centos" => self.apply_yum_changes(),
            "arch" => self.apply_pacman_changes(),
            "opensuse" => self.apply_zypper_changes(),
            _ => Err(anyhow!("Unsupported distribution: {}", self.distro_type))
        }
    }

    fn apply_apt_changes(&self) -> Result<()> {
        // Generate sources.list content from our repositories
        let sources_content = self.config.repositories
            .iter()
            .filter(|r| r.distribution == "ubuntu" || r.distribution == "debian")
            .map(|r| r.format_sources_list_entry())
            .collect::<Vec<_>>()
            .join("\n");

        // Write to a custom sources file to avoid overwriting system files
        let custom_sources_path = "/etc/apt/sources.list.d/lda-managed.list";
        fs::write(custom_sources_path, sources_content)?;
        
        Ok(())
    }

    fn apply_yum_changes(&self) -> Result<()> {
        // Implementation for applying YUM/DNF repository changes
        Ok(())
    }

    fn apply_pacman_changes(&self) -> Result<()> {
        // Implementation for applying Pacman repository changes
        Ok(())
    }

    fn apply_zypper_changes(&self) -> Result<()> {
        // Implementation for applying Zypper repository changes
        Ok(())
    }

    pub fn validate_repository(&self, repo: &Repository) -> Result<bool> {
        // Basic URL validation
        if !repo.url.starts_with("http://") && !repo.url.starts_with("https://") {
            return Err(anyhow!("Repository URL must start with http:// or https://"));
        }

        // Additional validation logic can be added here
        Ok(true)
    }

    pub fn get_repository_statistics(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        
        stats.insert("total".to_string(), self.config.repositories.len());
        stats.insert("enabled".to_string(), 
            self.config.repositories.iter().filter(|r| r.enabled).count());
        stats.insert("disabled".to_string(), 
            self.config.repositories.iter().filter(|r| !r.enabled).count());
        stats.insert("trusted".to_string(), 
            self.config.repositories.iter().filter(|r| r.trusted).count());
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_creation() {
        let repo = Repository::new("test", "https://example.com/repo", "ubuntu");
        assert_eq!(repo.name, "test");
        assert_eq!(repo.url, "https://example.com/repo");
        assert!(repo.enabled);
    }

    #[test]
    fn test_repository_enable_disable() {
        let mut repo = Repository::new("test", "https://example.com/repo", "ubuntu");
        repo.disable();
        assert!(!repo.enabled);
        repo.enable();
        assert!(repo.enabled);
    }

    #[test]
    fn test_mirror_creation() {
        let mirror = Mirror::new("https://mirror.example.com", "US");
        assert_eq!(mirror.url, "https://mirror.example.com");
        assert_eq!(mirror.country, "US");
        assert!(mirror.active);
    }
}

