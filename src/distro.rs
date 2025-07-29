use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use anyhow::Result;
use crate::compatibility_layer::CompatibilityLayer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroInfo {
    pub name: String,
    pub version: Option<String>,
    pub id: Option<String>,
    pub id_like: Option<String>,
    pub version_id: Option<String>,
    pub pretty_name: Option<String>,
    pub home_url: Option<String>,
    pub support_url: Option<String>,
    pub bug_report_url: Option<String>,
    pub package_manager: Option<String>,
}

impl DistroInfo {
    pub fn detect() -> Result<Self> {
        let contents = fs::read_to_string("/etc/os-release")?;
        Self::parse_from_os_release(&contents)
    }

    pub fn parse_from_os_release(contents: &str) -> Result<Self> {
        if contents.trim().is_empty() {
            return Err(anyhow::anyhow!("Empty os-release file"));
        }

        let mut fields = HashMap::new();
        
        for line in contents.lines() {
            if let Some((key, value)) = line.split_once('=') {
                let clean_value = value.trim_matches('"');
                fields.insert(key.trim().to_string(), clean_value.to_string());
            }
        }

        let package_manager = Self::detect_package_manager(&fields);

        Ok(DistroInfo {
            name: fields.get("NAME").cloned().unwrap_or_else(|| "Unknown".to_string()),
            version: fields.get("VERSION").cloned(),
            id: fields.get("ID").cloned(),
            id_like: fields.get("ID_LIKE").cloned(),
            version_id: fields.get("VERSION_ID").cloned(),
            pretty_name: fields.get("PRETTY_NAME").cloned(),
            home_url: fields.get("HOME_URL").cloned(),
            support_url: fields.get("SUPPORT_URL").cloned(),
            bug_report_url: fields.get("BUG_REPORT_URL").cloned(),
            package_manager,
        })
    }

