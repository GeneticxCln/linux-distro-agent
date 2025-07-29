use dialoguer::{Input, Confirm, Select};
use anyhow::Result;
use crate::distro_builder::{
    DistroConfig, BuildOptions, UserConfig, PackageConfig, KernelConfig,
    BootloaderConfig, BrandingConfig, FilesystemConfig, ValidationConfig,
    BaseSystem, DesktopEnvironment, KernelType, Bootloader,
    FilesystemType, CompressionType, UserAccount, NetworkConfig, ServicesConfig,
    ColorScheme, ProgressReporting, HostnameStrategy
};
use std::path::PathBuf;

pub struct ConfigWizard;

impl ConfigWizard {
    pub fn run() -> Result<DistroConfig> {
        println!("🎯 Welcome to the Interactive Linux Distribution Configuration Wizard!");
        println!("This wizard will guide you through creating a custom Linux distribution.");
        println!();

        // Basic distribution information
        println!("📋 Basic Distribution Information");
        let name: String = Input::new()
            .with_prompt("Distribution name")
            .default("MyCustomLinux".to_string())
            .interact_text()?;

        let version: String = Input::new()
            .with_prompt("Version")
            .default("1.0.0".to_string())
            .interact_text()?;

        let description: String = Input::new()
            .with_prompt("Description")
            .default(format!("Custom Linux distribution based on {name}"))
            .interact_text()?;

        // Architecture selection
        let architectures = vec!["x86_64", "aarch64", "i686", "armv7h"];
        let arch_index = Select::new()
            .with_prompt("Target architecture")
            .items(&architectures)
            .default(0)
            .interact()?;
        let architecture = architectures[arch_index].to_string();

        // Base system selection
        println!();
        println!("🏗️  Base System Selection");
        let base_systems = vec!["Arch Linux", "Debian", "Ubuntu", "Fedora", "CentOS", "openSUSE", "Alpine", "From Scratch"];
        let base_system_index = Select::new()
            .with_prompt("Select the base system")
            .items(&base_systems)
            .default(0)
            .interact()?;
        let base_system = match base_system_index {
            0 => BaseSystem::Arch,
            1 => BaseSystem::Debian,
            2 => BaseSystem::Ubuntu,
            3 => BaseSystem::Fedora,
            4 => BaseSystem::CentOS,
            5 => BaseSystem::OpenSUSE,
            6 => BaseSystem::Alpine,
            7 => BaseSystem::Scratch,
            _ => BaseSystem::Arch,
        };

        // Package configuration
        println!();
        println!("📦 Package Configuration");
        let essential_packages: String = Input::new()
            .with_prompt("Essential packages (comma-separated)")
            .default("base,linux,systemd,bash,coreutils".to_string())
            .interact_text()?;
        let essential_packages: Vec<String> = essential_packages
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let desktop_environments = vec!["None", "GNOME", "KDE", "XFCE", "LXDE", "i3", "Sway", "Cinnamon", "MATE"];
        let de_index = Select::new()
            .with_prompt("Desktop environment")
            .items(&desktop_environments)
            .default(0)
            .interact()?;
        let desktop_environment = match de_index {
            1 => DesktopEnvironment::Gnome,
            2 => DesktopEnvironment::Kde,
            3 => DesktopEnvironment::Xfce,
            4 => DesktopEnvironment::Lxde,
            5 => DesktopEnvironment::I3,
            6 => DesktopEnvironment::Sway,
            7 => DesktopEnvironment::Cinnamon,
            8 => DesktopEnvironment::Mate,
            _ => DesktopEnvironment::None,
        };

        let additional_packages: String = Input::new()
            .with_prompt("Additional packages (comma-separated, optional)")
            .default("".to_string())
            .interact_text()?;
        let additional_packages: Vec<String> = if additional_packages.is_empty() {
            vec![]
        } else {
            additional_packages
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        };

        let packages = PackageConfig {
            essential: essential_packages,
            desktop_environment: Some(desktop_environment),
            additional_packages,
            custom_repositories: vec![],
        };

        // Kernel configuration
        println!();
        println!("🐧 Kernel Configuration");
        let kernel_types = vec!["Vanilla", "LTS", "Hardened", "Real-time", "Custom"];
        let kernel_index = Select::new()
            .with_prompt("Kernel type")
            .items(&kernel_types)
            .default(0)
            .interact()?;
        let kernel_type = match kernel_index {
            0 => KernelType::Vanilla,
            1 => KernelType::Lts,
            2 => KernelType::Hardened,
            3 => KernelType::Rt,
            4 => KernelType::Custom("custom".to_string()),
            _ => KernelType::Vanilla,
        };

        let kernel_modules: String = Input::new()
            .with_prompt("Additional kernel modules (comma-separated, optional)")
            .default("".to_string())
            .interact_text()?;
        let kernel_modules: Vec<String> = if kernel_modules.is_empty() {
            vec![]
        } else {
            kernel_modules
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        };

        let kernel = KernelConfig {
            kernel_type,
            custom_config: None,
            modules: kernel_modules,
        };

        // Bootloader configuration
        println!();
        println!("🚀 Bootloader Configuration");
        let bootloaders = vec!["GRUB", "systemd-boot", "rEFInd", "Syslinux"];
        let bootloader_index = Select::new()
            .with_prompt("Bootloader")
            .items(&bootloaders)
            .default(0)
            .interact()?;
        let bootloader_type = match bootloader_index {
            0 => Bootloader::Grub,
            1 => Bootloader::Systemd,
            2 => Bootloader::Refind,
            3 => Bootloader::Syslinux,
            _ => Bootloader::Grub,
        };

        let timeout: u32 = Input::new()
            .with_prompt("Boot timeout (seconds)")
            .default(5)
            .interact_text()?;

        let bootloader = BootloaderConfig {
            bootloader: bootloader_type,
            timeout,
            default_entry: "default".to_string(),
        };

        // Branding configuration
        println!();
        println!("🎨 Branding Configuration");
        let setup_branding = Confirm::new()
            .with_prompt("Configure custom branding?")
            .default(false)
            .interact()?;

        let branding = if setup_branding {
            let logo_path: String = Input::new()
                .with_prompt("Logo file path (optional)")
                .default("".to_string())
                .interact_text()?;
            let logo_path = if logo_path.is_empty() { None } else { Some(PathBuf::from(logo_path)) };

            let wallpaper_path: String = Input::new()
                .with_prompt("Wallpaper file path (optional)")
                .default("".to_string())
                .interact_text()?;
            let wallpaper_path = if wallpaper_path.is_empty() { None } else { Some(PathBuf::from(wallpaper_path)) };

            let primary_color: String = Input::new()
                .with_prompt("Primary color (hex, e.g., #2196F3)")
                .default("#2196F3".to_string())
                .interact_text()?;

            let secondary_color: String = Input::new()
                .with_prompt("Secondary color (hex, e.g., #FFC107)")
                .default("#FFC107".to_string())
                .interact_text()?;

            BrandingConfig {
                logo: logo_path,
                wallpaper: wallpaper_path,
                colors: ColorScheme {
                    primary: primary_color,
                    secondary: secondary_color,
                    accent: "#FF5722".to_string(),
                },
                theme: Some("default".to_string()),
            }
        } else {
            BrandingConfig {
                logo: None,
                wallpaper: None,
                colors: ColorScheme {
                    primary: "#2196F3".to_string(),
                    secondary: "#FFC107".to_string(),
                    accent: "#FF5722".to_string(),
                },
                theme: None,
            }
        };

        // Filesystem configuration
        println!();
        println!("🗂️  Filesystem Configuration");
        let filesystems = vec!["SquashFS", "ext4", "btrfs", "xfs"];
        let fs_index = Select::new()
            .with_prompt("Root filesystem type")
            .items(&filesystems)
            .default(0)
            .interact()?;
        let filesystem_type = match fs_index {
            0 => FilesystemType::SquashFs,
            1 => FilesystemType::Ext4,
            2 => FilesystemType::Btrfs,
            3 => FilesystemType::Xfs,
            _ => FilesystemType::SquashFs,
        };

        let compressions = vec!["None", "gzip", "xz", "zstd", "lz4"];
        let comp_index = Select::new()
            .with_prompt("ISO compression")
            .items(&compressions)
            .default(2)
            .interact()?;
        let compression = match comp_index {
            0 => CompressionType::None,
            1 => CompressionType::Gzip,
            2 => CompressionType::Xz,
            3 => CompressionType::Zstd,
            4 => CompressionType::Lz4,
            _ => CompressionType::Xz,
        };

        let filesystem = FilesystemConfig {
            root_fs: filesystem_type,
            compression,
            size_limit: None,
        };

        // Build options
        println!();
        println!("⚙️  Build Options");
        let parallel_builds = Confirm::new()
            .with_prompt("Enable parallel builds?")
            .default(true)
            .interact()?;

        let max_jobs: Option<usize> = if parallel_builds {
            Some(Input::new()
                .with_prompt("Maximum parallel jobs")
                .default(num_cpus::get())
                .interact_text()?)
        } else {
            Some(1)
        };

        let enable_caching = Confirm::new()
            .with_prompt("Enable package caching?")
            .default(true)
            .interact()?;

        let enable_ccache = Confirm::new()
            .with_prompt("Enable ccache for compilation?")
            .default(true)
            .interact()?;

        let verbose_output = Confirm::new()
            .with_prompt("Enable verbose output?")
            .default(false)
            .interact()?;

        let enable_network = Confirm::new()
            .with_prompt("Enable network during build?")
            .default(true)
            .interact()?;

        let build_options = BuildOptions {
            parallel_builds,
            max_parallel_jobs: max_jobs,
            cleanup_on_failure: true,
            preserve_cache: enable_caching,
            enable_ccache,
            build_logs: verbose_output,
            progress_reporting: if verbose_output { ProgressReporting::Verbose } else { ProgressReporting::Standard },
            timeout_minutes: None,
        };

        // User configuration
        println!();
        println!("👤 Default User Configuration");
        let username: String = Input::new()
            .with_prompt("Default username")
            .default("user".to_string())
            .interact_text()?;

        let password: String = Input::new()
            .with_prompt("Default password")
            .default("password".to_string())
            .interact_text()?;

        let shells = vec!["bash", "zsh", "fish", "dash"];
        let shell_index = Select::new()
            .with_prompt("Default shell")
            .items(&shells)
            .default(0)
            .interact()?;
        let shell = shells[shell_index].to_string();

        let groups: String = Input::new()
            .with_prompt("User groups (comma-separated)")
            .default("wheel,audio,video,users".to_string())
            .interact_text()?;
        let groups: Vec<String> = groups
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let timezone: String = Input::new()
            .with_prompt("Timezone")
            .default("UTC".to_string())
            .interact_text()?;

        let locale: String = Input::new()
            .with_prompt("Locale")
            .default("en_US.UTF-8".to_string())
            .interact_text()?;

        let keyboard_layout: String = Input::new()
            .with_prompt("Keyboard layout")
            .default("us".to_string())
            .interact_text()?;

        let user_config = UserConfig {
            default_user: Some(UserAccount {
                username,
                password: Some(password),
                groups,
                shell: Some(shell),
                home_dir: None,
                sudo_access: true,
            }),
            root_password: None,
            timezone: Some(timezone),
            locale: Some(locale),
            keyboard_layout: Some(keyboard_layout),
            network_config: NetworkConfig {
                enable_networking: enable_network,
                dhcp: true,
                static_ip: None,
                dns_servers: vec![],
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
        };

        // Validation configuration
        println!();
        println!("✅ Validation Configuration");
        let strict_validation = Confirm::new()
            .with_prompt("Enable strict validation?")
            .default(true)
            .interact()?;

        let show_warnings = Confirm::new()
            .with_prompt("Show validation warnings?")
            .default(true)
            .interact()?;

        let max_iso_size_gb: f64 = Input::new()
            .with_prompt("Maximum ISO size (GB)")
            .default(4.7)
            .interact_text()?;

        let validate_packages = Confirm::new()
            .with_prompt("Validate package dependencies?")
            .default(true)
            .interact()?;

        let check_dependencies = Confirm::new()
            .with_prompt("Check build dependencies?")
            .default(true)
            .interact()?;

        let verify_signatures = Confirm::new()
            .with_prompt("Verify package signatures?")
            .default(true)
            .interact()?;

        let validation = ValidationConfig {
            strict_validation,
            warn_on_large_iso: show_warnings,
            max_iso_size_mb: (max_iso_size_gb * 1024.0) as u64,
            validate_packages,
            check_dependencies,
            verify_signatures,
        };

        // Output path
        println!();
        println!("📁 Output Configuration");
        let output_path: String = Input::new()
            .with_prompt("Output directory")
            .default("./output".to_string())
            .interact_text()?;

        // Create final configuration
        let config = DistroConfig {
            name,
            version,
            description,
            architecture,
            base_system,
            packages,
            kernel,
            bootloader,
            branding,
            filesystem,
            build_options,
            user_config,
            validation,
        };

        // Display configuration summary
        println!();
        println!("🎉 Configuration Summary");
        println!("=======================");
        println!("Distribution: {} v{}", config.name, config.version);
        println!("Description: {}", config.description);
        println!("Architecture: {:?}", config.architecture);
        println!("Base System: {:?}", config.base_system);
        println!("Desktop Environment: {:?}", config.packages.desktop_environment);
        println!("Kernel Type: {:?}", config.kernel.kernel_type);
        println!("Bootloader: {:?}", config.bootloader.bootloader);
        println!("Filesystem: {:?}", config.filesystem.root_fs);
        if let Some(user) = &config.user_config.default_user {
            println!("Default User: {}", user.username);
        }
        println!("Output Path: {output_path}");
        println!();

        let confirm = Confirm::new()
            .with_prompt("Save this configuration and proceed?")
            .default(true)
            .interact()?;

        if !confirm {
            println!("Configuration cancelled.");
            std::process::exit(0);
        }

        println!("✅ Configuration completed successfully!");
        println!("You can now run the build process with this configuration.");

        Ok(config)
    }
}
