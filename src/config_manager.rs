use anyhow::{Context, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub cache_duration: u64, // in seconds, default 300 (5 minutes)
    pub enable_aur: bool,    // Enable AUR support for Arch-based systems
    pub enable_flatpak: bool, // Enable Flatpak support
    pub enable_snap: bool,   // Enable Snap support
    pub default_editor: Option<String>,
    pub auto_update_cache: bool,
    pub history_enabled: bool,
    pub backup_before_install: bool,
    pub preferred_aur_helper: String, // paru, yay, etc.
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cache_duration: 300,
            enable_aur: true,
            enable_flatpak: true,
            enable_snap: false,
            default_editor: None,
            auto_update_cache: true,
            history_enabled: true,
            backup_before_install: false,
            preferred_aur_helper: "paru".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {config_path:?}"))?;
            
            let config: Config = toml::from_str(&content)
                .with_context(|| "Failed to parse config file")?;
            
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {parent:?}"))?;
        }
        
        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
        
        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {config_path:?}"))?;
        
        Ok(())
    }
    
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        
        Ok(config_dir.join("linux-distro-agent").join("config.toml"))
    }
}

pub struct ConfigManager {
    config: Config,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        Ok(Self { config })
    }

    pub fn load(&self) -> Result<Config> {
        Config::load()
    }

    pub fn save(&self) -> Result<()> {
        self.config.save()
    }

    pub fn reset(&mut self) -> Result<()> {
        self.config = Config::default();
        self.config.save()
    }

    pub fn edit(&self) -> Result<()> {
        let editor = self.config.default_editor.as_deref().unwrap_or("vi");
        let config_path = Config::config_path()?;
        std::process::Command::new(editor)
            .arg(config_path)
            .status()?;
        Ok(())
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "cache_duration" => self.config.cache_duration = value.parse()?,
            "enable_aur" => self.config.enable_aur = value.parse()?,
            "enable_flatpak" => self.config.enable_flatpak = value.parse()?,
            "enable_snap" => self.config.enable_snap = value.parse()?,
            "default_editor" => self.config.default_editor = Some(value.to_string()),
            "auto_update_cache" => self.config.auto_update_cache = value.parse()?,
            "history_enabled" => self.config.history_enabled = value.parse()?,
            "backup_before_install" => self.config.backup_before_install = value.parse()?,
            "preferred_aur_helper" => self.config.preferred_aur_helper = value.to_string(),
            _ => anyhow::bail!("Unknown configuration key: {}", key),
        }
        self.save()
    }
}
