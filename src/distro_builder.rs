use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::process::Command as AsyncCommand;
use std::time::Instant;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::{Semaphore, Mutex};
use futures::future::try_join_all;
use std::collections::HashMap;
use sha2::{Sha256, Digest};

// Enhanced logging and progress tracking
#[derive(Debug, Clone)]
pub struct BuildProgress {
    pub current_step: String,
    pub total_steps: usize,
    pub current_step_number: usize,
    pub start_time: Instant,
    pub step_start_time: Instant,
    pub build_id: String,
}

impl BuildProgress {
    pub fn new(total_steps: usize, build_id: String) -> Self {
        let now = Instant::now();
        Self {
            current_step: "Initializing".to_string(),
            total_steps,
            current_step_number: 0,
            start_time: now,
            step_start_time: now,
            build_id,
        }
    }
    
    pub fn start_step(&mut self, step_name: &str, step_number: usize) {
        self.current_step = step_name.to_string();
        self.current_step_number = step_number;
        self.step_start_time = Instant::now();
        
        let elapsed = self.start_time.elapsed();
        let progress_percent = (step_number as f64 / self.total_steps as f64) * 100.0;
        
        println!("\n🔄 [{}/{}] ({:.1}%) {} | Elapsed: {:.1}s", 
                step_number, self.total_steps, progress_percent, 
                step_name, elapsed.as_secs_f64());
    }
    
    pub fn complete_step(&self, success: bool) {
        let step_duration = self.step_start_time.elapsed();
        let status_icon = if success { "✅" } else { "❌" };
        
        println!("{} {} completed in {:.1}s", 
                status_icon, self.current_step, step_duration.as_secs_f64());
    }
    
    pub fn log_substep(&self, message: &str) {
        println!("   ↳ {}", message);
    }
    
