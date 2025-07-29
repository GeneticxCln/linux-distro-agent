use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;
use crate::config_manager::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageSource {
    Native,
    AUR,
    Flatpak,
    Snap,
    AppImage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSourceInfo {
    pub source: PackageSource,
    pub package_name: String,
    pub install_command: String,
    pub description: Option<String>,
    pub version: Option<String>,
}

pub struct PackageSourceManager {
    config: Config,
}

impl PackageSourceManager {
    pub fn new(_verbose: bool, _quiet: bool) -> Result<Self> {
        let config = Config::load().unwrap_or_default();
        
        Ok(Self {
            config,
        })
    }

    /// Search for a package across all available sources
    pub async fn search_package(&self, package_name: &str) -> Vec<PackageSourceInfo> {
        let mut sources = Vec::new();
        
        // Check AUR if enabled and on Arch-based system
        if self.config.enable_aur && self.is_arch_based() {
            if let Some(aur_info) = self.check_aur_package(package_name).await {
                sources.push(aur_info);
            }
        }
        
        // Check Flatpak if enabled
        if self.config.enable_flatpak {
            if let Some(flatpak_info) = self.check_flatpak_package(package_name).await {
                sources.push(flatpak_info);
            }
        }
        
        // Check Snap if enabled
        if self.config.enable_snap {
            if let Some(snap_info) = self.check_snap_package(package_name).await {
                sources.push(snap_info);
            }
        }
        
        sources
    }

    /// Search for packages in alternative sources
    pub async fn search_packages(&self, query: &str) -> Vec<String> {
        let mut results = Vec::new();

        // Use existing methods to check each source
        let sources = self.search_package(query).await;

        if sources.is_empty() {
            results.push(format!("No alternative packages found for '{}'", query));
        } else {
            results.push(format!("Found alternatives for '{}':", query));
            for source in sources {
                results.push(format!("- {} from {:?} ({:?})", source.package_name, source.source, source.description));
            }
        }

        results
    }

    /// Get the best available source for a package based on configuration preferences
    pub async fn get_best_source(&self, package_name: &str) -> Option<PackageSourceInfo> {
        let sources = self.search_package(package_name).await;
        
        if sources.is_empty() {
            return None;
        }
        
        // Priority: AUR (if Arch) > Flatpak > Snap
        // This can be made configurable later
        for source in &sources {
            match source.source {
                PackageSource::AUR if self.is_arch_based() => return Some(source.clone()),
                _ => continue,
            }
        }
        
        for source in &sources {
            match source.source {
                PackageSource::Flatpak => return Some(source.clone()),
                _ => continue,
            }
        }
        
        for source in &sources {
            match source.source {
                PackageSource::Snap => return Some(source.clone()),
                _ => continue,
            }
        }
        
        sources.first().cloned()
    }

    async fn check_aur_package(&self, package_name: &str) -> Option<PackageSourceInfo> {
        // Check if paru or yay is available
        let aur_helper = if self.command_exists("paru") {
            "paru"
        } else if self.command_exists("yay") {
            "yay"
        } else {
            return None;
        };

        // Use the AUR helper to search for the package
        let output = Command::new(aur_helper)
            .args(["-Ss", package_name])
            .output()
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // Parse the output to find exact matches
            for line in stdout.lines() {
                if line.contains(&format!("/{}", package_name)) || line.starts_with(&format!("{} ", package_name)) {
                    return Some(PackageSourceInfo {
                        source: PackageSource::AUR,
                        package_name: package_name.to_string(),
                        install_command: format!("{} -S {}", aur_helper, package_name),
                        description: self.extract_aur_description(&stdout, package_name),
                        version: None,
                    });
                }
            }
        }
        
        None
    }

    async fn check_flatpak_package(&self, package_name: &str) -> Option<PackageSourceInfo> {
        if !self.command_exists("flatpak") {
            return None;
        }

        let output = Command::new("flatpak")
            .args(["search", package_name])
            .output()
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // Parse flatpak search output
            for line in stdout.lines().skip(1) { // Skip header
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 3 {
                    let app_name = parts[0].trim();
                    let app_id = parts[2].trim();
                    
                    if app_name.to_lowercase().contains(&package_name.to_lowercase()) ||
                       app_id.to_lowercase().contains(&package_name.to_lowercase()) {
                        return Some(PackageSourceInfo {
                            source: PackageSource::Flatpak,
                            package_name: app_id.to_string(),
                            install_command: format!("flatpak install {}", app_id),
                            description: Some(parts.get(1).unwrap_or(&"").to_string()),
                            version: None,
                        });
                    }
                }
            }
        }
        
        None
    }

    async fn check_snap_package(&self, package_name: &str) -> Option<PackageSourceInfo> {
        if !self.command_exists("snap") {
            return None;
        }

        let output = Command::new("snap")
            .args(["find", package_name])
            .output()
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // Parse snap find output
            for line in stdout.lines().skip(1) { // Skip header
                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty() {
                    let snap_name = parts[0];
                    
                    if snap_name.to_lowercase().contains(&package_name.to_lowercase()) {
                        return Some(PackageSourceInfo {
                            source: PackageSource::Snap,
                            package_name: snap_name.to_string(),
                            install_command: format!("sudo snap install {}", snap_name),
                            description: parts.get(4..).map(|desc| desc.join(" ")),
                            version: parts.get(1).map(|v| v.to_string()),
                        });
                    }
                }
            }
        }
        
        None
    }

    fn command_exists(&self, command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn is_arch_based(&self) -> bool {
        // Check if we're on an Arch-based system
        std::fs::read_to_string("/etc/os-release")
            .map(|content| {
                content.contains("ID=arch") ||
                content.contains("ID=cachyos") ||
                content.contains("ID=endeavouros") ||
                content.contains("ID=manjaro") ||
                content.contains("ID_LIKE=arch")
            })
            .unwrap_or(false)
    }

    fn extract_aur_description(&self, output: &str, package_name: &str) -> Option<String> {
        let lines: Vec<&str> = output.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.contains(&format!("/{}", package_name)) || line.starts_with(&format!("{} ", package_name)) {
                // The description is typically on the next line
                if i + 1 < lines.len() {
                    let desc_line = lines[i + 1].trim();
                    if !desc_line.is_empty() && !desc_line.starts_with("aur/") {
                        return Some(desc_line.to_string());
                    }
                }
            }
        }
        None
    }

    /// Generate install suggestions when native package is not found
    pub async fn suggest_alternatives(&self, package_name: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        let sources = self.search_package(package_name).await;
        
        if sources.is_empty() {
            suggestions.push(format!("📦 No alternative sources found for '{}'", package_name));
            suggestions.push("💡 Try searching with a different name or check if the package exists".to_string());
        } else {
            suggestions.push(format!("📦 '{}' not found in native repositories, but available from:", package_name));
            
            for source in sources {
                match source.source {
                    PackageSource::AUR => {
                        suggestions.push(format!("  🔶 AUR: {}", source.install_command));
                        if let Some(desc) = source.description {
                            suggestions.push(format!("      {}", desc));
                        }
                    }
                    PackageSource::Flatpak => {
                        suggestions.push(format!("  📦 Flatpak: {}", source.install_command));
                        if let Some(desc) = source.description {
                            suggestions.push(format!("      {}", desc));
                        }
                    }
                    PackageSource::Snap => {
                        suggestions.push(format!("  🫰 Snap: {}", source.install_command));
                        if let Some(desc) = source.description {
                            suggestions.push(format!("      {}", desc));
                        }
                    }
                    _ => {}
                }
            }
        }
        
        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_source_info() {
        let info = PackageSourceInfo {
            source: PackageSource::AUR,
            package_name: "test-package".to_string(),
            install_command: "paru -S test-package".to_string(),
            description: Some("Test package".to_string()),
            version: None,
        };

        assert_eq!(info.package_name, "test-package");
        assert!(matches!(info.source, PackageSource::AUR));
    }

    #[tokio::test]
    async fn test_package_source_manager_creation() {
        let manager = PackageSourceManager::new(false, false);
        assert!(manager.is_ok());
    }
}
