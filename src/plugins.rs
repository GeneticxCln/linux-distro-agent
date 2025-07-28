use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub dependencies: Vec<String>,
    pub permissions: Vec<Permission>,
    pub entry_point: String,
    pub plugin_type: PluginType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginType {
    Command,        // Adds new CLI commands
    Monitor,        // System monitoring extensions
    Security,       // Security auditing extensions
    PackageManager, // Package manager extensions
    Distro,         // Distribution-specific extensions
    Integration,    // External service integrations
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    FileSystem(FileSystemPermission),
    Network(NetworkPermission),
    System(SystemPermission),
    Command(CommandPermission),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileSystemPermission {
    Read(String),     // Path pattern
    Write(String),    // Path pattern
    Execute(String),  // Path pattern
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkPermission {
    HttpClient,
    TcpConnect(String), // Host:Port pattern
    UdpConnect(String), // Host:Port pattern
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SystemPermission {
    ProcessList,
    SystemInfo,
    UserInfo,
    ServiceControl,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommandPermission {
    Execute(String), // Command pattern
    Sudo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub settings: HashMap<String, serde_json::Value>,
    pub permissions_granted: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub metadata: PluginMetadata,
    pub config: PluginConfig,
    pub path: PathBuf,
    pub status: PluginStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginStatus {
    Loaded,
    Failed(String),
    Disabled,
    NotFound,
}

pub trait Plugin {
    fn metadata(&self) -> &PluginMetadata;
    fn initialize(&mut self, config: &PluginConfig) -> Result<()>;
    fn execute(&self, args: &[String]) -> Result<String>;
    fn cleanup(&self) -> Result<()>;
}

pub struct PluginManager {
    plugins: HashMap<String, PluginInfo>,
    plugin_dirs: Vec<PathBuf>,
    config_dir: PathBuf,
}

impl PluginManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("linux-distro-agent")
            .join("plugins");

        let plugin_dirs = vec![
            config_dir.join("user"),
            PathBuf::from("/usr/share/linux-distro-agent/plugins"),
            PathBuf::from("/opt/linux-distro-agent/plugins"),
        ];

        Ok(Self {
            plugins: HashMap::new(),
            plugin_dirs,
            config_dir,
        })
    }

    pub fn discover_plugins(&mut self) -> Result<()> {
        for plugin_dir in self.plugin_dirs.clone() {
            if plugin_dir.exists() {
                self.scan_plugin_directory(&plugin_dir)?;
            }
        }
        Ok(())
    }

    fn scan_plugin_directory(&mut self, dir: &Path) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let manifest_path = path.join("plugin.toml");
                if manifest_path.exists() {
                    match self.load_plugin_metadata(&manifest_path) {
                        Ok(metadata) => {
                            let config = self.load_plugin_config(&metadata.name)?;
                            let plugin_info = PluginInfo {
                                metadata,
                                config,
                                path: path.clone(),
                                status: PluginStatus::Loaded,
                            };
                            self.plugins.insert(plugin_info.metadata.name.clone(), plugin_info);
                        }
                        Err(e) => {
                            eprintln!("Failed to load plugin from {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn load_plugin_metadata(&self, manifest_path: &Path) -> Result<PluginMetadata> {
        let content = fs::read_to_string(manifest_path)?;
        let metadata: PluginMetadata = toml::from_str(&content)?;
        Ok(metadata)
    }

    fn load_plugin_config(&self, plugin_name: &str) -> Result<PluginConfig> {
        let config_path = self.config_dir.join(format!("{}.toml", plugin_name));
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: PluginConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config
            let default_config = PluginConfig {
                enabled: false, // Plugins disabled by default for security
                settings: HashMap::new(),
                permissions_granted: vec![],
            };
            self.save_plugin_config(plugin_name, &default_config)?;
            Ok(default_config)
        }
    }

    fn save_plugin_config(&self, plugin_name: &str, config: &PluginConfig) -> Result<()> {
        fs::create_dir_all(&self.config_dir)?;
        let config_path = self.config_dir.join(format!("{}.toml", plugin_name));
        let content = toml::to_string_pretty(config)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn list_plugins(&self) -> Vec<&PluginInfo> {
        self.plugins.values().collect()
    }

    pub fn get_plugin(&self, name: &str) -> Option<&PluginInfo> {
        self.plugins.get(name)
    }

    pub fn enable_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.config.enabled = true;
            let config_clone = plugin.config.clone();
            self.save_plugin_config(name, &config_clone)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Plugin '{}' not found", name))
        }
    }

    pub fn disable_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            plugin.config.enabled = false;
            let config_clone = plugin.config.clone();
            self.save_plugin_config(name, &config_clone)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Plugin '{}' not found", name))
        }
    }

    pub fn execute_plugin(&self, name: &str, args: &[String]) -> Result<String> {
        if let Some(plugin) = self.plugins.get(name) {
            if !plugin.config.enabled {
                return Err(anyhow::anyhow!("Plugin '{}' is disabled", name));
            }

            // Security check: verify permissions
            self.check_plugin_permissions(plugin, args)?;

            // Execute plugin (this would typically load and run the plugin binary/script)
            self.run_plugin_executable(plugin, args)
        } else {
            Err(anyhow::anyhow!("Plugin '{}' not found", name))
        }
    }

    fn check_plugin_permissions(&self, plugin: &PluginInfo, args: &[String]) -> Result<()> {
        // This is a simplified permission check
        // In a real implementation, you'd have more sophisticated sandboxing
        
        for permission in &plugin.metadata.permissions {
            match permission {
                Permission::Command(CommandPermission::Sudo) => {
                    if !plugin.config.permissions_granted.contains(permission) {
                        return Err(anyhow::anyhow!(
                            "Plugin '{}' requires sudo permission but it's not granted", 
                            plugin.metadata.name
                        ));
                    }
                }
                Permission::Network(_) => {
                    if !plugin.config.permissions_granted.contains(permission) {
                        return Err(anyhow::anyhow!(
                            "Plugin '{}' requires network permission but it's not granted", 
                            plugin.metadata.name
                        ));
                    }
                }
                _ => {} // Other permission checks would go here
            }
        }

        Ok(())
    }

    fn run_plugin_executable(&self, plugin: &PluginInfo, args: &[String]) -> Result<String> {
        let entry_point = plugin.path.join(&plugin.metadata.entry_point);
        
        if !entry_point.exists() {
            return Err(anyhow::anyhow!(
                "Plugin entry point not found: {}", 
                entry_point.display()
            ));
        }

        // Determine how to execute the plugin based on its type
        let output = if entry_point.extension().and_then(|s| s.to_str()) == Some("py") {
            // Python plugin
            Command::new("python3")
                .arg(&entry_point)
                .args(args)
                .output()?
        } else if entry_point.is_executable() {
            // Binary plugin
            Command::new(&entry_point)
                .args(args)
                .output()?
        } else {
            return Err(anyhow::anyhow!(
                "Don't know how to execute plugin: {}", 
                entry_point.display()
            ));
        };

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!(
                "Plugin execution failed: {}", 
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    pub fn install_plugin(&mut self, plugin_path: &Path) -> Result<()> {
        // Validate plugin
        let manifest_path = plugin_path.join("plugin.toml");
        if !manifest_path.exists() {
            return Err(anyhow::anyhow!("Plugin manifest not found"));
        }

        let metadata = self.load_plugin_metadata(&manifest_path)?;
        
        // Copy plugin to user plugin directory
        let user_plugins_dir = self.config_dir.join("user");
        fs::create_dir_all(&user_plugins_dir)?;
        
        let target_dir = user_plugins_dir.join(&metadata.name);
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir)?;
        }
        
        self.copy_directory(plugin_path, &target_dir)?;
        
        // Load the installed plugin
        let config = self.load_plugin_config(&metadata.name)?;
        let plugin_info = PluginInfo {
            metadata,
            config,
            path: target_dir,
            status: PluginStatus::Loaded,
        };
        
        self.plugins.insert(plugin_info.metadata.name.clone(), plugin_info);
        
        Ok(())
    }

    pub fn uninstall_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(plugin) = self.plugins.get(name) {
            // Only allow uninstalling user plugins
            if plugin.path.starts_with(&self.config_dir.join("user")) {
                fs::remove_dir_all(&plugin.path)?;
                self.plugins.remove(name);
                
                // Remove config file
                let config_path = self.config_dir.join(format!("{}.toml", name));
                if config_path.exists() {
                    fs::remove_file(&config_path)?;
                }
                
                Ok(())
            } else {
                Err(anyhow::anyhow!("Cannot uninstall system plugin '{}'", name))
            }
        } else {
            Err(anyhow::anyhow!("Plugin '{}' not found", name))
        }
    }

    pub fn grant_permission(&mut self, plugin_name: &str, permission: Permission) -> Result<()> {
        if let Some(plugin) = self.plugins.get_mut(plugin_name) {
            if !plugin.config.permissions_granted.contains(&permission) {
                plugin.config.permissions_granted.push(permission);
                let config_clone = plugin.config.clone();
                self.save_plugin_config(plugin_name, &config_clone)?;
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Plugin '{}' not found", plugin_name))
        }
    }

    pub fn revoke_permission(&mut self, plugin_name: &str, permission: &Permission) -> Result<()> {
        if let Some(plugin) = self.plugins.get_mut(plugin_name) {
            plugin.config.permissions_granted.retain(|p| p != permission);
            let config_clone = plugin.config.clone();
            self.save_plugin_config(plugin_name, &config_clone)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Plugin '{}' not found", plugin_name))
        }
    }

    fn copy_directory(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if src_path.is_dir() {
                self.copy_directory(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    pub fn create_plugin_template(&self, name: &str, plugin_type: PluginType) -> Result<PathBuf> {
        let template_dir = self.config_dir.join("templates").join(name);
        fs::create_dir_all(&template_dir)?;

        // Create plugin manifest
        let metadata = PluginMetadata {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: format!("A {} plugin for Linux Distribution Agent", 
                match plugin_type {
                    PluginType::Command => "command",
                    PluginType::Monitor => "monitoring",
                    PluginType::Security => "security",
                    PluginType::PackageManager => "package manager",
                    PluginType::Distro => "distribution",
                    PluginType::Integration => "integration",
                }
            ),
            author: "Plugin Author".to_string(),
            license: "MIT".to_string(),
            dependencies: vec![],
            permissions: match plugin_type {
                PluginType::Security => vec![
                    Permission::FileSystem(FileSystemPermission::Read("/etc/*".to_string())),
                    Permission::System(SystemPermission::SystemInfo),
                ],
                PluginType::Monitor => vec![
                    Permission::System(SystemPermission::ProcessList),
                    Permission::System(SystemPermission::SystemInfo),
                ],
                _ => vec![],
            },
            entry_point: "main.py".to_string(),
            plugin_type,
        };

        let manifest_content = toml::to_string_pretty(&metadata)?;
        fs::write(template_dir.join("plugin.toml"), manifest_content)?;

        // Create sample Python script
        let python_script = format!(r#"#!/usr/bin/env python3
"""
{} Plugin for Linux Distribution Agent
"""

import sys
import json

def main():
    if len(sys.argv) < 2:
        print("Usage: {{}} <command> [args...]".format(sys.argv[0]))
        sys.exit(1)
    
    command = sys.argv[1]
    args = sys.argv[2:]
    
    if command == "info":
        print(json.dumps({{
            "name": "{}",
            "version": "0.1.0",
            "status": "running"
        }}))
    elif command == "execute":
        # Plugin-specific logic here
        print("Plugin {{}} executed with args: {{}}".format("{}", args))
    else:
        print("Unknown command: {{}}".format(command))
        sys.exit(1)

if __name__ == "__main__":
    main()
"#, metadata.description, name, name);
        fs::write(template_dir.join("main.py"), python_script)?;

        // Make script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(template_dir.join("main.py"))?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(template_dir.join("main.py"), perms)?;
        }

        // Create README
        let readme = format!(r#"# {} Plugin

## Description
{}

## Installation
1. Copy this directory to `~/.config/linux-distro-agent/plugins/user/{}`
2. Enable the plugin: `linux-distro-agent plugin enable {}`
3. Grant necessary permissions if required

## Usage
```bash
linux-distro-agent plugin exec {} info
linux-distro-agent plugin exec {} execute [args...]
```

## Configuration
Edit `~/.config/linux-distro-agent/plugins/{}.toml` to configure the plugin.
"#, name, metadata.description, name, name, name, name, name);

        fs::write(template_dir.join("README.md"), readme)?;

        Ok(template_dir)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            Self {
                plugins: HashMap::new(),
                plugin_dirs: vec![],
                config_dir: PathBuf::from("/tmp/lda-plugins"),
            }
        })
    }
}

// Helper trait to check if a path is executable
trait IsExecutable {
    fn is_executable(&self) -> bool;
}

impl IsExecutable for Path {
    fn is_executable(&self) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = self.metadata() {
                metadata.permissions().mode() & 0o111 != 0
            } else {
                false
            }
        }
        #[cfg(not(unix))]
        {
            // On non-Unix systems, assume files with certain extensions are executable
            if let Some(ext) = self.extension() {
                matches!(ext.to_str(), Some("exe") | Some("bat") | Some("cmd"))
            } else {
                false
            }
        }
    }
}