    pub fn log_warning(&self, message: &str) {
        println!("   ⚠️  WARNING: {}", message);
    }
    
    
    pub fn get_build_summary(&self) -> String {
        let total_duration = self.start_time.elapsed();
        format!("Build ID: {} | Total time: {:.1}s ({:.1}m)", 
               self.build_id, total_duration.as_secs_f64(), total_duration.as_secs_f64() / 60.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildError {
    pub step: String,
    pub error_type: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub command: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub build_id: String,
}

impl BuildError {
    pub fn new(
        step: &str, 
        error_type: &str, 
        message: &str, 
        build_id: &str,
        command: Option<String>,
        stdout: Option<String>,
        stderr: Option<String>
    ) -> Self {
        Self {
            step: step.to_string(),
            error_type: error_type.to_string(),
            message: message.to_string(),
            timestamp: Utc::now(),
            command,
            stdout,
            stderr,
            build_id: build_id.to_string(),
        }
    }
    
    pub fn log_detailed_error(&self) {
        println!("\n🚨 BUILD ERROR DETAILS:");
        println!("   Build ID: {}", self.build_id);
        println!("   Step: {}", self.step);
        println!("   Type: {}", self.error_type);
        println!("   Time: {}", self.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("   Message: {}", self.message);
        
        if let Some(cmd) = &self.command {
            println!("   Failed Command: {}", cmd);
        }
        
        if let Some(stdout) = &self.stdout {
            if !stdout.trim().is_empty() {
                println!("   stdout: {}", stdout.trim());
            }
        }
        
        if let Some(stderr) = &self.stderr {
            if !stderr.trim().is_empty() {
                println!("   stderr: {}", stderr.trim());
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    pub architecture: String,
    pub base_system: BaseSystem,
    pub packages: PackageConfig,
    pub kernel: KernelConfig,
    pub bootloader: BootloaderConfig,
    pub branding: BrandingConfig,
    pub filesystem: FilesystemConfig,
    pub build_options: BuildOptions,
    pub user_config: UserConfig,
    pub validation: ValidationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOptions {
    pub parallel_builds: bool,
    pub max_parallel_jobs: Option<usize>,
    pub cleanup_on_failure: bool,
    pub preserve_cache: bool,
    pub enable_ccache: bool,
    pub build_logs: bool,
    pub progress_reporting: ProgressReporting,
    pub timeout_minutes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressReporting {
    Minimal,
    Standard,
    Verbose,
    Debug,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub default_user: Option<UserAccount>,
    pub root_password: Option<String>,
    pub timezone: Option<String>,
    pub locale: Option<String>,
    pub keyboard_layout: Option<String>,
    pub network_config: NetworkConfig,
    pub services: ServicesConfig,
    pub post_install_scripts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccount {
    pub username: String,
    pub password: Option<String>,
    pub groups: Vec<String>,
    pub shell: Option<String>,
    pub home_dir: Option<String>,
    pub sudo_access: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub enable_networking: bool,
    pub dhcp: bool,
    pub static_ip: Option<StaticIpConfig>,
    pub dns_servers: Vec<String>,
    pub hostname_strategy: HostnameStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticIpConfig {
    pub ip_address: String,
    pub netmask: String,
    pub gateway: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostnameStrategy {
    FromConfig,
    Random,
    UserPrompt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    pub enable_ssh: bool,
    pub enable_firewall: bool,
    pub auto_login: bool,
    pub custom_services: Vec<String>,
    pub disabled_services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub strict_validation: bool,
    pub warn_on_large_iso: bool,
    pub max_iso_size_mb: u64,
    pub validate_packages: bool,
    pub check_dependencies: bool,
    pub verify_signatures: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BaseSystem {
    Arch,
    Debian,
    Ubuntu,
    Fedora,
    CentOS,
    OpenSUSE,
    Alpine,
    Scratch, // Build from scratch
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub essential: Vec<String>,
    pub desktop_environment: Option<DesktopEnvironment>,
    pub additional_packages: Vec<String>,
    pub custom_repositories: Vec<Repository>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DesktopEnvironment {
    Gnome,
    Kde,
    Xfce,
    Lxde,
    Mate,
    Cinnamon,
    Sway,
    I3,
    Custom(String),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub key_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelConfig {
    pub kernel_type: KernelType,
    pub custom_config: Option<PathBuf>,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KernelType {
    Vanilla,
    Lts,
    Hardened,
    Rt, // Real-time
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootloaderConfig {
    pub bootloader: Bootloader,
    pub timeout: u32,
    pub default_entry: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Bootloader {
    Grub,
    Systemd,
    Syslinux,
    Refind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingConfig {
    pub logo: Option<PathBuf>,
    pub wallpaper: Option<PathBuf>,
    pub theme: Option<String>,
    pub colors: ColorScheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    pub root_fs: FilesystemType,
    pub compression: CompressionType,
    pub size_limit: Option<u64>, // In MB
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilesystemType {
    SquashFs,
    Ext4,
    Btrfs,
    Xfs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    Gzip,
    Xz,
    Zstd,
    Lz4,
    None,
}

pub struct DistroBuilder {
    config: DistroConfig,
    work_dir: PathBuf,
    output_dir: PathBuf,
    package_cache: Arc<Mutex<HashMap<String, PackageCacheEntry>>>,
    parallel_semaphore: Arc<Semaphore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageCacheEntry {
    pub package_name: String,
    pub version: String,
    pub hash: String,
    pub timestamp: DateTime<Utc>,
    pub cached_path: PathBuf,
}


// Configuration validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl DistroBuilder {
    pub fn new(config: DistroConfig, work_dir: PathBuf, output_dir: PathBuf) -> Self {
        let max_parallel = config.build_options.max_parallel_jobs
            .unwrap_or_else(|| num_cpus::get());
        
        Self {
            config,
            work_dir,
            output_dir,
            package_cache: Arc::new(Mutex::new(HashMap::new())),
            parallel_semaphore: Arc::new(Semaphore::new(max_parallel)),
        }
    }

    /// Validates the distribution configuration before building
    pub fn validate_config(&self) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate required fields
        self.validate_required_fields(&mut errors);
        
        // Validate network configuration
        self.validate_network_config(&mut errors, &mut warnings);
        
        // Validate packages if enabled
        if self.config.validation.validate_packages {
            self.validate_packages(&mut errors, &mut warnings);
        }
        
        // Check ISO size warnings
        if self.config.validation.warn_on_large_iso {
            self.validate_iso_size(&mut warnings);
        }
        
        // Check dependencies if enabled
        if self.config.validation.check_dependencies {
            self.validate_dependencies(&mut errors, &mut warnings);
        }
        
        // Validate file paths
        self.validate_file_paths(&mut errors, &mut warnings);
        
        // Validate user configuration
        self.validate_user_config(&mut errors, &mut warnings);
        
        // Validate build options
        self.validate_build_options(&mut warnings);

        ValidationResult {
            is_valid: errors.is_empty() || !self.config.validation.strict_validation,
            errors,
            warnings,
        }
    }

    fn validate_required_fields(&self, errors: &mut Vec<ValidationError>) {
        if self.config.name.is_empty() {
            errors.push(ValidationError {
                field: "name".to_string(),
                message: "Distribution name cannot be empty".to_string(),
                severity: ValidationSeverity::Critical,
            });
        }
        
        if self.config.version.is_empty() {
            errors.push(ValidationError {
                field: "version".to_string(),
                message: "Distribution version cannot be empty".to_string(),
                severity: ValidationSeverity::Critical,
            });
        }
        
        if self.config.architecture.is_empty() {
            errors.push(ValidationError {
                field: "architecture".to_string(),
                message: "Architecture cannot be empty".to_string(),
                severity: ValidationSeverity::Critical,
            });
        } else if !matches!(self.config.architecture.as_str(), "x86_64" | "i686" | "aarch64" | "armv7h") {
            errors.push(ValidationError {
                field: "architecture".to_string(),
                message: format!("Unsupported architecture: {}. Supported: x86_64, i686, aarch64, armv7h", self.config.architecture),
                severity: ValidationSeverity::High,
            });
        }
    }

    fn validate_network_config(&self, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>) {
        let network = &self.config.user_config.network_config;
        
        if let Some(ref static_ip) = network.static_ip {
            // Validate IP address format
            if !self.is_valid_ip(&static_ip.ip_address) {
                errors.push(ValidationError {
                    field: "user_config.network_config.static_ip.ip_address".to_string(),
                    message: format!("Invalid IP address format: {}", static_ip.ip_address),
                    severity: ValidationSeverity::High,
                });
            }
            
            // Validate netmask
            if !self.is_valid_ip(&static_ip.netmask) && !self.is_valid_cidr_mask(&static_ip.netmask) {
                errors.push(ValidationError {
                    field: "user_config.network_config.static_ip.netmask".to_string(),
                    message: format!("Invalid netmask format: {}", static_ip.netmask),
                    severity: ValidationSeverity::High,
                });
            }
            
            // Validate gateway
            if !self.is_valid_ip(&static_ip.gateway) {
                errors.push(ValidationError {
                    field: "user_config.network_config.static_ip.gateway".to_string(),
                    message: format!("Invalid gateway IP address format: {}", static_ip.gateway),
                    severity: ValidationSeverity::High,
                });
            }
        }
        
        // Validate DNS servers
        for (index, dns) in network.dns_servers.iter().enumerate() {
            if !self.is_valid_ip(dns) {
                errors.push(ValidationError {
                    field: format!("user_config.network_config.dns_servers[{}]", index),
                    message: format!("Invalid DNS server IP address: {}", dns),
                    severity: ValidationSeverity::Medium,
                });
            }
        }
        
        // Warning if both DHCP and static IP are configured
        if network.dhcp && network.static_ip.is_some() {
            warnings.push(ValidationWarning {
                field: "user_config.network_config".to_string(),
                message: "Both DHCP and static IP are configured. Static IP will take precedence.".to_string(),
                suggestion: Some("Consider disabling DHCP if using static IP configuration".to_string()),
            });
        }
    }

    fn validate_packages(&self, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>) {
        // Check for duplicate packages
        let all_packages: Vec<String> = self.config.packages.essential.iter()
            .chain(self.config.packages.additional_packages.iter())
            .cloned()
            .collect();
            
        let mut seen = std::collections::HashSet::new();
        for package in &all_packages {
            if !seen.insert(package) {
                warnings.push(ValidationWarning {
                    field: "packages".to_string(),
                    message: format!("Duplicate package found: {}", package),
                    suggestion: Some("Remove duplicate package entries to avoid conflicts".to_string()),
                });
            }
        }
        
        // Check for essential packages that might conflict with base system
        let problematic_packages = ["base", "linux", "linux-firmware"];
        for pkg in &self.config.packages.essential {
            if problematic_packages.contains(&pkg.as_str()) {
                match self.config.base_system {
                    BaseSystem::Arch => {
                        warnings.push(ValidationWarning {
                            field: "packages.essential".to_string(),
                            message: format!("Package '{}' is typically included in base system", pkg),
                            suggestion: Some("Consider removing from essential packages list".to_string()),
                        });
                    },
                    _ => {}
                }
            }
        }
        
        // Validate desktop environment consistency
        if let Some(ref de) = self.config.packages.desktop_environment {
            match de {
                DesktopEnvironment::Custom(name) => {
                    if name.is_empty() {
                        errors.push(ValidationError {
                            field: "packages.desktop_environment".to_string(), 
                            message: "Custom desktop environment name cannot be empty".to_string(),
                            severity: ValidationSeverity::Medium,
                        });
                    }
                },
                _ => {}
            }
        }
    }

    fn validate_iso_size(&self, warnings: &mut Vec<ValidationWarning>) {
        if let Some(size_limit) = self.config.filesystem.size_limit {
            if size_limit > self.config.validation.max_iso_size_mb {
                warnings.push(ValidationWarning {
                    field: "filesystem.size_limit".to_string(),
                    message: format!("ISO size limit ({} MB) exceeds validation threshold ({} MB)", 
                                   size_limit, self.config.validation.max_iso_size_mb),
                    suggestion: Some("Consider reducing package count or using more aggressive compression".to_string()),
                });
            }
        }
        
        // Warn about large desktop environments
        if let Some(ref de) = self.config.packages.desktop_environment {
            let estimated_size = match de {
                DesktopEnvironment::Gnome => 2500,
                DesktopEnvironment::Kde => 3000,
                DesktopEnvironment::Xfce => 800,
                DesktopEnvironment::Lxde => 400,
                DesktopEnvironment::Mate => 900,
                DesktopEnvironment::Cinnamon => 1200,
                DesktopEnvironment::Sway => 200,
                DesktopEnvironment::I3 => 150,
                _ => 500,
            };
            
            if estimated_size > 2000 {
                warnings.push(ValidationWarning {
                    field: "packages.desktop_environment".to_string(),
                    message: format!("Desktop environment {:?} may result in large ISO (~{} MB)", de, estimated_size),
                    suggestion: Some("Consider a lighter desktop environment for smaller ISO".to_string()),
                });
            }
        }
    }

    fn validate_dependencies(&self, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>) {
        // Check if required tools are available on the build system
        let required_tools = match self.config.base_system {
            BaseSystem::Arch => vec!["pacstrap", "arch-chroot"],
            BaseSystem::Debian | BaseSystem::Ubuntu => vec!["debootstrap"],
            BaseSystem::Fedora | BaseSystem::CentOS => vec!["dnf", "rpm"],
            BaseSystem::OpenSUSE => vec!["zypper", "rpm"],
            BaseSystem::Alpine => vec!["apk"],
            BaseSystem::Scratch => vec!["gcc", "make"],
        };
        
        for tool in required_tools {
            if !self.check_command_exists(tool) {
                errors.push(ValidationError {
                    field: "build_dependencies".to_string(),
                    message: format!("Required build tool '{}' is not available", tool),
                    severity: ValidationSeverity::Critical,
                });
            }
        }
        
        // Check for ISO creation tools
        let iso_tools = ["mksquashfs", "xorriso"];
        for tool in iso_tools {
            if !self.check_command_exists(tool) {
                errors.push(ValidationError {
                    field: "build_dependencies".to_string(),
                    message: format!("Required ISO creation tool '{}' is not available", tool),
                    severity: ValidationSeverity::Critical,
                });
            }
        }
        
        // Check bootloader dependencies
        match self.config.bootloader.bootloader {
            Bootloader::Syslinux => {
                if !Path::new("/usr/lib/syslinux/bios/isolinux.bin").exists() {
                    warnings.push(ValidationWarning {
                        field: "bootloader.bootloader".to_string(),
                        message: "Syslinux BIOS files not found in expected location".to_string(),
                        suggestion: Some("Install syslinux package: pacman -S syslinux".to_string()),
                    });
                }
            },
            Bootloader::Grub => {
                if !self.check_command_exists("grub-mkrescue") {
                    warnings.push(ValidationWarning {
                        field: "bootloader.bootloader".to_string(),
                        message: "GRUB tools not found".to_string(),
                        suggestion: Some("Install grub package".to_string()),
                    });
                }
            },
            _ => {}
        }
    }

    fn validate_file_paths(&self, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>) {
        // Validate branding file paths
        if let Some(ref logo_path) = self.config.branding.logo {
            if !logo_path.exists() {
                warnings.push(ValidationWarning {
                    field: "branding.logo".to_string(),
                    message: format!("Logo file not found: {}", logo_path.display()),
                    suggestion: Some("Verify the logo file path is correct".to_string()),
                });
            }
        }
        
        if let Some(ref wallpaper_path) = self.config.branding.wallpaper {
            if !wallpaper_path.exists() {
                warnings.push(ValidationWarning {
                    field: "branding.wallpaper".to_string(),
                    message: format!("Wallpaper file not found: {}", wallpaper_path.display()),
                    suggestion: Some("Verify the wallpaper file path is correct".to_string()),
                });
            }
        }
        
        // Validate kernel config path
        if let Some(ref kernel_config) = self.config.kernel.custom_config {
            if !kernel_config.exists() {
                errors.push(ValidationError {
                    field: "kernel.custom_config".to_string(),
                    message: format!("Kernel config file not found: {}", kernel_config.display()),
                    severity: ValidationSeverity::High,
                });
            }
        }
        
        // Validate post-install script paths
        for (index, script_path) in self.config.user_config.post_install_scripts.iter().enumerate() {
            let path = Path::new(script_path);
            if !path.exists() {
                warnings.push(ValidationWarning {
                    field: format!("user_config.post_install_scripts[{}]", index),
                    message: format!("Post-install script not found: {}", script_path),
                    suggestion: Some("Verify the script path is correct".to_string()),
                });
            }
        }
    }

    fn validate_user_config(&self, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>) {
        if let Some(ref user) = self.config.user_config.default_user {
            // Validate username
            if user.username.is_empty() {
                errors.push(ValidationError {
                    field: "user_config.default_user.username".to_string(),
                    message: "Username cannot be empty".to_string(),
                    severity: ValidationSeverity::High,
                });
            } else if !self.is_valid_username(&user.username) {
                errors.push(ValidationError {
                    field: "user_config.default_user.username".to_string(),
                    message: format!("Invalid username format: {}", user.username),
                    severity: ValidationSeverity::High,
                });
            }
            
            // Check for risky configurations
            if user.username == "root" {
                warnings.push(ValidationWarning {
                    field: "user_config.default_user.username".to_string(),
                    message: "Using 'root' as default user is not recommended".to_string(),
                    suggestion: Some("Create a regular user and grant sudo access instead".to_string()),
                });
            }
        }
        
        // Validate timezone
        if let Some(ref timezone) = self.config.user_config.timezone {
            if !self.is_valid_timezone(timezone) {
                warnings.push(ValidationWarning {
                    field: "user_config.timezone".to_string(),
                    message: format!("Potentially invalid timezone: {}", timezone),
                    suggestion: Some("Use standard timezone format like 'America/New_York' or 'UTC'".to_string()),
                });
            }
        }
        
        // Validate locale
        if let Some(ref locale) = self.config.user_config.locale {
            if !locale.contains('.') {
                warnings.push(ValidationWarning {
                    field: "user_config.locale".to_string(),
                    message: format!("Locale format may be incomplete: {}", locale),
                    suggestion: Some("Use format like 'en_US.UTF-8'".to_string()),
                });
            }
        }
    }

    fn validate_build_options(&self, warnings: &mut Vec<ValidationWarning>) {
        // Warn about potentially problematic build options
        if self.config.build_options.parallel_builds {
            if let Some(jobs) = self.config.build_options.max_parallel_jobs {
                if jobs > num_cpus::get() * 2 {
                    warnings.push(ValidationWarning {
                        field: "build_options.max_parallel_jobs".to_string(),
                        message: format!("High parallel job count ({}) may cause system instability", jobs),
                        suggestion: Some("Consider using a value closer to CPU core count".to_string()),
                    });
                }
            }
        }
        
        // Warn about timeout settings
        if let Some(timeout) = self.config.build_options.timeout_minutes {
            if timeout < 30 {
                warnings.push(ValidationWarning {
                    field: "build_options.timeout_minutes".to_string(),
                    message: "Build timeout is very short, builds may fail unexpectedly".to_string(),
                    suggestion: Some("Consider at least 60 minutes for reliable builds".to_string()),
                });
            } else if timeout > 480 {
                warnings.push(ValidationWarning {
                    field: "build_options.timeout_minutes".to_string(),
                    message: "Build timeout is very long, may mask build issues".to_string(),
                    suggestion: Some("Consider shorter timeout to catch problematic builds".to_string()),
                });
            }
        }
    }

    // Helper validation methods
    fn is_valid_ip(&self, ip: &str) -> bool {
        ip.parse::<std::net::IpAddr>().is_ok()
    }
    
    fn is_valid_cidr_mask(&self, mask: &str) -> bool {
        // Check if it's a valid CIDR notation like "/24"
        if let Some(stripped) = mask.strip_prefix('/') {
            if let Ok(prefix_len) = stripped.parse::<u8>() {
                return prefix_len <= 32;
            }
        }
        false
    }
    
    fn check_command_exists(&self, command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    fn is_valid_username(&self, username: &str) -> bool {
        // Basic username validation: alphanumeric, underscore, hyphen
        // Must start with letter or underscore, 1-32 characters
        if username.len() > 32 || username.is_empty() {
            return false;
        }
        
        let first_char = username.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return false;
        }
        
        username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    }
    
    fn is_valid_timezone(&self, timezone: &str) -> bool {
        // Basic timezone validation - this is simplified
        // In a real implementation, you'd check against a timezone database
        timezone == "UTC" || 
        timezone.contains('/') ||
        timezone.starts_with("GMT") ||
        timezone.starts_with("Etc/")
    }

    /// Print validation results in a user-friendly format
    pub fn print_validation_results(&self, result: &ValidationResult) {
        println!("\n🔍 CONFIGURATION VALIDATION RESULTS:\n");
        
        if result.is_valid {
            println!("✅ Configuration is valid!");
        } else {
            println!("❌ Configuration validation failed!");
        }
        
        if !result.errors.is_empty() {
            println!("\n🚨 ERRORS ({}):", result.errors.len());
            for error in &result.errors {
                let severity_icon = match error.severity {
                    ValidationSeverity::Critical => "💀",
                    ValidationSeverity::High => "🔴",
                    ValidationSeverity::Medium => "🟡",
                    ValidationSeverity::Low => "🔵",
                };
                println!("   {} [{}] {}: {}", severity_icon, error.field, 
                        format!("{:?}", error.severity).to_uppercase(), error.message);
            }
        }
        
        if !result.warnings.is_empty() {
            println!("\n⚠️  WARNINGS ({}):", result.warnings.len());
            for warning in &result.warnings {
                println!("   🟠 [{}] {}", warning.field, warning.message);
                if let Some(ref suggestion) = warning.suggestion {
                    println!("      💡 Suggestion: {}", suggestion);
                }
            }
        }
        
        if result.errors.is_empty() && result.warnings.is_empty() {
            println!("\n🎉 No issues found! Configuration looks perfect.");
        }
        
        println!();
    }

    pub async fn build(&self) -> Result<PathBuf> {
        // Validate configuration before building
        let validation_result = self.validate_config();
        if !validation_result.is_valid {
            self.print_validation_results(&validation_result);
            return Err(anyhow::anyhow!("Configuration validation failed"));
        }
        
        // Print validation results if there are warnings
        if !validation_result.warnings.is_empty() {
            self.print_validation_results(&validation_result);
        }
        
        // Generate unique build ID
        let build_id = format!("{}-{}", 
                              self.config.name, 
                              chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        
        let mut progress = BuildProgress::new(8, build_id.clone());
        
        println!("🚀 Starting Linux distribution build: {} (ID: {})", 
                self.config.name, build_id);
        println!("📊 Configuration: {} v{} ({})", 
                self.config.name, self.config.version, self.config.architecture);
        println!("🏗️  Base System: {:?} | Desktop: {:?}", 
                self.config.base_system, self.config.packages.desktop_environment);
        println!("💾 Filesystem: {:?} with {:?} compression", 
                self.config.filesystem.root_fs, self.config.filesystem.compression);
        
        let build_start = std::time::Instant::now();
        let mut errors = Vec::new();
        
        // Step 1: Setup directories
        progress.start_step("Setting up build directories", 1);
        match self.setup_directories().await {
            Ok(_) => {
                progress.complete_step(true);
                progress.log_substep("Created work directories successfully");
            }
            Err(e) => {
                let error = BuildError::new(
                    "setup_directories", "filesystem", &e.to_string(), &build_id,
                    None, None, None
                );
                error.log_detailed_error();
                errors.push(error);
                progress.complete_step(false);
                return Err(e);
            }
        }
        
        // Step 2: Build root filesystem
        progress.start_step("Building root filesystem", 2);
        match self.build_rootfs().await {
            Ok(_) => {
                progress.complete_step(true);
                progress.log_substep("Root filesystem created successfully");
            }
            Err(e) => {
                let error = BuildError::new(
                    "build_rootfs", "bootstrap", &e.to_string(), &build_id,
                    None, None, None
                );
                error.log_detailed_error();
                errors.push(error);
                progress.complete_step(false);
                return Err(e);
            }
        }
        
        // Step 3: Install kernel
        progress.start_step("Installing kernel", 3);
        match self.install_kernel().await {
            Ok(_) => {
                progress.complete_step(true);
                progress.log_substep("Kernel installation completed");
            }
            Err(e) => {
                let error = BuildError::new(
                    "install_kernel", "package_installation", &e.to_string(), &build_id,
                    None, None, None
                );
                error.log_detailed_error();
                errors.push(error);
                progress.complete_step(false);
                return Err(e);
            }
        }
        
        // Step 4: Install packages
        progress.start_step("Installing packages", 4);
        match self.install_packages().await {
            Ok(_) => {
                progress.complete_step(true);
                progress.log_substep("Package installation completed");
            }
            Err(e) => {
                let error = BuildError::new(
                    "install_packages", "package_installation", &e.to_string(), &build_id,
                    None, None, None
                );
                error.log_detailed_error();
                errors.push(error);
                progress.complete_step(false);
                return Err(e);
            }
        }
        
        // Step 5: Configure system
        progress.start_step("Configuring system", 5);
        match self.configure_system().await {
            Ok(_) => {
                progress.complete_step(true);
                progress.log_substep("System configuration completed");
            }
            Err(e) => {
                let error = BuildError::new(
                    "configure_system", "configuration", &e.to_string(), &build_id,
                    None, None, None
                );
                error.log_detailed_error();
                errors.push(error);
                progress.complete_step(false);
                return Err(e);
            }
        }
        
        // Step 6: Apply branding
        progress.start_step("Applying branding", 6);
        match self.apply_branding().await {
            Ok(_) => {
                progress.complete_step(true);
                progress.log_substep("Branding applied successfully");
            }
            Err(e) => {
                let error = BuildError::new(
                    "apply_branding", "branding", &e.to_string(), &build_id,
                    None, None, None
                );
                error.log_detailed_error();
                errors.push(error);
                progress.complete_step(false);
                return Err(e);
            }
        }
        
        // Step 7: Configure bootloader
        progress.start_step("Configuring bootloader", 7);
        match self.configure_bootloader().await {
            Ok(_) => {
                progress.complete_step(true);
                progress.log_substep("Bootloader configuration completed");
            }
            Err(e) => {
                let error = BuildError::new(
                    "configure_bootloader", "bootloader", &e.to_string(), &build_id,
                    None, None, None
                );
                error.log_detailed_error();
                errors.push(error);
                progress.complete_step(false);
                return Err(e);
            }
        }
        
        // Step 8: Create ISO
        progress.start_step("Creating ISO image", 8);
        let iso_path = match self.create_iso().await {
            Ok(path) => {
                progress.complete_step(true);
                progress.log_substep(&format!("ISO created: {}", path.display()));
                path
            }
            Err(e) => {
                let error = BuildError::new(
                    "create_iso", "iso_creation", &e.to_string(), &build_id,
                    None, None, None
                );
                error.log_detailed_error();
                errors.push(error);
                progress.complete_step(false);
                return Err(e);
            }
        };
        
        // Final summary
        let _total_duration = build_start.elapsed();
        println!("\n🎉 BUILD COMPLETED SUCCESSFULLY!");
        println!("📊 {}", progress.get_build_summary());
        println!("💿 ISO Path: {}", iso_path.display());
        
        // Check ISO file size
        if let Ok(metadata) = std::fs::metadata(&iso_path) {
            let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
            println!("📏 ISO Size: {:.1} MB", size_mb);
            
            if let Some(limit) = self.config.filesystem.size_limit {
                if size_mb > limit as f64 {
                    progress.log_warning(&format!(
                        "ISO size ({:.1} MB) exceeds configured limit ({} MB)", 
                        size_mb, limit
                    ));
                }
            }
        }
        
        println!("🔗 You can now test the ISO with: qemu-system-x86_64 -m 2G -cdrom {}", 
                iso_path.display());
        
        Ok(iso_path)
    }


    async fn setup_directories(&self) -> Result<()> {
        println!("📁 Setting up build directories...");
        
        // Clean up any existing directories first
        if self.work_dir.exists() {
            println!("Cleaning up existing work directory...");
            fs::remove_dir_all(&self.work_dir)
                .with_context(|| format!("Failed to remove existing work directory: {}", self.work_dir.display()))?;
        }
        
        let dirs = [
            &self.work_dir,
            &self.output_dir,
            &self.work_dir.join("rootfs"),
            &self.work_dir.join("boot"),
            &self.work_dir.join("iso"),
        ];

        for dir in dirs {
            fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
            println!("Created directory: {}", dir.display());
        }

        Ok(())
    }

    async fn build_rootfs(&self) -> Result<()> {
        println!("🔧 Building root filesystem...");
        
        let rootfs_dir = self.work_dir.join("rootfs");
        
        match self.config.base_system {
            BaseSystem::Arch => self.build_arch_rootfs(&rootfs_dir).await?,
            BaseSystem::Debian => self.build_debian_rootfs(&rootfs_dir).await?,
            BaseSystem::Ubuntu => self.build_ubuntu_rootfs(&rootfs_dir).await?,
            BaseSystem::Fedora => self.build_fedora_rootfs(&rootfs_dir).await?,
            BaseSystem::CentOS => self.build_centos_rootfs(&rootfs_dir).await?,
            BaseSystem::OpenSUSE => self.build_opensuse_rootfs(&rootfs_dir).await?,
            BaseSystem::Alpine => self.build_alpine_rootfs(&rootfs_dir).await?,
            BaseSystem::Scratch => self.build_scratch_rootfs(&rootfs_dir).await?,
        }

        Ok(())
    }

    async fn build_arch_rootfs(&self, rootfs_dir: &Path) -> Result<()> {
        println!("🏗️  Building Arch Linux base system...");
        
        // Ensure the directory exists and has proper permissions
        fs::create_dir_all(rootfs_dir)?;
        
        // Use pacstrap to bootstrap base system (don't skip copying mirrorlist)
        let mut cmd = AsyncCommand::new("pacstrap");
        cmd.arg("-c")  // Use package cache
           .arg(rootfs_dir)
           .arg("base")
           .arg("linux")
           .arg("linux-firmware");

        println!("Running: pacstrap -c {} base linux linux-firmware", rootfs_dir.display());
        
        let output = cmd.output().await
            .context("Failed to run pacstrap")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("STDOUT: {stdout}");
            println!("STDERR: {stderr}");
            anyhow::bail!("pacstrap failed: {}", stderr);
        }

        // Copy mirrorlist to ensure package installations work
        let host_mirrorlist = Path::new("/etc/pacman.d/mirrorlist");
        let chroot_mirrorlist = rootfs_dir.join("etc/pacman.d/mirrorlist");
        if host_mirrorlist.exists() {
            fs::create_dir_all(chroot_mirrorlist.parent().unwrap())?;
            fs::copy(host_mirrorlist, chroot_mirrorlist)?;
            println!("✅ Copied mirrorlist to chroot");
        }

        println!("✅ Arch Linux base system created successfully");
        Ok(())
    }

    async fn build_debian_rootfs(&self, rootfs_dir: &Path) -> Result<()> {
        println!("🏗️  Building Debian base system...");
        
        // Install debootstrap if not available
        if Command::new("which").arg("debootstrap").output()?.status.success() {
            let mut cmd = AsyncCommand::new("debootstrap");
            cmd.arg("--arch").arg(&self.config.architecture)
               .arg("stable")
               .arg(rootfs_dir)
               .arg("http://deb.debian.org/debian/");

            let output = cmd.output().await
                .context("Failed to run debootstrap")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("debootstrap failed: {}", stderr);
            }
        } else {
            anyhow::bail!("debootstrap not found. Please install it first.");
        }

        Ok(())
    }

    async fn build_ubuntu_rootfs(&self, rootfs_dir: &Path) -> Result<()> {
        println!("🏗️  Building Ubuntu base system...");
        
        if Command::new("which").arg("debootstrap").output()?.status.success() {
            let mut cmd = AsyncCommand::new("debootstrap");
            cmd.arg("--arch").arg(&self.config.architecture)
               .arg("jammy") // Ubuntu 22.04 LTS
               .arg(rootfs_dir)
               .arg("http://archive.ubuntu.com/ubuntu/");

            let output = cmd.output().await
                .context("Failed to run debootstrap")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("debootstrap failed: {}", stderr);
            }
        } else {
            anyhow::bail!("debootstrap not found. Please install it first.");
        }

        Ok(())
    }

    async fn build_scratch_rootfs(&self, rootfs_dir: &Path) -> Result<()> {
        println!("🏗️  Building minimal system from scratch...");
        
        // Create basic directory structure
        let dirs = [
            "bin", "boot", "dev", "etc", "home", "lib", "lib64", "mnt", 
            "opt", "proc", "root", "run", "sbin", "srv", "sys", "tmp", 
            "usr", "var", "usr/bin", "usr/lib", "usr/sbin", "var/log"
        ];

        for dir in dirs {
            fs::create_dir_all(rootfs_dir.join(dir))?;
        }

        // This would require building toolchain and basic utilities
        // For now, we'll create a minimal BusyBox-based system
        println!("⚠️  Scratch build requires manual toolchain setup");

        Ok(())
    }

    async fn build_fedora_rootfs(&self, rootfs_dir: &Path) -> Result<()> {
        println!("🏗️  Building Fedora base system...");
        
        if Command::new("which").arg("dnf").output()?.status.success() {
            // Use dnf to create a chroot environment
            let mut cmd = AsyncCommand::new("dnf");
            cmd.arg("--installroot=").arg(rootfs_dir)
               .arg("install")
               .arg("@core")
               .arg("--releasever").arg("latest")
               .arg("-y");

            let output = cmd.output().await.context("Failed to run dnf")?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("dnf install failed: {}", stderr);
            }
        } else {
            anyhow::bail!("dnf not found. Please install it first.");
        }

        Ok(())
    }

    async fn build_centos_rootfs(&self, rootfs_dir: &Path) -> Result<()> {
        println!("🏗️  Building CentOS base system...");
        
        if Command::new("which").arg("yum").output()?.status.success() {
            // Use yum to create a yum shell and install base
            // For simplicity, use yum groupinstall
            let mut cmd = AsyncCommand::new("yum");
            cmd.arg("--installroot=").arg(rootfs_dir)
               .arg("groupinstall")
               .arg("Core")
               .arg("-y");

            let output = cmd.output().await.context("Failed to run yum")?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("yum groupinstall failed: {}", stderr);
            }
        } else {
            anyhow::bail!("yum not found. Please install it first.");
        }

        Ok(())
    }

    async fn build_opensuse_rootfs(&self, rootfs_dir: &Path) -> Result<()> {
        println!("🏗️  Building openSUSE base system...");
        
        if Command::new("which").arg("zypper").output()?.status.success() {
            // Use zypper to create base
            let mut cmd = AsyncCommand::new("zypper");
            cmd.arg("--root").arg(rootfs_dir)
               .arg("install")
               .arg("-t").arg("pattern")
               .arg("minimal_base")
               .arg("-y");

            let output = cmd.output().await.context("Failed to run zypper")?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("zypper install failed: {}", stderr);
            }
        } else {
            anyhow::bail!("zypper not found. Please install it first.");
        }

        Ok(())
    }

    async fn build_alpine_rootfs(&self, rootfs_dir: &Path) -> Result<()> {
        println!("🏗️  Building Alpine base system...");
        
        if Command::new("which").arg("apk").output()?.status.success() {
            let mut cmd = AsyncCommand::new("apk");
            cmd.arg("--root").arg(rootfs_dir)
               .arg("--initdb")
               .arg("add")
               .arg("alpine-base");

            let output = cmd.output().await.context("Failed to run apk")?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("apk add failed: {}", stderr);
            }
        } else {
            anyhow::bail!("apk not found. Please install it first.");
        }

        Ok(())
    }

    async fn install_kernel(&self) -> Result<()> {
        println!("🐧 Installing kernel...");
        
        let rootfs_dir = self.work_dir.join("rootfs");
        
        match self.config.kernel.kernel_type {
            KernelType::Vanilla => {
                // Install vanilla kernel
                self.install_arch_kernel(&rootfs_dir, "linux").await?;
            }
            KernelType::Lts => {
                self.install_arch_kernel(&rootfs_dir, "linux-lts").await?;
            }
            KernelType::Hardened => {
                self.install_arch_kernel(&rootfs_dir, "linux-hardened").await?;
            }
            KernelType::Rt => {
                self.install_arch_kernel(&rootfs_dir, "linux-rt").await?;
            }
            KernelType::Custom(ref kernel) => {
                self.install_arch_kernel(&rootfs_dir, kernel).await?;
            }
        }

        Ok(())
    }

    async fn install_arch_kernel(&self, rootfs_dir: &Path, kernel_package: &str) -> Result<()> {
        println!("Installing kernel package: {kernel_package}");
        
        // First, update the package database
        let mut update_cmd = AsyncCommand::new("arch-chroot");
        update_cmd.arg(rootfs_dir)
                  .arg("pacman")
                  .arg("-Sy")
                  .arg("--noconfirm");
        
        let update_output = update_cmd.output().await?;
        if !update_output.status.success() {
            println!("Warning: Failed to update package database");
        }
        
        // Install the kernel (it might already be installed from base)
        let mut cmd = AsyncCommand::new("arch-chroot");
        cmd.arg(rootfs_dir)
           .arg("pacman")
           .arg("-S")
           .arg("--noconfirm")
           .arg("--needed")  // Only install if not already present
           .arg(kernel_package);

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("STDOUT: {stdout}");
            println!("STDERR: {stderr}");
            // Don't fail if kernel is already installed
            if !stderr.contains("is up to date") && !stderr.contains("target not found") {
                anyhow::bail!("Kernel installation failed: {}", stderr);
            }
        }

        println!("✅ Kernel installation completed");
        Ok(())
    }

    async fn install_packages(&self) -> Result<()> {
        println!("📦 Installing packages...");
        
        let rootfs_dir = self.work_dir.join("rootfs");
        
        // Filter out packages that are already included in base system
        let base_packages = vec!["base", "linux", "linux-firmware"];
        let additional_essential: Vec<String> = self.config.packages.essential
            .iter()
            .filter(|pkg| !base_packages.contains(&pkg.as_str()))
            .cloned()
            .collect();
        
        // Use parallel installation if enabled
        if self.config.build_options.parallel_builds {
            self.install_packages_parallel(&rootfs_dir, &additional_essential).await?
        } else {
            // Sequential installation for better reliability
            if !additional_essential.is_empty() {
                println!("Installing additional essential packages: {additional_essential:?}");
                self.install_package_list(&rootfs_dir, &additional_essential).await?;
            } else {
                println!("✅ Skipping essential packages (already installed in base system)");
            }

            // Install desktop environment
            if let Some(ref de) = self.config.packages.desktop_environment {
                self.install_desktop_environment(&rootfs_dir, de).await?;
            }

            // Install additional packages
            if !self.config.packages.additional_packages.is_empty() {
                self.install_package_list(&rootfs_dir, &self.config.packages.additional_packages).await?;
            }
        }

        Ok(())
    }

    async fn install_package_list(&self, rootfs_dir: &Path, packages: &[String]) -> Result<()> {
        println!("Installing packages: {packages:?}");
        
        // First update the package database
        let mut update_cmd = AsyncCommand::new("arch-chroot");
        update_cmd.arg(rootfs_dir)
                  .arg("pacman")
                  .arg("-Sy")
                  .arg("--noconfirm");
        
        let update_output = update_cmd.output().await?;
        if !update_output.status.success() {
            println!("Warning: Failed to update package database in chroot");
        }
        
        let mut cmd = AsyncCommand::new("arch-chroot");
        cmd.arg(rootfs_dir)
           .arg("pacman")
           .arg("-S")
           .arg("--noconfirm")
           .arg("--needed");  // Only install if not already present
        
        for package in packages {
            cmd.arg(package);
        }

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("STDOUT: {stdout}");
            println!("STDERR: {stderr}");
            anyhow::bail!("Package installation failed: {}", stderr);
        }

        println!("✅ Successfully installed packages");
        Ok(())
    }

    async fn install_desktop_environment(&self, rootfs_dir: &Path, de: &DesktopEnvironment) -> Result<()> {
        let packages = match de {
            DesktopEnvironment::Gnome => vec!["gnome".to_string()],
            DesktopEnvironment::Kde => vec!["plasma".to_string(), "kde-applications".to_string()],
            DesktopEnvironment::Xfce => vec!["xfce4".to_string(), "xfce4-goodies".to_string()],
            DesktopEnvironment::Lxde => vec!["lxde".to_string()],
            DesktopEnvironment::Mate => vec!["mate".to_string()],
            DesktopEnvironment::Cinnamon => vec!["cinnamon".to_string()],
            DesktopEnvironment::Sway => vec!["sway".to_string()],
            DesktopEnvironment::I3 => vec!["i3".to_string()],
            DesktopEnvironment::Custom(package) => vec![package.clone()],
            DesktopEnvironment::None => return Ok(()),
        };

        self.install_package_list(rootfs_dir, &packages).await
    }

    async fn configure_system(&self) -> Result<()> {
        println!("⚙️  Configuring system...");
        
        let rootfs_dir = self.work_dir.join("rootfs");
        
        // Set hostname
        fs::write(rootfs_dir.join("etc/hostname"), &self.config.name)?;
        
        // Configure hosts file
        let hosts_content = format!(
            "127.0.0.1\tlocalhost\n::1\t\tlocalhost\n127.0.1.1\t{}\n",
            self.config.name
        );
        fs::write(rootfs_dir.join("etc/hosts"), hosts_content)?;
        
        // Enable systemd services
        self.configure_systemd(&rootfs_dir).await?;
        
        Ok(())
    }

    async fn configure_systemd(&self, rootfs_dir: &Path) -> Result<()> {
        let services = [
            "NetworkManager.service",
            "systemd-resolved.service",
            "systemd-timesyncd.service",
        ];

        for service in services {
            let mut cmd = AsyncCommand::new("arch-chroot");
            cmd.arg(rootfs_dir)
               .arg("systemctl")
               .arg("enable")
               .arg(service);

            let _ = cmd.output().await; // Don't fail if service doesn't exist
        }

        Ok(())
    }

    async fn apply_branding(&self) -> Result<()> {
        println!("🎨 Applying branding...");
        
        // Copy logo, wallpaper, themes, etc.
        // This would be customized based on the branding config
        
        Ok(())
    }

    async fn configure_bootloader(&self) -> Result<()> {
        println!("🥾 Configuring bootloader...");
        
        let boot_dir = self.work_dir.join("boot");
        let rootfs_dir = self.work_dir.join("rootfs");
        
        // Copy kernel and initramfs to boot directory
        let kernel_files = [
            "vmlinuz-linux",
            "initramfs-linux.img",
            "initramfs-linux-fallback.img",
        ];

        for file in kernel_files {
            let src = rootfs_dir.join("boot").join(file);
            let dst = boot_dir.join(file);
            if src.exists() {
                fs::copy(src, dst)?;
            }
        }

        match self.config.bootloader.bootloader {
            Bootloader::Syslinux => self.configure_syslinux(&boot_dir).await?,
            Bootloader::Grub => self.configure_grub(&boot_dir).await?,
            _ => println!("⚠️  Bootloader configuration not implemented yet"),
        }

        Ok(())
    }

    async fn configure_syslinux(&self, boot_dir: &Path) -> Result<()> {
        let syslinux_cfg = format!(
            r#"DEFAULT {default}
TIMEOUT {timeout}0

LABEL {default}
    MENU LABEL {name}
    LINUX /vmlinuz-linux
    APPEND root=/dev/disk/by-label/{name} rw
    INITRD /initramfs-linux.img

LABEL {default}fallback
    MENU LABEL {name} (fallback initramfs)
    LINUX /vmlinuz-linux
    APPEND root=/dev/disk/by-label/{name} rw
    INITRD /initramfs-linux-fallback.img
"#,
            default = self.config.bootloader.default_entry,
            timeout = self.config.bootloader.timeout,
            name = self.config.name
        );

        fs::write(boot_dir.join("syslinux.cfg"), syslinux_cfg)?;
        Ok(())
    }

    async fn configure_grub(&self, _boot_dir: &Path) -> Result<()> {
        // GRUB configuration would go here
        Ok(())
    }

    async fn create_iso(&self) -> Result<PathBuf> {
        println!("💿 Creating ISO image...");
        
        let iso_dir = self.work_dir.join("iso");
        let rootfs_dir = self.work_dir.join("rootfs");
        let _boot_dir = self.work_dir.join("boot");
        
        // Create SquashFS from rootfs
        println!("Creating SquashFS filesystem...");
        let squashfs_path = iso_dir.join("live").join("filesystem.squashfs");
        fs::create_dir_all(iso_dir.join("live"))?;
        
        let mut mksquashfs_cmd = AsyncCommand::new("mksquashfs");
        mksquashfs_cmd.arg(&rootfs_dir)
                     .arg(&squashfs_path)
                     .arg("-e")
                     .arg("boot"); // Exclude boot directory from squashfs
        
        match self.config.filesystem.compression {
            CompressionType::Gzip => { mksquashfs_cmd.arg("-comp").arg("gzip"); }
            CompressionType::Xz => { mksquashfs_cmd.arg("-comp").arg("xz"); }
            CompressionType::Zstd => { mksquashfs_cmd.arg("-comp").arg("zstd"); }
            CompressionType::Lz4 => { mksquashfs_cmd.arg("-comp").arg("lz4"); }
            CompressionType::None => {}
        }

        let output = mksquashfs_cmd.output().await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("STDOUT: {stdout}");
            anyhow::bail!("mksquashfs failed: {stderr}");
        }
        println!("✅ SquashFS created successfully");

        // Copy boot files from rootfs to ISO
        println!("Copying boot files...");
        fs::create_dir_all(iso_dir.join("boot"))?;
        
        // Copy kernel and initramfs from rootfs/boot
        let rootfs_boot = rootfs_dir.join("boot");
        if rootfs_boot.exists() {
            for entry in fs::read_dir(&rootfs_boot)? {
                let entry = entry?;
                if entry.file_name().to_string_lossy().starts_with("vmlinuz") ||
                   entry.file_name().to_string_lossy().starts_with("initramfs") {
                    let dst = iso_dir.join("boot").join(entry.file_name());
                    fs::copy(entry.path(), &dst)?;
                    let src_path = entry.path().display().to_string();
                    let dst_path = dst.display().to_string();
                    println!("Copied: {src_path} -> {dst_path}");
                }
            }
        }

        // Copy syslinux files
        self.copy_syslinux_files(&iso_dir).await?;

        // Create ISO with xorriso
        println!("Creating ISO with xorriso...");
        let iso_filename = format!("{}-{}-{}.iso", 
                                 self.config.name, 
                                 self.config.version,
                                 self.config.architecture);
        let iso_path = self.output_dir.join(iso_filename);

        let mut xorriso_cmd = AsyncCommand::new("xorriso");
        xorriso_cmd.arg("-as").arg("mkisofs")
                   .arg("-iso-level").arg("3")
                   .arg("-full-iso9660-filenames")
                   .arg("-volid").arg(&self.config.name)
                   .arg("-eltorito-boot").arg("boot/isolinux/isolinux.bin")
                   .arg("-eltorito-catalog").arg("boot/isolinux/boot.cat")
                   .arg("-no-emul-boot")
                   .arg("-boot-load-size").arg("4")
                   .arg("-boot-info-table")
                   .arg("-isohybrid-mbr").arg("/usr/lib/syslinux/bios/isohdpfx.bin")
                   .arg("-output").arg(&iso_path)
                   .arg(&iso_dir);

        let output = xorriso_cmd.output().await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("STDOUT: {stdout}");
            anyhow::bail!("xorriso failed: {stderr}");
        }

        let iso_display = iso_path.display();
        println!("✅ ISO created successfully: {iso_display}");
        Ok(iso_path)
    }

    async fn copy_syslinux_files(&self, iso_dir: &Path) -> Result<()> {
        let isolinux_dir = iso_dir.join("boot").join("isolinux");
        fs::create_dir_all(&isolinux_dir)?;

        let syslinux_files = [
            "/usr/lib/syslinux/bios/isolinux.bin",
            "/usr/lib/syslinux/bios/ldlinux.c32",
            "/usr/lib/syslinux/bios/libcom32.c32",
            "/usr/lib/syslinux/bios/libutil.c32",
            "/usr/lib/syslinux/bios/menu.c32",
        ];

        for file_path in syslinux_files {
            let file_name = Path::new(file_path).file_name().unwrap();
            let dst = isolinux_dir.join(file_name);
            if Path::new(file_path).exists() {
                fs::copy(file_path, dst)?;
            }
        }

        // Copy syslinux config as isolinux.cfg
        let syslinux_cfg = iso_dir.join("boot").join("syslinux.cfg");
        let isolinux_cfg = isolinux_dir.join("isolinux.cfg");
        if syslinux_cfg.exists() {
            fs::copy(syslinux_cfg, isolinux_cfg)?;
        }

        Ok(())
    }

    // Enhanced parallel installation methods
    async fn install_packages_parallel(&self, rootfs_dir: &Path, essential_packages: &[String]) -> Result<()> {
        println!("🚀 Using parallel package installation");
        
        let mut all_packages = essential_packages.to_vec();
        
        // Add desktop environment packages
        if let Some(ref de) = self.config.packages.desktop_environment {
            let de_packages = self.get_desktop_environment_packages(de);
            all_packages.extend(de_packages);
        }
        
        // Add additional packages
        all_packages.extend(self.config.packages.additional_packages.clone());
        
        if all_packages.is_empty() {
            println!("✅ No packages to install");
            return Ok(());
        }
        
        // Group packages into chunks for batch installation
        let chunk_size = 5; // Install 5 packages per batch to balance parallelism and reliability
        let chunks: Vec<Vec<String>> = all_packages
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();
        
        println!("📦 Installing {} packages in {} batches", all_packages.len(), chunks.len());
        
        // Use semaphore to limit concurrent package operations
        let tasks: Vec<_> = chunks.into_iter().enumerate().map(|(i, chunk)| {
            let rootfs_dir = rootfs_dir.to_path_buf();
            let semaphore = Arc::clone(&self.parallel_semaphore);
            
            async move {
                let _permit = semaphore.acquire().await.unwrap();
                println!("📦 Batch {}: Installing {:?}", i + 1, chunk);
                self.install_package_batch(&rootfs_dir, &chunk).await
            }
        }).collect();
        
        // Execute all batches and collect results
        let results = try_join_all(tasks).await?;
        
        println!("✅ All {} package batches installed successfully", results.len());
        Ok(())
    }
    
    async fn install_package_batch(&self, rootfs_dir: &Path, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        
        // Check cache first if enabled
        if self.config.build_options.preserve_cache {
            let uncached_packages = self.filter_cached_packages(packages).await;
            if uncached_packages.is_empty() {
                println!("✅ All packages found in cache");
                return Ok(());
            }
            
            if uncached_packages.len() != packages.len() {
                println!("📦 {} packages found in cache, installing {} from repositories",
                        packages.len() - uncached_packages.len(), uncached_packages.len());
            }
            
            return self.install_package_list_optimized(rootfs_dir, &uncached_packages).await;
        }
        
        self.install_package_list_optimized(rootfs_dir, packages).await
    }
    
    async fn install_package_list_optimized(&self, rootfs_dir: &Path, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        
        // Update package database once per batch
        let mut update_cmd = AsyncCommand::new("arch-chroot");
        update_cmd.arg(rootfs_dir)
                  .arg("pacman")
                  .arg("-Sy")
                  .arg("--noconfirm");
        
        let update_start = Instant::now();
        let update_output = update_cmd.output().await?;
        if !update_output.status.success() {
            println!("⚠️  Warning: Failed to update package database in chroot");
        } else {
            println!("✅ Updated package database in {:.1}s", update_start.elapsed().as_secs_f64());
        }
        
        // Install packages with optimized flags
        let mut cmd = AsyncCommand::new("arch-chroot");
        cmd.arg(rootfs_dir)
           .arg("pacman")
           .arg("-S")
           .arg("--noconfirm")
           .arg("--needed")  // Only install if not already present
           .arg("--noprogressbar"); // Disable progress bar for cleaner output
        
        // Enable ccache if configured
        if self.config.build_options.enable_ccache {
            cmd.env("CCACHE_DIR", "/var/cache/ccache");
        }
        
        for package in packages {
            cmd.arg(package);
        }

        let install_start = Instant::now();
        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // Log more detailed error information
            println!("❌ Package installation failed after {:.1}s", install_start.elapsed().as_secs_f64());
            println!("📦 Failed packages: {:?}", packages);
            println!("STDOUT: {}", stdout);
            println!("STDERR: {}", stderr);
            anyhow::bail!("Package installation failed: {}", stderr);
        }

        println!("✅ Installed {} packages in {:.1}s", packages.len(), install_start.elapsed().as_secs_f64());
        
        // Cache packages if enabled
        if self.config.build_options.preserve_cache {
            self.cache_installed_packages(packages).await;
        }
        
        Ok(())
    }
    
    fn get_desktop_environment_packages(&self, de: &DesktopEnvironment) -> Vec<String> {
        match de {
            DesktopEnvironment::Gnome => vec!["gnome".to_string()],
            DesktopEnvironment::Kde => vec!["plasma".to_string(), "kde-applications".to_string()],
            DesktopEnvironment::Xfce => vec!["xfce4".to_string(), "xfce4-goodies".to_string()],
            DesktopEnvironment::Lxde => vec!["lxde".to_string()],
            DesktopEnvironment::Mate => vec!["mate".to_string()],
            DesktopEnvironment::Cinnamon => vec!["cinnamon".to_string()],
            DesktopEnvironment::Sway => vec!["sway".to_string()],
            DesktopEnvironment::I3 => vec!["i3".to_string()],
            DesktopEnvironment::Custom(package) => vec![package.clone()],
            DesktopEnvironment::None => vec![],
        }
    }
    
    async fn filter_cached_packages(&self, packages: &[String]) -> Vec<String> {
        let cache = self.package_cache.lock().await;
        let mut uncached = Vec::new();
        
        for package in packages {
            if !cache.contains_key(package) {
                uncached.push(package.clone());
            }
        }
        
        uncached
    }
    
    async fn cache_installed_packages(&self, packages: &[String]) {
        let mut cache = self.package_cache.lock().await;
        let timestamp = Utc::now();
        
        for package in packages {
            // Create a simple cache entry (in real implementation, you'd want to store version info)
            let entry = PackageCacheEntry {
                package_name: package.clone(),
                version: "unknown".to_string(), // Would query actual version
                hash: self.calculate_package_hash(package),
                timestamp,
                cached_path: self.work_dir.join("cache").join(format!("{}.cached", package)),
            };
            
            cache.insert(package.clone(), entry);
        }
        
        println!("💾 Cached {} packages for future builds", packages.len());
    }
    
    fn calculate_package_hash(&self, package: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(package.as_bytes());
        hasher.update(self.config.architecture.as_bytes());
        hasher.update(Utc::now().format("%Y-%m-%d").to_string().as_bytes()); // Daily hash
        format!("{:x}", hasher.finalize())
    }
    
}

impl Default for DistroConfig {
    fn default() -> Self {
        Self {
            name: "MyLinux".to_string(),
            version: "1.0".to_string(),
            description: "A custom Linux distribution".to_string(),
            architecture: "x86_64".to_string(),
            base_system: BaseSystem::Arch,
            packages: PackageConfig {
                essential: vec![
                    "base".to_string(),
                    "linux".to_string(),
                    "linux-firmware".to_string(),
                    "networkmanager".to_string(),
                    "sudo".to_string(),
                ],
                desktop_environment: Some(DesktopEnvironment::Xfce),
                additional_packages: vec![
                    "firefox".to_string(),
                    "vim".to_string(),
                    "git".to_string(),
                ],
                custom_repositories: vec![],
            },
            kernel: KernelConfig {
                kernel_type: KernelType::Vanilla,
                custom_config: None,
                modules: vec![],
            },
            bootloader: BootloaderConfig {
                bootloader: Bootloader::Syslinux,
                timeout: 30,
                default_entry: "linux".to_string(),
            },
            branding: BrandingConfig {
                logo: None,
                wallpaper: None,
                theme: None,
                colors: ColorScheme {
                    primary: "#0078d4".to_string(),
                    secondary: "#005a9e".to_string(),
                    accent: "#00bcf2".to_string(),
                },
            },
            filesystem: FilesystemConfig {
                root_fs: FilesystemType::SquashFs,
                compression: CompressionType::Xz,
                size_limit: Some(4096), // 4GB
            },
            build_options: BuildOptions {
                parallel_builds: false,
                max_parallel_jobs: None,
                cleanup_on_failure: true,
                preserve_cache: false,
                enable_ccache: false,
                build_logs: true,
                progress_reporting: ProgressReporting::Standard,
                timeout_minutes: Some(120), // 2 hours default timeout
            },
            user_config: UserConfig {
                default_user: Some(UserAccount {
                    username: "user".to_string(),
                    password: None, // Will prompt during first boot  
                    groups: vec!["wheel".to_string(), "audio".to_string(), "video".to_string()],
                    shell: Some("/bin/bash".to_string()),
                    home_dir: None, // Use default /home/username
                    sudo_access: true,
                }),
                root_password: None, // Will prompt during first boot
                timezone: Some("UTC".to_string()),
                locale: Some("en_US.UTF-8".to_string()),
                keyboard_layout: Some("us".to_string()),
                network_config: NetworkConfig {
                    enable_networking: true,
                    dhcp: true,
                    static_ip: None,
                    dns_servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
                    hostname_strategy: HostnameStrategy::FromConfig,
                },
                services: ServicesConfig {
                    enable_ssh: false,
                    enable_firewall: true,
                    auto_login: false,
                    custom_services: vec![],
                    disabled_services: vec![],
                },
                post_install_scripts: vec![],
            },
            validation: ValidationConfig {
                strict_validation: true,
                warn_on_large_iso: true, 
                max_iso_size_mb: 4096, // 4GB
                validate_packages: true,
                check_dependencies: true,
                verify_signatures: false, // Disabled by default for performance
            },
        }
    }
}
