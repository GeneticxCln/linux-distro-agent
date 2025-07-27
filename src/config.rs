use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManagerConfig {
    pub install_command: String,
    pub search_command: String,
    pub update_command: String,
    pub remove_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionConfig {
    pub name: String,
    pub ids: Vec<String>,
    pub id_like: Option<Vec<String>>,
    pub package_manager: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub package_managers: HashMap<String, PackageManagerConfig>,
    pub distributions: Vec<DistributionConfig>,
}

impl Default for Config {
    fn default() -> Self {
        let mut package_managers = HashMap::new();
        
        package_managers.insert("pacman".to_string(), PackageManagerConfig {
            install_command: "sudo pacman -S {package}".to_string(),
            search_command: "pacman -Ss {query}".to_string(),
            update_command: "sudo pacman -Syu".to_string(),
            remove_command: "sudo pacman -R {package}".to_string(),
        });
        
        package_managers.insert("apt".to_string(), PackageManagerConfig {
            install_command: "sudo apt install {package}".to_string(),
            search_command: "apt search {query}".to_string(),
            update_command: "sudo apt update && sudo apt upgrade".to_string(),
            remove_command: "sudo apt remove {package}".to_string(),
        });
        
        package_managers.insert("dnf".to_string(), PackageManagerConfig {
            install_command: "sudo dnf install {package}".to_string(),
            search_command: "dnf search {query}".to_string(),
            update_command: "sudo dnf upgrade".to_string(),
            remove_command: "sudo dnf remove {package}".to_string(),
        });
        
        package_managers.insert("zypper".to_string(), PackageManagerConfig {
            install_command: "sudo zypper install {package}".to_string(),
            search_command: "zypper search {query}".to_string(),
            update_command: "sudo zypper update".to_string(),
            remove_command: "sudo zypper remove {package}".to_string(),
        });
        
        package_managers.insert("portage".to_string(), PackageManagerConfig {
            install_command: "sudo emerge {package}".to_string(),
            search_command: "emerge --search {query}".to_string(),
            update_command: "sudo emerge --sync && sudo emerge -uDN @world".to_string(),
            remove_command: "sudo emerge --unmerge {package}".to_string(),
        });
        
        package_managers.insert("nix".to_string(), PackageManagerConfig {
            install_command: "echo 'Add {package} to /etc/nixos/configuration.nix environment.systemPackages, then run: sudo nixos-rebuild switch'".to_string(),
            search_command: "nix search nixpkgs {query} --extra-experimental-features nix-command --extra-experimental-features flakes".to_string(),
            update_command: "sudo nixos-rebuild switch --upgrade".to_string(),
            remove_command: "echo 'Remove {package} from /etc/nixos/configuration.nix, then run: sudo nixos-rebuild switch'".to_string(),
        });
        
        package_managers.insert("apk".to_string(), PackageManagerConfig {
            install_command: "sudo apk add {package}".to_string(),
            search_command: "apk search {query}".to_string(),
            update_command: "sudo apk update && sudo apk upgrade".to_string(),
            remove_command: "sudo apk del {package}".to_string(),
        });

        let distributions = vec![
            DistributionConfig {
                name: "Arch-based".to_string(),
                ids: vec!["arch".to_string(), "cachyos".to_string(), "endeavouros".to_string(), "manjaro".to_string()],
                id_like: Some(vec!["arch".to_string()]),
                package_manager: "pacman".to_string(),
            },
            DistributionConfig {
                name: "Debian-based".to_string(),
                ids: vec!["ubuntu".to_string(), "debian".to_string(), "pop".to_string(), "elementary".to_string()],
                id_like: Some(vec!["debian".to_string(), "ubuntu".to_string()]),
                package_manager: "apt".to_string(),
            },
            DistributionConfig {
                name: "Red Hat-based".to_string(),
                ids: vec!["fedora".to_string(), "rhel".to_string(), "centos".to_string(), "rocky".to_string(), "almalinux".to_string()],
                id_like: Some(vec!["fedora".to_string(), "rhel".to_string()]),
                package_manager: "dnf".to_string(),
            },
            DistributionConfig {
                name: "SUSE-based".to_string(),
                ids: vec!["opensuse".to_string(), "opensuse-leap".to_string(), "opensuse-tumbleweed".to_string()],
                id_like: Some(vec!["suse".to_string()]),
                package_manager: "zypper".to_string(),
            },
            DistributionConfig {
                name: "Gentoo".to_string(),
                ids: vec!["gentoo".to_string()],
                id_like: None,
                package_manager: "portage".to_string(),
            },
            DistributionConfig {
                name: "NixOS".to_string(),
                ids: vec!["nixos".to_string()],
                id_like: None,
                package_manager: "nix".to_string(),
            },
            DistributionConfig {
                name: "Alpine".to_string(),
                ids: vec!["alpine".to_string()],
                id_like: None,
                package_manager: "apk".to_string(),
            },
        ];

        Config {
            package_managers,
            distributions,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
            
            let config: Config = toml::from_str(&contents)
                .with_context(|| "Failed to parse config file")?;
            
            Ok(config)
        } else {
            // Create default config file
            let default_config = Config::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        let contents = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
        
        fs::write(&config_path, contents)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
        
        Ok(())
    }

    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        
        Ok(config_dir.join("linux-distro-agent").join("config.toml"))
    }

    #[allow(dead_code)]
    pub fn get_config_file_path() -> Result<PathBuf> {
        Self::get_config_path()
    }

    pub fn detect_package_manager(&self, id: &str, id_like: Option<&str>) -> Option<String> {
        let id_lower = id.to_lowercase();
        
        // Check direct ID matches first
        for distro in &self.distributions {
            if distro.ids.iter().any(|dist_id| dist_id.to_lowercase() == id_lower) {
                return Some(distro.package_manager.clone());
            }
        }
        
        // Fall back to ID_LIKE matches
        if let Some(id_like) = id_like {
            let id_like_lower = id_like.to_lowercase();
            for distro in &self.distributions {
                if let Some(ref id_like_list) = distro.id_like {
                    for like_id in id_like_list {
                        if id_like_lower.contains(&like_id.to_lowercase()) {
                            return Some(distro.package_manager.clone());
                        }
                    }
                }
            }
        }
        
        None
    }

    #[allow(dead_code)]
    pub fn get_package_manager_config(&self, pm_name: &str) -> Option<&PackageManagerConfig> {
        self.package_managers.get(pm_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.package_managers.is_empty());
        assert!(!config.distributions.is_empty());
        assert!(config.package_managers.contains_key("pacman"));
        assert!(config.package_managers.contains_key("apt"));
    }

    #[test]
    fn test_detect_package_manager() {
        let config = Config::default();
        
        // Test direct ID match
        assert_eq!(config.detect_package_manager("cachyos", None), Some("pacman".to_string()));
        assert_eq!(config.detect_package_manager("ubuntu", None), Some("apt".to_string()));
        
        // Test ID_LIKE fallback
        assert_eq!(config.detect_package_manager("unknown", Some("arch")), Some("pacman".to_string()));
        assert_eq!(config.detect_package_manager("unknown", Some("debian")), Some("apt".to_string()));
        
        // Test no match
        assert_eq!(config.detect_package_manager("unknown", None), None);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        
        assert_eq!(config.package_managers.len(), deserialized.package_managers.len());
        assert_eq!(config.distributions.len(), deserialized.distributions.len());
    }
}
