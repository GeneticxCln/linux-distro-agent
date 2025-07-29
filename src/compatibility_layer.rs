use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use anyhow::Result;

/// Maps package names across different distributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMapping {
    /// Common/canonical package name
    pub canonical_name: String,
    /// Package names per distribution/package manager
    pub distro_packages: HashMap<String, String>,
    /// Optional description
    pub description: Option<String>,
    /// Package categories (dev-tools, multimedia, etc.)
    pub categories: Vec<String>,
}

/// Manages compatibility mappings between different Linux distributions
#[derive(Debug)]
pub struct CompatibilityLayer {
    /// Package mappings keyed by canonical name
    pub mappings: HashMap<String, PackageMapping>,
    /// Reverse lookup: distro package name -> canonical name
    reverse_mappings: HashMap<String, String>,
}

impl CompatibilityLayer {
    pub fn new() -> Self {
        let mut layer = Self {
            mappings: HashMap::new(),
            reverse_mappings: HashMap::new(),
        };
        layer.initialize_common_packages();
        layer
    }

    /// Add a package mapping
    pub fn add_mapping(&mut self, mapping: PackageMapping) {
        // Update reverse mappings
        for (distro, package) in &mapping.distro_packages {
            self.reverse_mappings.insert(
                format!("{}:{}", distro, package),
                mapping.canonical_name.clone()
            );
        }
        
        self.mappings.insert(mapping.canonical_name.clone(), mapping);
    }

    /// Get package name for a specific distribution
    pub fn get_package_for_distro(&self, canonical_name: &str, distro: &str) -> Option<String> {
        self.mappings.get(canonical_name)
            .and_then(|mapping| mapping.distro_packages.get(distro))
            .cloned()
    }

    /// Get canonical name from distro-specific package name
    pub fn get_canonical_name(&self, distro: &str, package_name: &str) -> Option<String> {
        let key = format!("{}:{}", distro, package_name);
        self.reverse_mappings.get(&key).cloned()
    }

