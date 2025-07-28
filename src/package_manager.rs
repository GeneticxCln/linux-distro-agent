use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManager {
    pub name: String,
    pub install_cmd: String,
    pub remove_cmd: String,
    pub search_cmd: String,
    pub update_cmd: String,
    pub list_cmd: String,
    pub info_cmd: String,
    pub requires_sudo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageOperation {
    pub operation_type: OperationType,
    pub packages: Vec<String>,
    pub options: HashMap<String, String>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    Install,
    Remove,
    Update,
    Search,
    List,
    Info,
}

pub struct PackageManagerRegistry {
    managers: HashMap<String, PackageManager>,
}

impl PackageManagerRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            managers: HashMap::new(),
        };
        registry.initialize_default_managers();
        registry
    }

    fn initialize_default_managers(&mut self) {
        // Pacman (Arch Linux)
        self.managers.insert("pacman".to_string(), PackageManager {
            name: "pacman".to_string(),
            install_cmd: "sudo pacman -S --noconfirm".to_string(),
            remove_cmd: "sudo pacman -R".to_string(),
            search_cmd: "pacman -Ss".to_string(),
            update_cmd: "sudo pacman -Syu".to_string(),
            list_cmd: "pacman -Q".to_string(),
            info_cmd: "pacman -Si".to_string(),
            requires_sudo: true,
        });

        // APT (Debian/Ubuntu)
        self.managers.insert("apt".to_string(), PackageManager {
            name: "apt".to_string(),
            install_cmd: "sudo apt update && sudo apt install -y".to_string(),
            remove_cmd: "sudo apt remove".to_string(),
            search_cmd: "apt search".to_string(),
            update_cmd: "sudo apt update && sudo apt upgrade -y".to_string(),
            list_cmd: "apt list --installed".to_string(),
            info_cmd: "apt show".to_string(),
            requires_sudo: true,
        });

        // DNF (Fedora/RHEL)
        self.managers.insert("dnf".to_string(), PackageManager {
            name: "dnf".to_string(),
            install_cmd: "sudo dnf install -y".to_string(),
            remove_cmd: "sudo dnf remove".to_string(),
            search_cmd: "dnf search".to_string(),
            update_cmd: "sudo dnf update -y".to_string(),
            list_cmd: "dnf list installed".to_string(),
            info_cmd: "dnf info".to_string(),
            requires_sudo: true,
        });

        // Zypper (openSUSE)
        self.managers.insert("zypper".to_string(), PackageManager {
            name: "zypper".to_string(),
            install_cmd: "sudo zypper install -y".to_string(),
            remove_cmd: "sudo zypper remove".to_string(),
            search_cmd: "zypper search".to_string(),
            update_cmd: "sudo zypper update -y".to_string(),
            list_cmd: "zypper search --installed".to_string(),
            info_cmd: "zypper info".to_string(),
            requires_sudo: true,
        });

        // Portage (Gentoo)
        self.managers.insert("portage".to_string(), PackageManager {
            name: "portage".to_string(),
            install_cmd: "sudo emerge".to_string(),
            remove_cmd: "sudo emerge --unmerge".to_string(),
            search_cmd: "emerge --search".to_string(),
            update_cmd: "sudo emerge --sync && sudo emerge -uDN @world".to_string(),
            list_cmd: "qlist -I".to_string(),
            info_cmd: "emerge --info".to_string(),
            requires_sudo: true,
        });

        // Nix (NixOS)
        self.managers.insert("nix".to_string(), PackageManager {
            name: "nix".to_string(),
            install_cmd: "nix-env -i".to_string(),
            remove_cmd: "nix-env -e".to_string(),
            search_cmd: "nix search".to_string(),
            update_cmd: "nix-channel --update && nix-env -u".to_string(),
            list_cmd: "nix-env -q".to_string(),
            info_cmd: "nix-env -qa --description".to_string(),
            requires_sudo: false,
        });

        // APK (Alpine Linux)
        self.managers.insert("apk".to_string(), PackageManager {
            name: "apk".to_string(),
            install_cmd: "sudo apk add".to_string(),
            remove_cmd: "sudo apk del".to_string(),
            search_cmd: "apk search".to_string(),
            update_cmd: "sudo apk update && sudo apk upgrade".to_string(),
            list_cmd: "apk list --installed".to_string(),
            info_cmd: "apk info".to_string(),
            requires_sudo: true,
        });
    }

}

impl Default for PackageManagerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

