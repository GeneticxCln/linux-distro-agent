mod config;
mod config_manager;
mod distro;
mod distro_builder;
mod executor;
mod logger;
mod history;
mod cache;
mod monitoring;
mod remote_control;
mod package_manager;
mod system_config;
mod system_logger;
mod wsm;
mod security;
mod plugins;

use clap::{Parser, Subcommand, CommandFactory};
use clap_complete::{generate, Generator, Shell};
use anyhow::Result;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use distro::DistroInfo;
use distro_builder::{DistroBuilder, DistroConfig};
use executor::CommandExecutor;
use logger::Logger;

#[derive(Parser)]
#[clap(name = "linux-distro-agent")]
#[clap(about = "A comprehensive Linux distribution agent")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// Enable verbose output
    #[clap(short, long, global = true)]
    verbose: bool,
    
    /// Quiet mode - suppress non-essential output
    #[clap(short, long, global = true, conflicts_with = "verbose")]
    quiet: bool,
    
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Edit configuration in default editor
    Edit,
    /// Reset configuration to defaults
    Reset,
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
}

#[derive(Subcommand)]
enum CacheAction {
    /// Show cache status
    Status,
    /// Clear all cached data
    Clear,
    /// Show cached entries
    List,
}

#[derive(Subcommand)]
enum Commands {
    /// Detect and display current Linux distribution information
    Detect {
        /// Show extended information
        #[clap(short, long)]
        extended: bool,
    },
    /// Get package manager command for installing a package
    Install {
        /// Package name to install
        package: String,
        /// Execute the command directly (requires confirmation)
        #[clap(short, long)]
        execute: bool,
    },
    /// Get package manager command for searching packages
    Search {
        /// Search query
        query: String,
        /// Execute the command directly
        #[clap(short, long)]
        execute: bool,
    },
    /// Get system update command
    Update {
        /// Execute the command directly (requires confirmation)
        #[clap(short, long)]
        execute: bool,
    },
    /// Display comprehensive system information as JSON
    Info {
        /// Pretty print JSON output
        #[clap(short, long)]
        pretty: bool,
    },
    /// List all supported distributions and package managers
    ListSupported,
    /// Check system compatibility and provide recommendations
    Doctor,
    /// Remove a package
    Remove {
        /// Package name to remove
        package: String,
        /// Execute the command directly (requires confirmation)
        #[clap(short, long)]
        execute: bool,
    },
    /// List installed packages or package information
    List {
        /// Show detailed package information
        #[clap(short, long)]
        detailed: bool,
        /// Filter packages by name pattern
        #[clap(short, long)]
        filter: Option<String>,
    },
    /// Show package information
    PackageInfo {
        /// Package name to get information about  
        package: String,
    },
    /// Show command history
    History {
        /// Number of recent entries to show
        #[clap(short, long, default_value = "10")]
        limit: usize,
        /// Search history for specific terms
        #[clap(short, long)]
        search: Option<String>,
        /// Clear history
        #[clap(long)]
        clear: bool,
    },
    /// Configuration management
    Config {
        #[clap(subcommand)]
        action: ConfigAction,
    },
    /// Cache management
    Cache {
        #[clap(subcommand)]
        action: CacheAction,
    },
    /// Generate shell completion scripts
    Completions {
        /// The shell to generate completions for
        #[clap(value_enum)]
        shell: Shell,
    },
    /// Build a custom Linux distribution ISO
    BuildDistro {
        /// Distribution name
        #[clap(short, long)]
        name: Option<String>,
        /// Configuration file path
        #[clap(short = 'c', long)]
        config: Option<PathBuf>,
        /// Work directory for build process
        #[clap(short = 'w', long)]
        work_dir: Option<PathBuf>,
        /// Output directory for the ISO
        #[clap(short = 'o', long)]
        output_dir: Option<PathBuf>,
        /// Use default minimal configuration
        #[clap(long)]
        minimal: bool,
    },
    /// Generate a distro configuration template
    GenerateConfig {
        /// Output file path
        #[clap(short = 'o', long)]
        output: Option<PathBuf>,
        /// Configuration template type
        #[clap(long, default_value = "minimal")]
        template: String,
    },
    /// Update LDA to the latest version
    SelfUpdate {
        /// Force update even if already on latest version
        #[clap(short, long)]
        force: bool,
        /// Show what would be updated without actually updating
        #[clap(long)]
        dry_run: bool,
    },
    /// System monitoring and health checks
    Monitor {
        /// Show system metrics
        #[clap(short, long)]
        metrics: bool,
        /// Run health checks
        #[clap(long)]
        health: bool,
        /// Show metrics history
        #[clap(long)]
        history: bool,
    },
    /// Remote host management
    Remote {
        /// Remote host name
        #[clap(short, long)]
        host: String,
        /// Command to execute
        #[clap(short, long)]
        command: String,
        /// Run command as root
        #[clap(long)]
        sudo: bool,
        /// Test connectivity only
        #[clap(long)]
        test: bool,
    },
    /// System configuration management
    SystemConfig {
        /// Show current system configuration
        #[clap(short, long)]
        show: bool,
        /// Generate sample configuration
        #[clap(long)]
        sample: bool,
    },
    /// Window System Manager (WSM) - manage window systems, desktop environments, and displays
    Wsm {
        /// Detect current window system information
        #[clap(short, long)]
        detect: bool,
        /// Show available sessions
        #[clap(long)]
        sessions: bool,
        /// Show display configuration
        #[clap(long)]
        displays: bool,
        /// Restart a component (gdm, sddm, lightdm, x11)
        #[clap(long)]
        restart: Option<String>,
        /// Switch session type (x11, wayland)
        #[clap(long)]
        switch: Option<String>,
    },
    /// Security auditing and system hardening
    Security {
        /// Run full security audit
        #[clap(long)]
        audit: bool,
        /// Show security report in JSON format
        #[clap(long)]
        json: bool,
        /// Filter findings by severity level (low, medium, high, critical)
        #[clap(long)]
        severity: Option<String>,
        /// Filter findings by category
        #[clap(long)]
        category: Option<String>,
    },
    /// Plugin management system
    Plugin {
        /// List all available plugins
        #[clap(short, long)]
        list: bool,
        /// Show plugin information
        #[clap(long)]
        info: Option<String>,
        /// Enable a plugin
        #[clap(long)]
        enable: Option<String>,
        /// Disable a plugin
        #[clap(long)]
        disable: Option<String>,
        /// Execute a plugin
        #[clap(long)]
        exec: Option<String>,
        /// Plugin execution arguments
        #[clap(long)]
        args: Vec<String>,
        /// Install plugin from directory
        #[clap(long)]
        install: Option<PathBuf>,
        /// Uninstall a plugin
        #[clap(long)]
        uninstall: Option<String>,
        /// Create plugin template
        #[clap(long)]
        create: Option<String>,
        /// Plugin type for template creation
        #[clap(long, default_value = "command")]
        plugin_type: String,
    },
}

