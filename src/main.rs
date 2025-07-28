mod config;
mod config_manager;
mod distro;
mod distro_builder;
mod executor;
mod logger;
mod history;
mod cache;
mod system_config;
mod monitoring;
mod wsm;
mod service_manager;
mod remote_control;
mod package_manager;
mod system_logger;
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
use monitoring::SystemMonitor;
use wsm::WindowSystemManager;
use service_manager::ServiceManager;
use remote_control::{RemoteController, RemoteTask};
use system_config::SystemConfig;
use security::SecurityAuditor;
use plugins::PluginManager;

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
    /// System monitoring and metrics
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
    /// Window system management
    WindowSystem {
        /// Detect window system information
        #[clap(short, long)]
        detect: bool,
        /// List available sessions
        #[clap(long)]
        sessions: bool,
        /// Get display configuration
        #[clap(long)]
        displays: bool,
    },
    /// Service management
    Services {
        /// List services (optionally filtered)
        #[clap(short, long)]
        list: bool,
        /// Show failed services
        #[clap(long)]
        failed: bool,
        /// Show active services
        #[clap(long)]
        active: bool,
        /// Service name for specific operations
        #[clap(long)]
        service: Option<String>,
        /// Service action (start, stop, restart, enable, disable, status)
        #[clap(long)]
        action: Option<String>,
        /// Show top processes
        #[clap(long)]
        top: bool,
        /// Number of items to show
        #[clap(long, default_value = "10")]
        limit: usize,
    },
    /// Remote control operations
    Remote {
        /// Add a remote host
        #[clap(long)]
        add_host: Option<String>,
        /// Test connectivity to a host
        #[clap(long)]
        test: Option<String>,
        /// Execute command on remote hosts
        #[clap(long)]
        execute: Option<String>,
        /// Hosts to target (comma separated)
        #[clap(long)]
        hosts: Option<String>,
        /// Run in parallel
        #[clap(long)]
        parallel: bool,
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

fn main() -> Result<()> {
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
            let mut monitor = SystemMonitor::new();
            
            if metrics {
                match monitor.collect_metrics() {
                    Ok(metrics) => {
                        let json = serde_json::to_string_pretty(&metrics)?;
                        logger.json(&json);
                    }
                    Err(e) => logger.error(&format!("Failed to collect metrics: {}", e)),
                }
            }
            
            if health {
                let health_checks = monitor.run_health_checks();
                logger.info("System Health Checks:");
                for check in health_checks {
                    let status_symbol = match check.status {
                        monitoring::HealthStatus::Healthy => "✓",
                        monitoring::HealthStatus::Warning => "⚠",
                        monitoring::HealthStatus::Critical => "✗",
                        monitoring::HealthStatus::Unknown => "?",
                    };
                    logger.info(format!("{} {}: {}", status_symbol, check.name, check.message));
                }
            }
            
            if history {
                let metrics_history = monitor.get_history();
                if metrics_history.is_empty() {
                    logger.info("No metrics history available. Run with --metrics first.");
                } else {
                    logger.info("Metrics History:");
                    for metric in metrics_history {
                        logger.info(format!("Timestamp: {}, CPU: {:.1}%, Memory: {:.1}%", 
                            metric.timestamp,
                            metric.cpu_usage,
                            if metric.memory_usage.total > 0 {
                                (metric.memory_usage.used as f64 / metric.memory_usage.total as f64) * 100.0
                            } else { 0.0 }
                        ));
                    }
                }
            }
        }
        Commands::WindowSystem { detect, sessions, displays } => {
            let mut wsm = WindowSystemManager::new();
            
            if detect {
                match wsm.detect_window_system() {
                    Ok(info) => {
                        let json = serde_json::to_string_pretty(&info)?;
                        logger.json(&json);
                    }
                    Err(e) => logger.error(&format!("Failed to detect window system: {}", e)),
                }
            }
            
            if sessions {
                match wsm.list_available_sessions() {
                    Ok(sessions) => {
                        logger.info("Available Sessions:");
                        for session in sessions {
                            logger.info(format!("• {}", session));
                        }
                    }
                    Err(e) => logger.error(&format!("Failed to list sessions: {}", e)),
                }
            }
            
            if displays {
                match wsm.get_display_configuration() {
                    Ok(config) => {
                        logger.info("Display Configuration:");
                        for (key, value) in config {
                            logger.info(format!("{}: {}", key, value));
                        }
                    }
                    Err(e) => logger.error(&format!("Failed to get display configuration: {}", e)),
                }
            }
        }
        Commands::Services { list, failed, active, service, action, top, limit } => {
            let mut service_manager = ServiceManager::new();
            
            if list {
                match service_manager.list_services(None) {
                    Ok(services) => {
                        logger.info("System Services:");
                        for svc in services.iter().take(limit) {
                            let status = match svc.status {
                                service_manager::ServiceStatus::Active => "✓ Active",
                                service_manager::ServiceStatus::Inactive => "○ Inactive",
                                service_manager::ServiceStatus::Failed => "✗ Failed",
                                _ => "? Unknown",
                            };
                            logger.info(format!("{:<30} {:<15} {}", svc.name, status, svc.description));
                        }
                    }
                    Err(e) => logger.error(&format!("Failed to list services: {}", e)),
                }
            }
            
            if failed {
                match service_manager.list_failed_services() {
                    Ok(services) => {
                        logger.info("Failed Services:");
                        for svc in services {
                            logger.error(format!("✗ {}: {}", svc.name, svc.description));
                        }
                    }
                    Err(e) => logger.error(&format!("Failed to list failed services: {}", e)),
                }
            }
            
            if active {
                match service_manager.list_active_services() {
                    Ok(services) => {
                        logger.info("Active Services:");
                        for svc in services.iter().take(limit) {
                            logger.success(format!("✓ {}: {}", svc.name, svc.description));
                        }
                    }
                    Err(e) => logger.error(&format!("Failed to list active services: {}", e)),
                }
            }
            
            if let (Some(svc_name), Some(svc_action)) = (service, action) {
                if let Some(cmd) = service_manager.get_service_command(&svc_action, &svc_name) {
                    logger.output(format!("To {} '{}', run: {}", svc_action, svc_name, cmd.command));
                } else {
                    logger.error(&format!("Unknown action '{}' for service '{}'", svc_action, svc_name));
                }
            }
            
            if top {
                match service_manager.get_top_processes(limit) {
                    Ok(processes) => {
                        logger.info("Top Processes:");
                        logger.info(format!("{:<10} {:<15} {:<8} {:<8} {}", "PID", "USER", "CPU%", "MEM%", "COMMAND"));
                        for proc in processes {
                            logger.info(format!("{:<10} {:<15} {:<8.1} {:<8.1} {}", 
                                proc.pid, proc.user, proc.cpu_percent, proc.memory_percent, proc.command));
                        }
                    }
                    Err(e) => logger.error(&format!("Failed to get top processes: {}", e)),
                }
            }
        }
        Commands::Remote { add_host, test, execute, hosts, parallel } => {
            let system_config = SystemConfig::load()?;
            let remote_controller = RemoteController::new(system_config.remote);
            
            if let Some(host_name) = add_host {
                logger.info(format!("To add host '{}', edit your hosts configuration file:", host_name));
                logger.info(RemoteController::generate_inventory_template());
            }
            
            if let Some(host_name) = test {
                let rt = tokio::runtime::Runtime::new()?;
                match rt.block_on(remote_controller.test_connectivity(&host_name)) {
                    Ok(success) => {
                        if success {
                            logger.success(format!("✓ Connection to '{}' successful", host_name));
                        } else {
                            logger.error(format!("✗ Connection to '{}' failed", host_name));
                        }
                    }
                    Err(e) => logger.error(&format!("Connection test failed: {}", e)),
                }
            }
            
            if let (Some(command), Some(target_hosts)) = (execute, hosts) {
                let host_list: Vec<String> = target_hosts.split(',').map(|s| s.trim().to_string()).collect();
                let task = RemoteTask {
                    id: "manual-execution".to_string(),
                    command: command.clone(),
                    hosts: host_list,
                    parallel,
                    timeout: Some(std::time::Duration::from_secs(300)),
                    become_root: false,
                };
                
                let rt = tokio::runtime::Runtime::new()?;
                match rt.block_on(remote_controller.execute_task(&task)) {
                    Ok(results) => {
                        logger.info("Remote Execution Results:");
                        for result in results {
                            if result.success {
                                logger.success(format!("✓ {}: Command completed successfully", result.host));
                                if !result.stdout.is_empty() {
                                    logger.info(format!("Output: {}", result.stdout.trim()));
                                }
                            } else {
                                logger.error(format!("✗ {}: Command failed (exit code: {:?})", result.host, result.exit_code));
                                if !result.stderr.is_empty() {
                                    logger.error(format!("Error: {}", result.stderr.trim()));
                                }
                            }
                        }
                    }
                    Err(e) => logger.error(&format!("Remote execution failed: {}", e)),
                }
            }
        }
        Commands::Security { audit, json, severity, category } => {
            let mut security_auditor = SecurityAuditor::new();

            if audit {
                logger.info("Running full security audit...");
                match security_auditor.run_full_audit() {
                    Ok(audit_result) => {
                        logger.info("Security Audit Results:");
                        logger.info(format!("Total Issues: {}", audit_result.findings.len()));
                        let filtered_issues = audit_result.filter_by(severity, category);
                        for issue in &filtered_issues {
                            logger.info(format!("[{}] {} - {}", issue.severity, issue.category, issue.description));
                        }
                    }
                    Err(e) => logger.error(&format!("Security audit failed: {}", e)),
                }
            }

            if json {
                logger.info("Outputting security report in JSON format...");
                match security_auditor.get_security_report_json() {
                    Ok(json_report) => logger.output(json_report),
                    Err(e) => logger.error(&format!("Failed to generate JSON report: {}", e)),
                }
            }
        }
        Commands::Plugin { list, info, enable, disable, exec, args, install, uninstall, create, plugin_type: _ } => {
            let plugin_manager = PluginManager::new()?;

            if list {
                logger.info("Available plugins:");
                let plugins = plugin_manager.list_plugins();
                for plugin in plugins {
                    let status = if plugin.config.enabled { "✓ Enabled" } else { "○ Disabled" };
                    logger.info(format!("{:<20} {:<10} {} - {}", plugin.metadata.name, plugin.metadata.version, status, plugin.metadata.description));
                }
            }

            if let Some(plugin_name) = info {
                logger.info(format!("Retrieving information for plugin: {}", plugin_name));
                match plugin_manager.get_plugin(&plugin_name) {
                    Some(info) => logger.info(format!("Plugin Info: Name: {}, Version: {}, Enabled: {}", info.metadata.name, info.metadata.version, info.config.enabled)),
                    None => logger.error(&format!("Plugin '{}' not found", plugin_name)),
                }
            }

            if let Some(_plugin_name) = enable { /* Implement enable plugin logic */ }
            if let Some(_plugin_name) = disable { /* Implement disable plugin logic */ }
            if let Some(_exec_name) = exec { /* Implement execute plugin logic */ }
            if !args.is_empty() { /* Implement plugin argument passing */ }
            if let Some(_dir) = install { /* Implement install plugin logic */ }
            if let Some(_plugin_name) = uninstall { /* Implement uninstall plugin logic */ }
            if let Some(_name) = create { /* Implement create plugin template logic */ }
        }
    }

    Ok(())
}