    fn detect_package_manager(fields: &HashMap<String, String>) -> Option<String> {
        let id = fields.get("ID")?.to_lowercase();
        let id_like = fields.get("ID_LIKE").map(|s| s.to_lowercase());

        match id.as_str() {
            "arch" | "cachyos" | "endeavouros" | "manjaro" => Some("pacman".to_string()),
            "ubuntu" | "debian" | "pop" | "elementary" => Some("apt".to_string()),
            "fedora" | "rhel" | "centos" | "rocky" | "almalinux" => Some("dnf".to_string()),
            "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" => Some("zypper".to_string()),
            "gentoo" => Some("portage".to_string()),
            "nixos" => Some("nix".to_string()),
            "alpine" => Some("apk".to_string()),
            "void" => Some("xbps".to_string()),
            _ => {
                if let Some(id_like) = id_like {
                    if id_like.contains("arch") {
                        Some("pacman".to_string())
                    } else if id_like.contains("debian") || id_like.contains("ubuntu") {
                        Some("apt".to_string())
                    } else if id_like.contains("fedora") || id_like.contains("rhel") {
                        Some("dnf".to_string())
                    } else if id_like.contains("suse") {
                        Some("zypper".to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn get_package_install_command(&self, package: &str) -> Option<String> {
        let compatibility_layer = CompatibilityLayer::new();
        
        let final_package = compatibility_layer.get_package_for_distro(package, self.id.as_deref().unwrap_or(""))
            .unwrap_or_else(|| package.to_string());

        match self.package_manager.as_deref() {
            Some("pacman") => Some(format!("sudo pacman -S {}", final_package)),
            Some("apt") => Some(format!("sudo apt install {}", final_package)),
            Some("dnf") => Some(format!("sudo dnf install {}", final_package)),
            Some("zypper") => Some(format!("sudo zypper install {}", final_package)),
            Some("portage") => Some(format!("sudo emerge {}", final_package)),
            Some("nix") => Some(format!("nix-env -iA nixpkgs.{}", final_package)),
            Some("apk") => Some(format!("sudo apk add {}", final_package)),
            Some("xbps") => Some(format!("sudo xbps-install {}", final_package)),
            Some("paru") => Some(format!("paru -S {}", final_package)),
            Some("yay") => Some(format!("yay -S {}", final_package)),
            Some("flatpak") => Some(format!("flatpak install {}", final_package)),
            Some("snap") => Some(format!("sudo snap install {}", final_package)),
            _ => None,
        }
    }

    pub fn get_package_search_command(&self, query: &str) -> Option<String> {
        match self.package_manager.as_deref() {
            Some("pacman") => Some(format!("pacman -Ss {query}")),
            Some("apt") => Some(format!("apt search {query}")),
            Some("dnf") => Some(format!("dnf search {query}")),
            Some("zypper") => Some(format!("zypper search {query}")),
            Some("portage") => Some(format!("emerge --search {query}")),
            Some("nix") => Some(format!("nix-env -qaP | grep {query}")),
            Some("apk") => Some(format!("apk search {query}")),
            _ => None,
        }
    }

    pub fn get_system_update_command(&self) -> Option<String> {
        match self.package_manager.as_deref() {
            Some("pacman") => Some("sudo pacman -Syu".to_string()),
            Some("apt") => Some("sudo apt update && sudo apt upgrade".to_string()),
            Some("dnf") => Some("sudo dnf upgrade".to_string()),
            Some("zypper") => Some("sudo zypper update".to_string()),
            Some("portage") => Some("sudo emerge --sync && sudo emerge -uDN @world".to_string()),
            Some("nix") => Some("sudo nixos-rebuild switch --upgrade".to_string()),
            Some("apk") => Some("sudo apk update && sudo apk upgrade".to_string()),
            _ => None,
        }
    }

    pub fn get_package_remove_command(&self, package: &str) -> Option<String> {
        let compatibility_layer = CompatibilityLayer::new();
        
        let final_package = compatibility_layer.get_package_for_distro(package, self.id.as_deref().unwrap_or(""))
            .unwrap_or_else(|| package.to_string());

        match self.package_manager.as_deref() {
            Some("pacman") => Some(format!("sudo pacman -R {}", final_package)),
            Some("apt") => Some(format!("sudo apt remove {}", final_package)),
            Some("dnf") => Some(format!("sudo dnf remove {}", final_package)),
            Some("zypper") => Some(format!("sudo zypper remove {}", final_package)),
            Some("portage") => Some(format!("sudo emerge --unmerge {}", final_package)),
            Some("nix") => Some(format!("nix-env -e {}", final_package)),
            Some("apk") => Some(format!("sudo apk del {}", final_package)),
            _ => None,
        }
    }

    pub fn get_package_list_command(&self, detailed: bool, filter: Option<&str>) -> Option<String> {
        match self.package_manager.as_deref() {
            Some("pacman") => Some(format!("pacman -Q{}{}", if detailed { "i" } else { "" }, filter.map(|f| format!(" | grep {f}")).unwrap_or_default())),
            Some("apt") => Some(format!("dpkg-query -l{}", filter.map(|f| format!(" | grep {f}")).unwrap_or_default())),
            Some("dnf") => Some(format!("dnf list installed{}", filter.map(|f| format!(" | grep {f}")).unwrap_or_default())),
            Some("zypper") => Some(format!("zypper se --installed-only{}", filter.map(|f| format!(" | grep {f}")).unwrap_or_default())),
            Some("portage") => Some(format!("equery list{}", filter.map(|f| format!(" | grep {f}")).unwrap_or_default())),
            Some("nix") => Some(format!("nix-env -q{}", filter.map(|f| format!(" | grep {f}")).unwrap_or_default())),
            Some("apk") => Some(format!("apk list --installed{}", filter.map(|f| format!(" | grep {f}")).unwrap_or_default())),
            _ => None,
        }
    }

    pub fn get_package_info_command(&self, package: &str) -> Option<String> {
        match self.package_manager.as_deref() {
            Some("pacman") => Some(format!("pacman -Qi {package}")),
            Some("apt") => Some(format!("apt show {package}")),
            Some("dnf") => Some(format!("dnf info {package}")),
            Some("zypper") => Some(format!("zypper info {package}")),
            Some("portage") => Some(format!("equery list {package}")),
            Some("nix") => Some(format!("nix-env -qaP | grep {package}")),
            Some("apk") => Some(format!("apk info {package}")),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cachyos() {
        let os_release = r#"NAME="CachyOS Linux"
PRETTY_NAME="CachyOS"
ID=cachyos
BUILD_ID=rolling
ANSI_COLOR="38;2;23;147;209"
HOME_URL="https://cachyos.org/"
DOCUMENTATION_URL="https://wiki.cachyos.org/"
SUPPORT_URL="https://discuss.cachyos.org/"
BUG_REPORT_URL="https://github.com/cachyos"
PRIVACY_POLICY_URL="https://cachyos.org/privacy_policy/"
LOGO=cachyos"#;

        let distro = DistroInfo::parse_from_os_release(os_release).unwrap();
        assert_eq!(distro.name, "CachyOS Linux");
        assert_eq!(distro.id, Some("cachyos".to_string()));
        assert_eq!(distro.package_manager, Some("pacman".to_string()));
        assert_eq!(distro.home_url, Some("https://cachyos.org/".to_string()));
    }

    #[test]
    fn test_parse_ubuntu() {
        let os_release = r#"NAME="Ubuntu"
VERSION="22.04.3 LTS (Jammy Jellyfish)"
ID=ubuntu
ID_LIKE=debian
PRETTY_NAME="Ubuntu 22.04.3 LTS"
VERSION_ID="22.04"
HOME_URL="https://www.ubuntu.com/"
SUPPORT_URL="https://help.ubuntu.com/"
BUG_REPORT_URL="https://bugs.launchpad.net/ubuntu/"
PRIVACY_POLICY_URL="https://www.ubuntu.com/legal/terms-and-policies/privacy-policy"
VERSION_CODENAME=jammy
UBUNTU_CODENAME=jammy"#;

        let distro = DistroInfo::parse_from_os_release(os_release).unwrap();
        assert_eq!(distro.name, "Ubuntu");
        assert_eq!(distro.id, Some("ubuntu".to_string()));
        assert_eq!(distro.package_manager, Some("apt".to_string()));
        assert_eq!(distro.version_id, Some("22.04".to_string()));
    }

    #[test]
    fn test_parse_fedora() {
        let os_release = r#"NAME="Fedora Linux"
VERSION="39 (Workstation Edition)"
ID=fedora
VERSION_ID=39
VERSION_CODENAME=""
PLATFORM_ID="platform:f39"
PRETTY_NAME="Fedora Linux 39 (Workstation Edition)"
ANSI_COLOR="0;38;2;60;110;180"
LOGO=fedora-logo-icon
CPE_NAME="cpe:/o:fedoraproject:fedora:39"
DEFAULT_HOSTNAME="fedora"
HOME_URL="https://fedoraproject.org/"
DOCUMENTATION_URL="https://docs.fedoraproject.org/en-US/fedora/f39/system-administrators-guide/"
SUPPORT_URL="https://ask.fedoraproject.org/"
BUG_REPORT_URL="https://bugzilla.redhat.com/"
REDHAT_BUGZILLA_PRODUCT="Fedora"
REDHAT_BUGZILLA_PRODUCT_VERSION=39
REDHAT_SUPPORT_PRODUCT="Fedora"
REDHAT_SUPPORT_PRODUCT_VERSION=39
SUPPORT_END=2024-11-12"#;

        let distro = DistroInfo::parse_from_os_release(os_release).unwrap();
        assert_eq!(distro.name, "Fedora Linux");
        assert_eq!(distro.id, Some("fedora".to_string()));
        assert_eq!(distro.package_manager, Some("dnf".to_string()));
        assert_eq!(distro.version_id, Some("39".to_string()));
    }

    #[test]
    fn test_unknown_distro_with_arch_like() {
        let os_release = r#"NAME="CustomArch"
ID=customarch
ID_LIKE=arch
PRETTY_NAME="CustomArch Linux""#;

        let distro = DistroInfo::parse_from_os_release(os_release).unwrap();
        assert_eq!(distro.name, "CustomArch");
        assert_eq!(distro.id, Some("customarch".to_string()));
        assert_eq!(distro.package_manager, Some("pacman".to_string()));
    }

    #[test]
    fn test_commands() {
        let distro = DistroInfo {
            name: "Test Linux".to_string(),
            version: None,
            id: Some("test".to_string()),
            id_like: None,
            version_id: None,
            pretty_name: None,
            home_url: None,
            support_url: None,
            bug_report_url: None,
            package_manager: Some("pacman".to_string()),
        };

        assert_eq!(
            distro.get_package_install_command("vim"),
            Some("sudo pacman -S vim".to_string())
        );
        assert_eq!(
            distro.get_package_search_command("git"),
            Some("pacman -Ss git".to_string())
        );
        assert_eq!(
            distro.get_system_update_command(),
            Some("sudo pacman -Syu".to_string())
        );
    }

    #[test]
    fn test_empty_os_release() {
        let os_release = "";
        let distro = DistroInfo::parse_from_os_release(os_release);
        assert!(distro.is_err());
    }

    #[test]
    fn test_partial_os_release() {
        let os_release = "NAME=Unknown\nID=unknown";
        let distro = DistroInfo::parse_from_os_release(os_release).unwrap();
        assert_eq!(distro.name, "Unknown");
        assert_eq!(distro.id, Some("unknown".to_string()));
    }

    #[test]
    fn test_wrong_fields_os_release() {
        let os_release = r#"WRONG_FIELD="Not Distro"
ID=nodistro
"#;

        let distro = DistroInfo::parse_from_os_release(os_release).unwrap();
        assert_eq!(distro.name, "Unknown");
        assert_eq!(distro.id, Some("nodistro".to_string()));
    }
}