fn print_completions<G: Generator>(generator: G, cmd: &mut clap::Command) {
    generate(generator, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

fn handle_self_update(logger: &Logger, force: bool, dry_run: bool) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    logger.info(format!("Current LDA version: {}", current_version));
    
    if dry_run {
        logger.info("[DRY RUN] Checking for updates...");
    } else {
        logger.info("Checking for updates...");
    }
    
    // Check for latest release from GitHub API
    let output = Command::new("curl")
        .args(["-s", "https://api.github.com/repos/GeneticxCln/linux-distro-agent/releases/latest"])
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to check for updates. Make sure curl is installed and you have internet access."));
    }
    
    let response = String::from_utf8(output.stdout)?;
    let json: serde_json::Value = serde_json::from_str(&response)?;
    
    // Check if the response indicates no releases exist
    if let Some(message) = json["message"].as_str() {
        if message == "Not Found" {
            logger.info("No releases found in the repository yet.");
            logger.info("The self-update feature will be available once the first release is published.");
            return Ok(());
        }
    }
    
    let latest_version = json["tag_name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Could not parse latest version from GitHub API"))?;
    
    let latest_version = latest_version.trim_start_matches('v');
    
    logger.info(format!("Latest LDA version: {}", latest_version));
    
    if latest_version == current_version && !force {
        logger.success("🎉 You're already running the latest version!");
        return Ok(());
    }
    
    if dry_run {
        if latest_version != current_version {
            logger.info(format!("[DRY RUN] Would update from {} to {}", current_version, latest_version));
        } else {
            logger.info("[DRY RUN] Would force update (same version)");
        }
        return Ok(());
    }
    
    logger.info("Downloading and installing the latest version...");
    
    // Download and execute the install script
    let install_cmd = "curl -fsSL https://raw.githubusercontent.com/GeneticxCln/linux-distro-agent/main/install.sh | bash";
    
    logger.verbose(format!("Running: {}", install_cmd));
    
    let status = Command::new("bash")
        .args(["-c", install_cmd])
        .status()?;
    
    if status.success() {
        logger.success("🎉 LDA has been successfully updated!");
        logger.info("You may need to restart your terminal or run 'hash -r' to use the updated version.");
    } else {
        return Err(anyhow::anyhow!("Update failed. Please try updating manually."));
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Handle completions command early to avoid unnecessary distro detection
    if let Commands::Completions { shell } = cli.command {
        let mut cmd = Cli::command();
        print_completions(shell, &mut cmd);
        return Ok(());
    }
    
    let logger = Logger::new(cli.verbose, cli.quiet);
    
    // Handle distro builder commands that don't need distro detection
    match &cli.command {
        Commands::BuildDistro { name, config, work_dir, output_dir, minimal } => {
            let config = if *minimal {
                logger.info("Using default minimal configuration.");
                DistroConfig::default()
            } else if let Some(config_path) = config {
                logger.info("Loading configuration from file.");
                let config_string = std::fs::read_to_string(config_path)?;
                toml::from_str(&config_string)?
            } else {
                return Err(anyhow::anyhow!("No configuration provided! Use --minimal or provide a config file."));
            };

            // Override the name if provided
            let config = if let Some(name) = name {
                DistroConfig { name: name.clone(), ..config }
            } else {
                config
            };

            // Define work and output directories
            let work_dir = work_dir.clone().unwrap_or_else(|| "./work_dir".into());
            let output_dir = output_dir.clone().unwrap_or_else(|| "./output".into());

            // Create builder
            let builder = DistroBuilder::new(config, work_dir, output_dir);
            let rt = tokio::runtime::Runtime::new()?;
            let iso_path = rt.block_on(builder.build())?;

            logger.success(format!("🎉 Distro build complete! ISO created at: {}", iso_path.display()));
            return Ok(());
        }
        Commands::GenerateConfig { output, template } => {
            let template_config = match template.as_str() {
                "minimal" => DistroConfig::default(),
                _ => return Err(anyhow::anyhow!("Unknown template type: {}", template)),
            };

            let toml_string = toml::to_string_pretty(&template_config)?;
            if let Some(output_path) = output {
                std::fs::write(output_path, &toml_string)?;
                logger.success("Configuration template written to file.");
            } else {
                println!("{}", toml_string);
            }
            return Ok(());
        }
        Commands::SelfUpdate { force, dry_run } => {
            return handle_self_update(&logger, *force, *dry_run);
        }
        _ => {}
    }
    
    // For other commands, detect distro
    let distro = DistroInfo::detect()?;

    match cli.command {
        Commands::Detect { extended } => {
            logger.info(format!("Detected Linux distribution: {}", distro.name));
            if let Some(version) = &distro.version {
                logger.info(format!("Version: {version}"));
            }
            if let Some(pm) = &distro.package_manager {
                logger.info(format!("Package Manager: {pm}"));
            }
            
            if extended {
                if let Some(id) = &distro.id {
                    logger.verbose(format!("ID: {id}"));
                }
                if let Some(id_like) = &distro.id_like {
                    logger.verbose(format!("ID Like: {id_like}"));
                }
                if let Some(pretty_name) = &distro.pretty_name {
                    logger.verbose(format!("Pretty Name: {pretty_name}"));
                }
                if let Some(home_url) = &distro.home_url {
                    logger.verbose(format!("Home URL: {home_url}"));
                }
                if let Some(support_url) = &distro.support_url {
                    logger.verbose(format!("Support URL: {support_url}"));
                }
                if let Some(bug_report_url) = &distro.bug_report_url {
                    logger.verbose(format!("Bug Report URL: {bug_report_url}"));
                }
            }
        }
        Commands::Install { package, execute } => {
            match distro.get_package_install_command(&package) {
                Some(cmd) => {
                    if execute {
                        let _ = CommandExecutor::execute_command(&cmd, true)?;
                    } else {
                        logger.output(format!("To install '{package}', run: {cmd}"));
                    }
                }
                None => logger.error("Unable to determine package install command for this distribution"),
            }
        }
        Commands::Search { query, execute } => {
            match distro.get_package_search_command(&query) {
                Some(cmd) => {
                    if execute {
                        let safe_to_run = CommandExecutor::is_safe_to_execute(&cmd);
                        let _ = CommandExecutor::execute_command(&cmd, !safe_to_run)?;
                    } else {
                        logger.output(format!("To search for '{query}', run: {cmd}"));
                    }
                }
                None => logger.error("Unable to determine package search command for this distribution"),
            }
        }
        Commands::Update { execute } => {
            match distro.get_system_update_command() {
                Some(cmd) => {
                    if execute {
                        let _ = CommandExecutor::execute_command(&cmd, true)?;
                    } else {
                        logger.output(format!("To update the system, run: {cmd}"));
                    }
                }
                None => logger.error("Unable to determine system update command for this distribution"),
            }
        }
        Commands::Info { pretty } => {
            let json = if pretty {
                serde_json::to_string_pretty(&distro)?
            } else {
                serde_json::to_string(&distro)?
            };
            logger.json(&json);
        }
        Commands::ListSupported => {
            logger.info("Supported Distributions and Package Managers:");
            logger.info("");
            logger.info("• Arch-based: pacman (Arch Linux, CachyOS, Manjaro, EndeavourOS)");
            logger.info("• Debian-based: apt (Ubuntu, Debian, Pop!_OS, Elementary OS)");
            logger.info("• Red Hat-based: dnf (Fedora, RHEL, CentOS, Rocky Linux, AlmaLinux)");
            logger.info("• SUSE-based: zypper (openSUSE Leap, openSUSE Tumbleweed)");
            logger.info("• Gentoo: portage");
            logger.info("• NixOS: nix");
            logger.info("• Alpine: apk");
        }
        Commands::Doctor => {
            logger.info("System Compatibility Check:");
            logger.info("");
            logger.success(format!("✓ Distribution: {}", distro.name));
            
            if let Some(pm) = &distro.package_manager {
                logger.success(format!("✓ Package Manager: {pm}"));
            } else {
                logger.warn("⚠ Package Manager: Unknown - limited functionality");
            }
            
            if distro.version.is_some() {
                logger.success("✓ Version information available");
            } else {
                logger.warn("⚠ Version information not available");
            }
            
            logger.info("");
            logger.info("Recommendations:");
            if distro.package_manager.is_none() {
                logger.info("• Consider adding support for your distribution");
                logger.info("• Check if your distribution uses a supported package manager");
            } else {
                logger.success("• Your system is fully supported!");
                logger.info("• All package management commands should work correctly");
            }
        }
        Commands::Remove { package, execute } => {
            match distro.get_package_remove_command(&package) {
                Some(cmd) => {
                    if execute {
                        let _ = CommandExecutor::execute_command(&cmd, true)?;
                    } else {
                        logger.output(format!("To remove '{package}', run: {cmd}"));
                    }
                }
                None => logger.error("Unable to determine package remove command for this distribution"),
            }
        }
        Commands::List { detailed, filter } => {
            match distro.get_package_list_command(detailed, filter.as_deref()) {
                Some(cmd) => {
                    let safe_to_run = CommandExecutor::is_safe_to_execute(&cmd);
                    let _ = CommandExecutor::execute_command(&cmd, !safe_to_run)?;
                }
                None => logger.error("Unable to determine package list command for this distribution"),
            }
        }
        Commands::PackageInfo { package } => {
            match distro.get_package_info_command(&package) {
                Some(cmd) => {
                    let safe_to_run = CommandExecutor::is_safe_to_execute(&cmd);
                    let _ = CommandExecutor::execute_command(&cmd, !safe_to_run)?;
                }
                None => logger.error("Unable to determine package info command for this distribution"),
            }
        }
        Commands::History { limit, search, clear } => {
            let mut history_manager = history::HistoryManager::new()?;
            
            if clear {
                history_manager.clear()?;
                logger.success("Command history cleared");
            } else {
                let entries = if let Some(search_term) = search {
                    history_manager.search(&search_term, limit)?
                } else {
                    history_manager.get_recent(limit)?
                };
                
                if entries.is_empty() {
                    logger.info("No history entries found");
                } else {
                    logger.info("Command History:");
                    for entry in entries {
                        logger.output(format!("{} - {} - {}", 
                            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                            entry.command,
                            entry.package.as_ref().map_or("N/A".to_string(), |p| p.clone())
                        ));
                    }
                }
            }
        }
        Commands::Config { action } => {
            let mut config_manager = config_manager::ConfigManager::new()?;
            
            match action {
                ConfigAction::Show => {
                    let config = config_manager.load()?;
                    let json = serde_json::to_string_pretty(&config)?;
                    logger.output(json);
                }
                ConfigAction::Edit => {
                    config_manager.edit()?;
                    logger.success("Configuration file opened in editor");
                }
                ConfigAction::Reset => {
                    config_manager.reset()?;
                    logger.success("Configuration reset to defaults");
                }
                ConfigAction::Set { key, value } => {
                    config_manager.set(&key, &value)?;
                    logger.success(format!("Set {key} = {value}"));
                }
            }
        }
        Commands::Cache { action } => {
            let mut cache_manager = cache::CacheManager::new()?;
            
            match action {
                CacheAction::Status => {
                    let status = cache_manager.status()?;
                    logger.info(format!("Cache entries: {}", status.entry_count));
                    logger.info(format!("Cache size: {} bytes", status.total_size));
                    logger.info(format!("Last updated: {}", 
                        status.last_updated.map_or("Never".to_string(), 
                            |ts| ts.format("%Y-%m-%d %H:%M:%S").to_string())
                    ));
                }
                CacheAction::Clear => {
                    cache_manager.clear()?;
                    logger.success("Cache cleared");
                }
                CacheAction::List => {
                    let entries = cache_manager.list()?;
                    if entries.is_empty() {
                        logger.info("No cache entries found");
                    } else {
                        logger.info("Cache entries:");
                        for entry in entries {
                            logger.output(format!("{} - {}", entry.key, 
                                entry.created_at.format("%Y-%m-%d %H:%M:%S")
                            ));
                        }
                    }
                }
            }
        }
        Commands::Completions { .. } => {
            // This case is handled early in the function
            unreachable!()
        }
        Commands::BuildDistro { .. } => {
            // This case is handled early in the function
            unreachable!()
        }
        Commands::GenerateConfig { .. } => {
            // This case is handled early in the function
            unreachable!()
        }
        Commands::SelfUpdate { .. } => {
            // This case is handled early in the function
            unreachable!()
        }
        Commands::Monitor { metrics, health, history } => {
            let mut monitor = monitoring::SystemMonitor::new();
            
            if health {
                let health_checks = monitor.run_health_checks();
                logger.info("System Health Checks:");
                for check in health_checks {
                    match check.status {
                        monitoring::HealthStatus::Healthy => logger.success(format!("✓ {}: {}", check.name, check.message)),
                        monitoring::HealthStatus::Warning => logger.warn(format!("⚠ {}: {}", check.name, check.message)),
                        monitoring::HealthStatus::Critical => logger.error(format!("✗ {}: {}", check.name, check.message)),
                        monitoring::HealthStatus::Unknown => logger.info(format!("? {}: {}", check.name, check.message)),
                    }
                }
            }
            
            if metrics {
                match monitor.collect_metrics() {
                    Ok(metrics) => {
                        let json = serde_json::to_string_pretty(&metrics)?;
                        logger.json(&json);
                    }
                    Err(e) => logger.error(format!("Failed to collect metrics: {}", e)),
                }
            }
            
            if history {
                let history = monitor.get_history();
                if history.is_empty() {
                    logger.info("No metrics history available");
                } else {
                    logger.info("Metrics History:");
                    for (i, entry) in history.iter().enumerate() {
                        logger.output(format!("[{}] {} - CPU: {:.1}%, Memory: {:.1}GB/{:.1}GB", 
                            i + 1,
                            chrono::DateTime::from_timestamp(entry.timestamp as i64, 0)
                                .unwrap_or_default()
                                .format("%Y-%m-%d %H:%M:%S"),
                            entry.cpu_usage,
                            entry.memory_usage.used as f64 / 1024.0 / 1024.0 / 1024.0,
                            entry.memory_usage.total as f64 / 1024.0 / 1024.0 / 1024.0
                        ));
                    }
                }
            }
            
            // Default: show basic metrics if no specific option provided
            if !metrics && !health && !history {
                match monitor.collect_metrics() {
                    Ok(metrics) => {
                        logger.info("System Metrics:");
                        logger.info(format!("CPU Usage: {:.1}%", metrics.cpu_usage));
                        logger.info(format!("Memory: {:.1}GB/{:.1}GB ({:.1}%)", 
                            metrics.memory_usage.used as f64 / 1024.0 / 1024.0 / 1024.0,
                            metrics.memory_usage.total as f64 / 1024.0 / 1024.0 / 1024.0,
                            (metrics.memory_usage.used as f64 / metrics.memory_usage.total as f64) * 100.0
                        ));
                        logger.info(format!("Load Average: {:.2}, {:.2}, {:.2}", 
                            metrics.load_average.one_min,
                            metrics.load_average.five_min,
                            metrics.load_average.fifteen_min
                        ));
                        logger.info(format!("Uptime: {} days", metrics.uptime.as_secs() / 86400));
                    }
                    Err(e) => logger.error(format!("Failed to collect metrics: {}", e)),
                }
            }
        }
        Commands::Remote { host, command, sudo, test } => {
            let system_config = system_config::SystemConfig::load()?;
            let controller = remote_control::RemoteController::new(system_config.remote);
            
            if test {
                logger.info(format!("Testing connectivity to {}", host));
                match controller.test_connectivity(&host).await {
                    Ok(true) => logger.success(format!("✓ Successfully connected to {}", host)),
                    Ok(false) => logger.error(format!("✗ Failed to connect to {}", host)),
                    Err(e) => logger.error(format!("Connection test failed: {}", e)),
                }
            } else {
                let task = remote_control::RemoteTask {
                    id: "manual-command".to_string(),
                    command: command.clone(),
                    hosts: vec![host.clone()],
                    parallel: false,
                    timeout: Some(std::time::Duration::from_secs(60)),
                    become_root: sudo,
                };
                
                match controller.execute_task(&task).await {
                    Ok(results) => {
                        for result in results {
                            logger.info(format!("Host: {}", result.host));
                            logger.info(format!("Success: {}", result.success));
                            if !result.stdout.is_empty() {
                                logger.output(format!("Output:\n{}", result.stdout));
                            }
                            if !result.stderr.is_empty() {
                                logger.error(format!("Error:\n{}", result.stderr));
                            }
                            logger.info(format!("Duration: {:?}", result.duration));
                        }
                    }
                    Err(e) => logger.error(format!("Failed to execute remote command: {}", e)),
                }
            }
        }
        Commands::SystemConfig { show, sample } => {
            if sample {
                let sample_config = system_config::SystemConfig::generate_sample_config();
                logger.output(sample_config);
            } else if show {
                match system_config::SystemConfig::load() {
                    Ok(config) => {
                        let json = serde_json::to_string_pretty(&config)?;
                        logger.json(&json);
                    }
                    Err(e) => logger.error(format!("Failed to load system configuration: {}", e)),
                }
            } else {
                logger.info("System Configuration Management:");
                logger.info("Use --show to display current configuration");
                logger.info("Use --sample to generate a sample configuration");
            }
        }
        Commands::Wsm { detect, sessions, displays, restart, switch } => {
            let mut wsm = wsm::WindowSystemManager::new();
            
            if detect {
                match wsm.detect_window_system() {
                    Ok(info) => {
                        let json = serde_json::to_string_pretty(&info)?;
                        logger.json(&json);
                    }
                    Err(e) => logger.error(format!("Failed to detect window system: {}", e)),
                }
            } else if sessions {
                match wsm.list_available_sessions() {
                    Ok(sessions) => {
                        logger.info("Available Sessions:");
                        for session in sessions {
                            logger.output(format!("• {}", session));
                        }
                    }
                    Err(e) => logger.error(format!("Failed to list sessions: {}", e)),
                }
            } else if displays {
                match wsm.get_display_configuration() {
                    Ok(config) => {
                        logger.info("Display Configuration:");
                        for (key, value) in config {
                            logger.output(format!("{}: {}", key, value));
                        }
                    }
                    Err(e) => logger.error(format!("Failed to get display configuration: {}", e)),
                }
            } else if let Some(component) = restart {
                if let Some(cmd) = wsm.get_restart_command(&component) {
                    if cmd.requires_root {
                        logger.warn("This command requires root privileges:");
                    }
                    logger.output(format!("To {}: {}", cmd.description, cmd.command));
                } else {
                    logger.error(format!("Unknown component: {}", component));
                }
            } else if let Some(session_type) = switch {
                if let Some(cmd) = wsm.get_switch_session_command(&session_type) {
                    logger.output(format!("To {}: {}", cmd.description, cmd.command));
                } else {
                    logger.error(format!("Unknown session type: {}", session_type));
                }
            } else {
                // Default: detect and show basic window system info
                match wsm.detect_window_system() {
                    Ok(info) => {
                        logger.info("Window System Information:");
                        logger.info(format!("Window System: {:?}", info.window_system));
                        if let Some(de) = &info.desktop_environment {
                            logger.info(format!("Desktop Environment: {:?}", de));
                        }
                        if let Some(dm) = &info.display_manager {
                            logger.info(format!("Display Manager: {:?}", dm));
                        }
                        if let Some(wm) = &info.window_manager {
                            logger.info(format!("Window Manager: {}", wm));
                        }
                        logger.info(format!("Session Type: {:?}", info.session_type));
                        if let Some(compositor) = &info.compositor {
                            logger.info(format!("Compositor: {}", compositor));
                        }
                        logger.info(format!("Displays: {} detected", info.displays.len()));
                    }
                    Err(e) => logger.error(format!("Failed to detect window system: {}", e)),
                }
            }
        }
    }

    Ok(())
}
