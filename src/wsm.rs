use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSystemInfo {
    pub window_system: WindowSystem,
    pub desktop_environment: Option<DesktopEnvironment>,
    pub display_manager: Option<DisplayManager>,
    pub window_manager: Option<String>,
    pub session_type: SessionType,
    pub displays: Vec<DisplayInfo>,
    pub compositor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowSystem {
    X11,
    Wayland,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DesktopEnvironment {
    GNOME,
    KDE,
    XFCE,
    LXDE,
    LXQt,
    Mate,
    Cinnamon,
    Pantheon,
    Budgie,
    Enlightenment,
    #[serde(rename = "i3")]
    I3,
    Sway,
    Awesome,
    Openbox,
    Fluxbox,
    BSPWM,
    Qtile,
    DWM,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayManager {
    GDM,
    SDDM,
    LightDM,
    XDM,
    LXDM,
    Ly,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionType {
    X11,
    Wayland,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub name: String,
    pub resolution: String,
    pub refresh_rate: f64,
    pub is_primary: bool,
    pub position: (i32, i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WSMCommand {
    pub command: String,
    pub description: String,
    pub requires_root: bool,
}

pub struct WindowSystemManager {
    current_info: Option<WindowSystemInfo>,
}

impl WindowSystemManager {
    pub fn new() -> Self {
        Self {
            current_info: None,
        }
    }

    pub fn detect_window_system(&mut self) -> Result<WindowSystemInfo> {
        let window_system = self.detect_window_system_type()?;
        let desktop_environment = self.detect_desktop_environment();
        let display_manager = self.detect_display_manager();
        let window_manager = self.detect_window_manager();
        let session_type = self.detect_session_type();
        let displays = self.detect_displays()?;
        let compositor = self.detect_compositor();

        let info = WindowSystemInfo {
            window_system,
            desktop_environment,
            display_manager,
            window_manager,
            session_type,
            displays,
            compositor,
        };

        self.current_info = Some(info.clone());
        Ok(info)
    }

    fn detect_window_system_type(&self) -> Result<WindowSystem> {
        // Check for Wayland
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            return Ok(WindowSystem::Wayland);
        }

        // Check for X11
        if std::env::var("DISPLAY").is_ok() {
            return Ok(WindowSystem::X11);
        }

        // Check for running display servers
        if self.is_process_running("Xorg") || self.is_process_running("X") {
            return Ok(WindowSystem::X11);
        }

        if self.is_process_running("weston") || self.is_process_running("sway") || 
           self.is_process_running("mutter") {
            return Ok(WindowSystem::Wayland);
        }

        Ok(WindowSystem::Unknown)
    }

    fn detect_desktop_environment(&self) -> Option<DesktopEnvironment> {
        // Check environment variables
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            return match desktop.to_lowercase().as_str() {
                "gnome" => Some(DesktopEnvironment::GNOME),
                "kde" | "plasma" => Some(DesktopEnvironment::KDE),
                "xfce" => Some(DesktopEnvironment::XFCE),
                "lxde" => Some(DesktopEnvironment::LXDE),
                "lxqt" => Some(DesktopEnvironment::LXQt),
                "mate" => Some(DesktopEnvironment::Mate),
                "cinnamon" => Some(DesktopEnvironment::Cinnamon),
                "pantheon" => Some(DesktopEnvironment::Pantheon),
                "budgie" => Some(DesktopEnvironment::Budgie),
                "enlightenment" => Some(DesktopEnvironment::Enlightenment),
                "i3" => Some(DesktopEnvironment::I3),
                "sway" => Some(DesktopEnvironment::Sway),
                "awesome" => Some(DesktopEnvironment::Awesome),
                "openbox" => Some(DesktopEnvironment::Openbox),
                "fluxbox" => Some(DesktopEnvironment::Fluxbox),
                "bspwm" => Some(DesktopEnvironment::BSPWM),
                "qtile" => Some(DesktopEnvironment::Qtile),
                "dwm" => Some(DesktopEnvironment::DWM),
                _ => Some(DesktopEnvironment::Custom(desktop)),
            };
        }

        // Check for specific processes
        if self.is_process_running("gnome-shell") {
            return Some(DesktopEnvironment::GNOME);
        }
        if self.is_process_running("plasmashell") {
            return Some(DesktopEnvironment::KDE);
        }
        if self.is_process_running("xfce4-panel") {
            return Some(DesktopEnvironment::XFCE);
        }
        if self.is_process_running("lxpanel") {
            return Some(DesktopEnvironment::LXDE);
        }
        if self.is_process_running("mate-panel") {
            return Some(DesktopEnvironment::Mate);
        }
        if self.is_process_running("cinnamon") {
            return Some(DesktopEnvironment::Cinnamon);
        }
        if self.is_process_running("i3") {
            return Some(DesktopEnvironment::I3);
        }
        if self.is_process_running("sway") {
            return Some(DesktopEnvironment::Sway);
        }

        None
    }

    fn detect_display_manager(&self) -> Option<DisplayManager> {
        // Check for running display managers
        if self.is_process_running("gdm") || self.is_process_running("gdm3") {
            return Some(DisplayManager::GDM);
        }
        if self.is_process_running("sddm") {
            return Some(DisplayManager::SDDM);
        }
        if self.is_process_running("lightdm") {
            return Some(DisplayManager::LightDM);
        }
        if self.is_process_running("xdm") {
            return Some(DisplayManager::XDM);
        }
        if self.is_process_running("lxdm") {
            return Some(DisplayManager::LXDM);
        }
        if self.is_process_running("ly") {
            return Some(DisplayManager::Ly);
        }

        // Check systemd services
        if let Ok(output) = Command::new("systemctl")
            .args(&["is-active", "--quiet", "gdm"])
            .output()
        {
            if output.status.success() {
                return Some(DisplayManager::GDM);
            }
        }

        if let Ok(output) = Command::new("systemctl")
            .args(&["is-active", "--quiet", "sddm"])
            .output()
        {
            if output.status.success() {
                return Some(DisplayManager::SDDM);
            }
        }

        None
    }

    fn detect_window_manager(&self) -> Option<String> {
        if let Ok(wm) = std::env::var("WINDOW_MANAGER") {
            return Some(wm);
        }

        // Common window managers
        let window_managers = [
            "i3", "sway", "awesome", "dwm", "bspwm", "qtile", "openbox", 
            "fluxbox", "jwm", "fvwm", "ratpoison", "xmonad", "herbstluftwm"
        ];

        for wm in &window_managers {
            if self.is_process_running(wm) {
                return Some(wm.to_string());
            }
        }

        None
    }

    fn detect_session_type(&self) -> SessionType {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            SessionType::Wayland
        } else if std::env::var("DISPLAY").is_ok() {
            SessionType::X11
        } else {
            SessionType::Unknown
        }
    }

    fn detect_displays(&self) -> Result<Vec<DisplayInfo>> {
        let mut displays = Vec::new();

        // Try xrandr for X11
        if let Ok(output) = Command::new("xrandr").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if line.contains(" connected") {
                    if let Some(display) = self.parse_xrandr_line(line) {
                        displays.push(display);
                    }
                }
            }
        }

        // Fallback: create a default display if none found
        if displays.is_empty() {
            displays.push(DisplayInfo {
                name: "Unknown".to_string(),
                resolution: "Unknown".to_string(),
                refresh_rate: 60.0,
                is_primary: true,
                position: (0, 0),
            });
        }

        Ok(displays)
    }

    fn parse_xrandr_line(&self, line: &str) -> Option<DisplayInfo> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let name = parts[0].to_string();
            let is_primary = line.contains("primary");
            
            // Extract resolution and position
            if let Some(resolution_part) = parts.iter().find(|p| p.contains("x") && p.contains("+")) {
                let res_parts: Vec<&str> = resolution_part.split('+').collect();
                if res_parts.len() >= 3 {
                    let resolution = res_parts[0].to_string();
                    let x = res_parts[1].parse().unwrap_or(0);
                    let y = res_parts[2].parse().unwrap_or(0);
                    
                    return Some(DisplayInfo {
                        name,
                        resolution,
                        refresh_rate: 60.0,
                        is_primary,
                        position: (x, y),
                    });
                }
            }
        }
        None
    }

    fn detect_compositor(&self) -> Option<String> {
        let compositors = [
            "mutter", "kwin_x11", "kwin_wayland", "xfwm4", "openbox", 
            "compiz", "picom", "compton", "weston", "sway"
        ];

        for compositor in &compositors {
            if self.is_process_running(compositor) {
                return Some(compositor.to_string());
            }
        }

        None
    }

    fn is_process_running(&self, process_name: &str) -> bool {
        Command::new("pgrep")
            .arg(process_name)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn get_restart_command(&self, component: &str) -> Option<WSMCommand> {
        match component.to_lowercase().as_str() {
            "gdm" => Some(WSMCommand {
                command: "systemctl restart gdm".to_string(),
                description: "Restart GDM display manager".to_string(),
                requires_root: true,
            }),
            "sddm" => Some(WSMCommand {
                command: "systemctl restart sddm".to_string(),
                description: "Restart SDDM display manager".to_string(),
                requires_root: true,
            }),
            "lightdm" => Some(WSMCommand {
                command: "systemctl restart lightdm".to_string(),
                description: "Restart LightDM display manager".to_string(),
                requires_root: true,
            }),
            "x11" | "xorg" => Some(WSMCommand {
                command: "systemctl restart display-manager".to_string(),
                description: "Restart X11 display server".to_string(),
                requires_root: true,
            }),
            _ => None,
        }
    }

    pub fn get_switch_session_command(&self, session_type: &str) -> Option<WSMCommand> {
        match session_type.to_lowercase().as_str() {
            "wayland" => Some(WSMCommand {
                command: "logout and select Wayland session at login".to_string(),
                description: "Switch to Wayland session".to_string(),
                requires_root: false,
            }),
            "x11" => Some(WSMCommand {
                command: "logout and select X11 session at login".to_string(),
                description: "Switch to X11 session".to_string(),
                requires_root: false,
            }),
            _ => None,
        }
    }

    pub fn list_available_sessions(&self) -> Result<Vec<String>> {
        let mut sessions = Vec::new();
        
        // Check for desktop files in common session directories
        let session_dirs = [
            "/usr/share/xsessions",
            "/usr/share/wayland-sessions",
            "/usr/local/share/xsessions",
            "/usr/local/share/wayland-sessions",
        ];

        for dir in &session_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.ends_with(".desktop") {
                            let session_name = file_name.trim_end_matches(".desktop");
                            sessions.push(session_name.to_string());
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    pub fn get_display_configuration(&self) -> Result<HashMap<String, String>> {
        let mut config = HashMap::new();
        
        // Get current display configuration
        if let Ok(output) = Command::new("xrandr").arg("--current").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            config.insert("xrandr_output".to_string(), output_str.to_string());
        }

        // Get resolution information
        if let Some(info) = &self.current_info {
            for display in &info.displays {
                config.insert(
                    format!("display_{}", display.name),
                    format!("{}@{}Hz at ({},{})", 
                        display.resolution, 
                        display.refresh_rate,
                        display.position.0,
                        display.position.1
                    )
                );
            }
        }

        Ok(config)
    }
}

impl Default for WindowSystemManager {
    fn default() -> Self {
        Self::new()
    }
}