    /// Get install command for a canonical package on a specific distro
    pub fn get_install_command(&self, canonical_name: &str, distro: &str) -> Option<String> {
        if let Some(package_name) = self.get_package_for_distro(canonical_name, distro) {
            match distro {
                "arch" | "pacman" => Some(format!("sudo pacman -S --noconfirm {}", package_name)),
                "debian" | "ubuntu" | "apt" => Some(format!("sudo apt update && sudo apt install -y {}", package_name)),
                "fedora" | "rhel" | "centos" | "dnf" => Some(format!("sudo dnf install -y {}", package_name)),
                "opensuse" | "zypper" => Some(format!("sudo zypper install -y {}", package_name)),
                "gentoo" | "portage" => Some(format!("sudo emerge {}", package_name)),
                "nixos" | "nix" => Some(format!("nix-env -i {}", package_name)),
                "alpine" | "apk" => Some(format!("sudo apk add {}", package_name)),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Search for packages by category
    pub fn get_packages_by_category(&self, category: &str) -> Vec<&PackageMapping> {
        self.mappings.values()
            .filter(|mapping| mapping.categories.contains(&category.to_string()))
            .collect()
    }

    /// Get all available categories
    pub fn get_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.mappings.values()
            .flat_map(|mapping| &mapping.categories)
            .cloned()
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }

    /// Initialize common package mappings
    fn initialize_common_packages(&mut self) {
        // Development tools
        self.add_mapping(PackageMapping {
            canonical_name: "git".to_string(),
            distro_packages: [
                ("arch".to_string(), "git".to_string()),
                ("cachyos".to_string(), "git".to_string()),
                ("endeavouros".to_string(), "git".to_string()),
                ("manjaro".to_string(), "git".to_string()),
                ("debian".to_string(), "git".to_string()),
                ("ubuntu".to_string(), "git".to_string()),
                ("pop".to_string(), "git".to_string()),
                ("elementary".to_string(), "git".to_string()),
                ("fedora".to_string(), "git".to_string()),
                ("rhel".to_string(), "git".to_string()),
                ("centos".to_string(), "git".to_string()),
                ("rocky".to_string(), "git".to_string()),
                ("almalinux".to_string(), "git".to_string()),
                ("opensuse".to_string(), "git".to_string()),
                ("opensuse-leap".to_string(), "git".to_string()),
                ("opensuse-tumbleweed".to_string(), "git".to_string()),
                ("gentoo".to_string(), "dev-vcs/git".to_string()),
                ("nixos".to_string(), "git".to_string()),
                ("alpine".to_string(), "git".to_string()),
                ("void".to_string(), "git".to_string()),
            ].into(),
            description: Some("Git version control system".to_string()),
            categories: vec!["dev-tools".to_string(), "vcs".to_string()],
        });

        self.add_mapping(PackageMapping {
            canonical_name: "gcc".to_string(),
            distro_packages: [
                ("arch".to_string(), "gcc".to_string()),
                ("debian".to_string(), "gcc".to_string()),
                ("ubuntu".to_string(), "gcc".to_string()),
                ("fedora".to_string(), "gcc".to_string()),
                ("opensuse".to_string(), "gcc".to_string()),
                ("gentoo".to_string(), "sys-devel/gcc".to_string()),
                ("nixos".to_string(), "gcc".to_string()),
                ("alpine".to_string(), "gcc".to_string()),
            ].into(),
            description: Some("GNU Compiler Collection".to_string()),
            categories: vec!["dev-tools".to_string(), "compiler".to_string()],
        });

        // Text editors
        self.add_mapping(PackageMapping {
            canonical_name: "vim".to_string(),
            distro_packages: [
                ("arch".to_string(), "vim".to_string()),
                ("debian".to_string(), "vim".to_string()),
                ("ubuntu".to_string(), "vim".to_string()),
                ("fedora".to_string(), "vim-enhanced".to_string()),
                ("opensuse".to_string(), "vim".to_string()),
                ("gentoo".to_string(), "app-editors/vim".to_string()),
                ("nixos".to_string(), "vim".to_string()),
                ("alpine".to_string(), "vim".to_string()),
            ].into(),
            description: Some("Vi IMproved text editor".to_string()),
            categories: vec!["editors".to_string(), "terminal".to_string()],
        });

        // Network tools
        self.add_mapping(PackageMapping {
            canonical_name: "curl".to_string(),
            distro_packages: [
                ("arch".to_string(), "curl".to_string()),
                ("debian".to_string(), "curl".to_string()),
                ("ubuntu".to_string(), "curl".to_string()),
                ("fedora".to_string(), "curl".to_string()),
                ("opensuse".to_string(), "curl".to_string()),
                ("gentoo".to_string(), "net-misc/curl".to_string()),
                ("nixos".to_string(), "curl".to_string()),
                ("alpine".to_string(), "curl".to_string()),
            ].into(),
            description: Some("Command line tool for transferring data with URLs".to_string()),
            categories: vec!["network".to_string(), "tools".to_string()],
        });

        // Media tools
        self.add_mapping(PackageMapping {
            canonical_name: "ffmpeg".to_string(),
            distro_packages: [
                ("arch".to_string(), "ffmpeg".to_string()),
                ("debian".to_string(), "ffmpeg".to_string()),
                ("ubuntu".to_string(), "ffmpeg".to_string()),
                ("fedora".to_string(), "ffmpeg".to_string()),
                ("opensuse".to_string(), "ffmpeg".to_string()),
                ("gentoo".to_string(), "media-video/ffmpeg".to_string()),
                ("nixos".to_string(), "ffmpeg".to_string()),
                ("alpine".to_string(), "ffmpeg".to_string()),
            ].into(),
            description: Some("Complete solution to record, convert and stream audio and video".to_string()),
            categories: vec!["multimedia".to_string(), "video".to_string(), "audio".to_string()],
        });

        // System tools
        self.add_mapping(PackageMapping {
            canonical_name: "htop".to_string(),
            distro_packages: [
                ("arch".to_string(), "htop".to_string()),
                ("debian".to_string(), "htop".to_string()),
                ("ubuntu".to_string(), "htop".to_string()),
                ("fedora".to_string(), "htop".to_string()),
                ("opensuse".to_string(), "htop".to_string()),
                ("gentoo".to_string(), "sys-process/htop".to_string()),
                ("nixos".to_string(), "htop".to_string()),
                ("alpine".to_string(), "htop".to_string()),
            ].into(),
            description: Some("Interactive process viewer".to_string()),
            categories: vec!["system".to_string(), "monitoring".to_string()],
        });

        // Python
        self.add_mapping(PackageMapping {
            canonical_name: "python3".to_string(),
            distro_packages: [
                ("arch".to_string(), "python".to_string()),
                ("debian".to_string(), "python3".to_string()),
                ("ubuntu".to_string(), "python3".to_string()),
                ("fedora".to_string(), "python3".to_string()),
                ("opensuse".to_string(), "python3".to_string()),
                ("gentoo".to_string(), "dev-lang/python".to_string()),
                ("nixos".to_string(), "python3".to_string()),
                ("alpine".to_string(), "python3".to_string()),
            ].into(),
            description: Some("Python 3 programming language".to_string()),
            categories: vec!["dev-tools".to_string(), "programming".to_string()],
        });

        // Build systems
        self.add_mapping(PackageMapping {
            canonical_name: "make".to_string(),
            distro_packages: [
                ("arch".to_string(), "make".to_string()),
                ("debian".to_string(), "make".to_string()),
                ("ubuntu".to_string(), "make".to_string()),
                ("fedora".to_string(), "make".to_string()),
                ("opensuse".to_string(), "make".to_string()),
                ("gentoo".to_string(), "sys-devel/make".to_string()),
                ("nixos".to_string(), "gnumake".to_string()),
                ("alpine".to_string(), "make".to_string()),
            ].into(),
            description: Some("GNU Make build automation tool".to_string()),
            categories: vec!["dev-tools".to_string(), "build".to_string()],
        });
    }

    /// Load additional mappings from a configuration file
    pub fn load_from_file(&mut self, path: &std::path::Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let mappings: Vec<PackageMapping> = serde_json::from_str(&content)?;
        
        for mapping in mappings {
            self.add_mapping(mapping);
        }
        
        Ok(())
    }

    /// Save current mappings to a file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        let mappings: Vec<&PackageMapping> = self.mappings.values().collect();
        let content = serde_json::to_string_pretty(&mappings)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get similar packages (fuzzy matching)
    pub fn find_similar_packages(&self, query: &str) -> Vec<&PackageMapping> {
        let query_lower = query.to_lowercase();
        self.mappings.values()
            .filter(|mapping| {
                mapping.canonical_name.to_lowercase().contains(&query_lower) ||
                mapping.description.as_ref()
                    .map_or(false, |desc| desc.to_lowercase().contains(&query_lower)) ||
                mapping.distro_packages.values()
                    .any(|pkg| pkg.to_lowercase().contains(&query_lower))
            })
            .collect()
    }
}

impl Default for CompatibilityLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_mapping() {
        let compat = CompatibilityLayer::new();
        
        // Test getting package for distro
        assert_eq!(
            compat.get_package_for_distro("git", "arch"),
            Some("git".to_string())
        );
        assert_eq!(
            compat.get_package_for_distro("git", "gentoo"),
            Some("dev-vcs/git".to_string())
        );
        
        // Test getting canonical name
        assert_eq!(
            compat.get_canonical_name("gentoo", "dev-vcs/git"),
            Some("git".to_string())
        );
    }

    #[test]
    fn test_install_command() {
        let compat = CompatibilityLayer::new();
        
        let cmd = compat.get_install_command("git", "arch");
        assert!(cmd.is_some());
        assert!(cmd.unwrap().contains("pacman"));
        
        let cmd = compat.get_install_command("git", "debian");
        assert!(cmd.is_some());
        assert!(cmd.unwrap().contains("apt"));
    }

    #[test]
    fn test_categories() {
        let compat = CompatibilityLayer::new();
        let categories = compat.get_categories();
        assert!(categories.contains(&"dev-tools".to_string()));
        assert!(categories.contains(&"editors".to_string()));
    }
}
