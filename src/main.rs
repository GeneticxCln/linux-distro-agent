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
mod agent;
mod self_update;
mod distributed_cache;
mod signing_verification;
mod compatibility_layer;

use clap::{Parser, Subcommand, CommandFactory};
use clap_complete::{generate, Generator, Shell};
use anyhow::Result;
use std::io;
use std::path::PathBuf;
use distro::DistroInfo;
use self_update::{SelfUpdater, UpdateConfig};
use distro_builder::{DistroBuilder, DistroConfig};
use executor::CommandExecutor;
use logger::Logger;
use config::Config;

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
    /// Show distributed cache status
    DistributedStatus,
    /// Clear distributed cache
    DistributedClear,
    /// List distributed cache entries
    DistributedList,
    /// Add entry to distributed cache
    DistributedAdd {
        /// Cache key
        key: String,
        /// Cache value
        value: String,
        /// TTL in seconds
        #[clap(short, long, default_value = "3600")]
        ttl: u64,
    },
    /// Get entry from distributed cache
    DistributedGet {
        /// Cache key
        key: String,
    },
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
        /// Check for updates only, don't install
        #[clap(short, long)]
        check: bool,
        /// Include pre-release versions
        #[clap(long)]
        pre_release: bool,
        /// Update channel (stable, beta, alpha, nightly)
        #[clap(long, default_value = "stable")]
        channel: String,
        /// Show current update configuration
        #[clap(long)]
        config: bool,
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
        /// Enable real-time monitoring with continuous updates
        #[clap(short, long)]
        watch: bool,
        /// Refresh interval in seconds for real-time monitoring
        #[clap(short, long, default_value = "2")]
        interval: u64,
        /// Output format (json, table, plain)
        #[clap(short, long, default_value = "table")]
        format: String,
        /// Filter metrics to display (comma-separated: cpu,memory,disk,network)
        #[clap(long)]
        filter: Option<String>,
        /// Show only critical health issues
        #[clap(long)]
        critical_only: bool,
        /// List available health checks
        #[clap(long)]
        list_checks: bool,
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
    /// AI Agent - Intelligent task planning and execution
    Agent {
        /// Start the intelligent agent loop
        #[clap(short, long)]
        start: bool,
        /// Add a task to the agent queue
        #[clap(long)]
        add_task: Option<String>,
        /// Show agent status and current tasks
        #[clap(long)]
        status: bool,
        /// Show agent learning data and statistics
        #[clap(long)]
        stats: bool,
        /// Clear all tasks from the agent queue
        #[clap(long)]
        clear_tasks: bool,
        /// Enable dry-run mode (tasks won't be executed)
        #[clap(long)]
        dry_run: bool,
    },
    /// Package signing and verification
    Verify {
        /// Path to the package file to verify
        #[clap(short, long)]
        package: Option<PathBuf>,
        /// GPG key ID to verify against
        #[clap(short, long)]
        key_id: Option<String>,
        /// Show signature details
        #[clap(long)]
        details: bool,
        /// Verify repository signatures
        #[clap(long)]
        repo: bool,
        /// Trust level for verification (low, medium, high)
        #[clap(long, default_value = "medium")]
        trust_level: String,
    },
}

fn print_completions<G: Generator>(generator: G, cmd: &mut clap::Command) {
    generate(generator, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

async fn handle_self_update(
    logger: &Logger, 
    force: bool, 
    dry_run: bool, 
    check: bool, 
    pre_release: bool, 
    channel: &str, 
    show_config: bool
) -> Result<()> {
    use self_update::UpdateChannel;
    
    // Parse update channel
    let update_channel = match channel {
        "stable" => UpdateChannel::Stable,
        "beta" => UpdateChannel::Beta,
        "alpha" => UpdateChannel::Alpha,
        "nightly" => UpdateChannel::Nightly,
        _ => {
            logger.error(format!("Invalid update channel: {channel}. Valid options: stable, beta, alpha, nightly"));
            return Ok(());
        }
    };
    
    // Create update configuration
    let config = UpdateConfig {
        pre_release,
        update_channel,
        ..UpdateConfig::default()
    };
    
    if show_config {
        logger.info("📋 Update Configuration:");
        logger.info(format!("  Check Interval: {} hours", config.check_interval));
        logger.info(format!("  Auto Update: {}", config.auto_update));
        logger.info(format!("  Pre-release: {}", config.pre_release));
        logger.info(format!("  Backup Count: {}", config.backup_count));
        logger.info(format!("  Fallback to Source: {}", config.fallback_to_source));
        logger.info(format!("  Update Channel: {:?}", config.update_channel));
        return Ok(());
    }
    
    let updater = SelfUpdater::new(config, logger.clone())?;
    
    if check {
        let update_info = updater.check_for_updates().await?;
        logger.info(format!("📦 Current Version: {}", update_info.current_version));
        logger.info(format!("📦 Latest Version: {}", update_info.latest_version));
        
        if update_info.needs_update {
            logger.info("🔄 Update Available!");
            if let Some(size) = update_info.asset_size {
                logger.info(format!("📏 Download Size: {:.2} MB", size as f64 / 1024.0 / 1024.0));
            }
            if update_info.is_prerelease {
                logger.warn("⚠️  This is a pre-release version");
            }
            
            if !update_info.release_notes.trim().is_empty() {
                logger.info("📝 Release Notes:");
                for line in update_info.release_notes.lines().take(5) {
                    logger.info(format!("   {line}"));
                }
                if update_info.release_notes.lines().count() > 5 {
                    logger.info("   ... (truncated)");
                }
            }
            
            logger.info("💡 Run 'lda self-update' to install the update");
        } else {
            logger.success("✅ You're running the latest version!");
        }
    } else {
        updater.perform_update(force, dry_run).await?;
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
                println!("{toml_string}");
            }
            return Ok(());
        }
        Commands::SelfUpdate { force, dry_run, check, pre_release, channel, config } => {
            return handle_self_update(&logger, *force, *dry_run, *check, *pre_release, channel, *config).await;
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
            
            // Try config-based package manager detection as fallback/confirmation
            let config = Config::load().unwrap_or_else(|_| {
                logger.verbose("Using default configuration for package manager detection");
                Config::default()
            });
            
            if let Some(id) = &distro.id {
let id_like_str = distro.id_like.as_deref();
                if let Some(config_pm) = config.detect_package_manager(id, id_like_str) {
                    match &distro.package_manager {
                        Some(detected_pm) if detected_pm != &config_pm => {
                            logger.info(format!("Config-based Package Manager: {config_pm} (differs from detected: {detected_pm})"));
                        }
                        None => {
                            logger.info(format!("Config-based Package Manager: {config_pm}"));
                        }
                        _ => {
                            logger.verbose(format!("Config confirms package manager: {config_pm}"));
                        }
                    }
                }
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
                    // Show both config_manager config and legacy config
                    let config = config_manager.load()?;
                    let json = serde_json::to_string_pretty(&config)?;
                    logger.output("Configuration Manager Settings:");
                    logger.output(json);
                    
                    // Also show legacy config if it exists
                    if let Ok(legacy_config) = config::Config::load() {
                        let legacy_json = serde_json::to_string_pretty(&legacy_config)?;
                        logger.output("\nLegacy Configuration:");
                        logger.output(legacy_json);
                    }
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
let mut distributed_cache = distributed_cache::PackageCache::new(std::time::Duration::from_secs(3600));
            
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
                CacheAction::DistributedStatus => {
                    let status = distributed_cache.get_cache_stats();
                    logger.info("Distributed Cache Status:");
                    logger.info(format!("Total entries: {}", status.total_entries));
logger.info(format!("Total size: {} bytes", status.total_size_bytes));
let hit_rate = status.hit_count as f64 / (status.hit_count + status.miss_count) as f64;
logger.info(format!("Hit rate: {:.1}%", hit_rate * 100.0));
                    logger.info(format!("Last cleanup: {:?}", status.last_cleanup));
                }
                CacheAction::DistributedClear => {
                    distributed_cache.clear_cache();
                    logger.success("Distributed cache cleared");
                }
                CacheAction::DistributedList => {
                    let entries = distributed_cache.list_entries();
                    if entries.is_empty() {
                        logger.info("No distributed cache entries found");
                    } else {
                        logger.info("Distributed cache entries:");
                        for key in entries {
                            logger.output(format!("Key: {key}"));
                        }
                    }
                }
                CacheAction::DistributedAdd { key, value, ttl } => {
                    let _ttl_duration = std::time::Duration::from_secs(ttl);
                    let key_clone = key.clone();
                    distributed_cache.store(key, value.into_bytes());
                    logger.success(format!("Added entry '{key_clone}' to distributed cache with TTL of {ttl} seconds"));
                }
                CacheAction::DistributedGet { key } => {
                    match distributed_cache.retrieve(&key) {
                        Some(value) => {
match String::from_utf8(value.to_vec()) {
                                Ok(string_value) => logger.output(format!("Value for '{key}': {string_value}")),
                                Err(_) => logger.output(format!("Value for '{key}': <binary data>")),
                            }
                        }
                        None => logger.info(format!("No value found for key '{key}'")),
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
        Commands::Monitor { metrics, health, history, watch, interval, format, filter, critical_only, list_checks } => {
            let mut monitor = monitoring::SystemMonitor::new();
            
            // List available health checks
            if list_checks {
                logger.info("Available Health Checks:");
                logger.info("• disk_usage - Monitor disk space usage");
                logger.info("• memory_usage - Monitor memory usage");
                logger.info("• load_average - Monitor system load");
                logger.info("• process_count - Monitor running processes");
                return Ok(());
            }
            
            // Helper function to format metrics output
            let format_metrics_output = |metrics: &monitoring::SystemMetrics, format: &str, filter: &Option<String>| -> String {
                let mut filtered_metrics = metrics.clone();
                
                // Apply filter if specified
                if let Some(filter_str) = filter {
                    let filters: Vec<&str> = filter_str.split(',').map(|s| s.trim()).collect();
                    if !filters.contains(&"cpu") {
                        filtered_metrics.cpu_usage = 0.0;
                    }
                    // Note: Complete filtering would require modifying SystemMetrics structure
                    // This is a simplified version showing the concept
                }
                
                match format {
                    "json" => serde_json::to_string_pretty(&filtered_metrics).unwrap_or_else(|_| "Error formatting JSON".to_string()),
                    "csv" => {
                        format!("timestamp,cpu_usage,memory_used,memory_total,load_1m,load_5m,load_15m,uptime\n{},{:.1},{},{},{:.2},{:.2},{:.2},{}",
                            chrono::Utc::now().timestamp(),
                            filtered_metrics.cpu_usage,
                            filtered_metrics.memory_usage.used,
                            filtered_metrics.memory_usage.total,
                            filtered_metrics.load_average.one_min,
                            filtered_metrics.load_average.five_min,
                            filtered_metrics.load_average.fifteen_min,
                            filtered_metrics.uptime.as_secs()
                        )
                    },
                    "plain" => {
                        format!("CPU: {:.1}% | Memory: {:.1}GB/{:.1}GB ({:.1}%) | Load: {:.2}, {:.2}, {:.2} | Uptime: {} days",
                            filtered_metrics.cpu_usage,
                            filtered_metrics.memory_usage.used as f64 / 1024.0 / 1024.0 / 1024.0,
                            filtered_metrics.memory_usage.total as f64 / 1024.0 / 1024.0 / 1024.0,
                            (filtered_metrics.memory_usage.used as f64 / filtered_metrics.memory_usage.total as f64) * 100.0,
                            filtered_metrics.load_average.one_min,
                            filtered_metrics.load_average.five_min,
                            filtered_metrics.load_average.fifteen_min,
                            filtered_metrics.uptime.as_secs() / 86400
                        )
                    },
                    _ => { // "table" format (default)
                        let mut output = String::new();
                        output.push_str("┌──────────────────┬─────────────────┐\n");
                        output.push_str("│ Metric           │ Value           │\n");
                        output.push_str("├──────────────────┼─────────────────┤\n");
                        output.push_str(&format!("│ CPU Usage        │ {:>13.1}% │\n", filtered_metrics.cpu_usage));
                        output.push_str(&format!("│ Memory Used      │ {:>11.1} GB │\n", filtered_metrics.memory_usage.used as f64 / 1024.0 / 1024.0 / 1024.0));
                        output.push_str(&format!("│ Memory Total     │ {:>11.1} GB │\n", filtered_metrics.memory_usage.total as f64 / 1024.0 / 1024.0 / 1024.0));
                        output.push_str(&format!("│ Load (1m)        │ {:>15.2} │\n", filtered_metrics.load_average.one_min));
                        output.push_str(&format!("│ Load (5m)        │ {:>15.2} │\n", filtered_metrics.load_average.five_min));
                        output.push_str(&format!("│ Load (15m)       │ {:>15.2} │\n", filtered_metrics.load_average.fifteen_min));
                        output.push_str(&format!("│ Uptime           │ {:>11} days │\n", filtered_metrics.uptime.as_secs() / 86400));
                        output.push_str("└──────────────────┴─────────────────┘");
                        output
                    }
                }
            };
            
            // Handle real-time monitoring with watch mode
            if watch {
                logger.info(&format!("📊 Real-time monitoring started (refresh every {}s). Press Ctrl+C to stop...", interval));
                
                // Set up Ctrl+C handler
                let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
                let r = running.clone();
                ctrlc::set_handler(move || {
                    r.store(false, std::sync::atomic::Ordering::SeqCst);
                }).expect("Error setting Ctrl+C handler");
                
                while running.load(std::sync::atomic::Ordering::SeqCst) {
                    // Clear screen for better real-time display
                    print!("\x1B[2J\x1B[1;1H");
                    
                    match monitor.collect_metrics() {
                        Ok(metrics) => {
                            let output = format_metrics_output(&metrics, &format[..], &filter);
                            println!("{}", output);
                            println!("\nLast updated: {} | Press Ctrl+C to stop", chrono::Local::now().format("%H:%M:%S"));
                        }
                        Err(e) => logger.error(&format!("Failed to collect metrics: {}", e)),
                    }
                    
                    std::thread::sleep(std::time::Duration::from_secs(interval));
                }
                
                logger.info("\n📊 Real-time monitoring stopped.");
                return Ok(());
            }
            
            // Handle health checks with optional critical-only filter
            if health {
                let health_checks = monitor.run_health_checks();
                logger.info("System Health Checks:");
                for check in health_checks {
                    let should_show = if critical_only {
                        matches!(check.status, monitoring::HealthStatus::Critical)
                    } else {
                        true
                    };
                    
                    if should_show {
                        match check.status {
                            monitoring::HealthStatus::Healthy => logger.success(&format!("✓ {}: {}", check.name, check.message)),
                            monitoring::HealthStatus::Warning => logger.warn(&format!("⚠ {}: {}", check.name, check.message)),
                            monitoring::HealthStatus::Critical => logger.error(&format!("✗ {}: {}", check.name, check.message)),
                            monitoring::HealthStatus::Unknown => logger.info(&format!("? {}: {}", check.name, check.message)),
                        }
                    }
                }
            }
            
            // Handle metrics display with formatting
            if metrics {
                match monitor.collect_metrics() {
                    Ok(metrics) => {
                        let output = format_metrics_output(&metrics, &format[..], &filter);
                        if format == "json" {
                            logger.json(&output);
                        } else {
                            logger.output(&output);
                        }
                    }
                    Err(e) => logger.error(&format!("Failed to collect metrics: {}", e)),
                }
            }
            
            // Handle history display
            if history {
                let history = monitor.get_history();
                if history.is_empty() {
                    logger.info("No metrics history available");
                } else {
                    match &format[..] {
                        "json" => {
                            let json = serde_json::to_string_pretty(&history)?;
                            logger.json(&json);
                        }
                        "csv" => {
                            logger.output("timestamp,cpu_usage,memory_used,memory_total,load_1m,load_5m,load_15m");
                            for entry in history.iter() {
                                logger.output(&format!("{},{:.1},{},{},{:.2},{:.2},{:.2}",
                                    entry.timestamp,
                                    entry.cpu_usage,
                                    entry.memory_usage.used,
                                    entry.memory_usage.total,
                                    entry.load_average.one_min,
                                    entry.load_average.five_min,
                                    entry.load_average.fifteen_min
                                ));
                            }
                        }
                        _ => {
                            logger.info("Metrics History:");
                            for (i, entry) in history.iter().enumerate() {
                                logger.output(&format!("[{}] {} - CPU: {:.1}%, Memory: {:.1}GB/{:.1}GB", 
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
                }
            }
            
            // Default: show basic metrics if no specific option provided
            if !metrics && !health && !history && !watch && !list_checks {
                // Try to get latest cached metrics first, then collect new ones
                let metrics_to_display = if let Some(cached_metrics) = monitor.get_latest_metrics() {
                    cached_metrics.clone()
                } else {
                    monitor.collect_metrics()?
                };
                
                let output = format_metrics_output(&metrics_to_display, &format[..], &filter);
                if format == "json" {
                    logger.json(&output);
                } else {
                    logger.output(&output);
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
        Commands::Security { audit, json, severity, category } => {
            let mut security_auditor = security::SecurityAuditor::new();

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
                    Err(e) => logger.error(format!("Security audit failed: {}", e)),
                }
            }

            if json {
                logger.info("Outputting security report in JSON format...");
                match security_auditor.get_security_report_json() {
                    Ok(json_report) => logger.output(json_report),
                    Err(e) => logger.error(format!("Failed to generate JSON report: {}", e)),
                }
            }
        }
        Commands::Plugin { list, info, enable, disable, exec, args, install, uninstall, create, plugin_type } => {
            let mut plugin_manager = plugins::PluginManager::new()?;

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
                    None => logger.error(format!("Plugin '{}' not found", plugin_name)),
                }
            }

            if let Some(plugin_name) = enable {
                match plugin_manager.enable_plugin(&plugin_name) {
                    Ok(()) => logger.success(format!("Plugin '{}' enabled", plugin_name)),
                    Err(e) => logger.error(format!("Failed to enable plugin '{}': {}", plugin_name, e)),
                }
            }

            if let Some(plugin_name) = disable {
                match plugin_manager.disable_plugin(&plugin_name) {
                    Ok(()) => logger.success(format!("Plugin '{}' disabled", plugin_name)),
                    Err(e) => logger.error(format!("Failed to disable plugin '{}': {}", plugin_name, e)),
                }
            }

            if let Some(plugin_name) = exec {
                match plugin_manager.execute_plugin(&plugin_name, &args) {
                    Ok(output) => logger.output(output),
                    Err(e) => logger.error(format!("Failed to execute plugin '{}': {}", plugin_name, e)),
                }
            }

            if let Some(plugin_path) = install {
                match plugin_manager.install_plugin(&plugin_path) {
                    Ok(()) => logger.success(format!("Plugin installed from '{}'", plugin_path.display())),
                    Err(e) => logger.error(format!("Failed to install plugin: {}", e)),
                }
            }

            if let Some(plugin_name) = uninstall {
                match plugin_manager.uninstall_plugin(&plugin_name) {
                    Ok(()) => logger.success(format!("Plugin '{}' uninstalled", plugin_name)),
                    Err(e) => logger.error(format!("Failed to uninstall plugin '{}': {}", plugin_name, e)),
                }
            }

            if let Some(plugin_name) = create {
                let plugin_type_enum = match plugin_type.as_str() {
                    "command" => plugins::PluginType::Command,
                    "monitor" => plugins::PluginType::Monitor,
                    "security" => plugins::PluginType::Security,
                    "package" => plugins::PluginType::PackageManager,
                    "distro" => plugins::PluginType::Distro,
                    "integration" => plugins::PluginType::Integration,
                    _ => {
                        logger.error(format!("Unknown plugin type: {}. Available types: command, monitor, security, package, distro, integration", plugin_type));
                        return Ok(());
                    }
                };
                
                match plugin_manager.create_plugin_template(&plugin_name, plugin_type_enum) {
                    Ok(template_path) => logger.success(format!("Plugin template created at: {}", template_path.display())),
                    Err(e) => logger.error(format!("Failed to create plugin template: {}", e)),
                }
            }
        }
        Commands::Agent { start, add_task, status, stats, clear_tasks, dry_run } => {
            let mut agent = agent::IntelligentAgent::new(cli.verbose, cli.quiet);
            
            if start {
                logger.info("🤖 Starting Intelligent Agent...");
                match agent.run_agent_loop().await {
                    Ok(()) => logger.success("Agent loop completed successfully"),
                    Err(e) => logger.error(format!("Agent loop failed: {}", e)),
                }
            } else if let Some(task_description) = add_task {
                // Parse task type from description
                let task_type = if task_description.contains("install") || task_description.contains("remove") {
                    agent::TaskType::PackageManagement
                } else if task_description.contains("security") || task_description.contains("audit") {
                    agent::TaskType::SecurityAudit
                } else if task_description.contains("config") {
                    agent::TaskType::SystemConfiguration
                } else if task_description.contains("monitor") {
                    agent::TaskType::Monitoring
                } else {
                    agent::TaskType::PackageManagement
                };
                
                let task = agent.create_task_from_command(
                    "manual",
                    &[task_description],
                    task_type
                );
                
                match agent.add_task(task) {
                    Ok(()) => logger.success("Task added to agent queue"),
                    Err(e) => logger.error(format!("Failed to add task: {}", e)),
                }
            } else if status {
                let state = agent.get_state();
                logger.info("🤖 Agent Status:");
                logger.info(format!("Current tasks: {}", state.current_tasks.len()));
                logger.info(format!("Completed tasks: {}", state.completed_tasks.len()));
                logger.info(format!("Failed tasks: {}", state.failed_tasks.len()));
                logger.info(format!("Safety violations: {}", state.safety_violations));
                logger.info(format!("Last update: {:?}", state.last_update));
                
                if !state.current_tasks.is_empty() {
                    logger.info("\nCurrent Tasks:");
                    for task in &state.current_tasks {
                        logger.info(format!("  • {} (Priority: {:?}, Safety: {:?})", 
                            task.description, task.priority, task.safety_level));
                    }
                }
            } else if stats {
                let state = agent.get_state();
                let total_tasks = state.completed_tasks.len() + state.failed_tasks.len();
                let success_rate = if total_tasks > 0 {
                    (state.completed_tasks.len() as f64 / total_tasks as f64) * 100.0
                } else {
                    0.0
                };
                
                let avg_duration = if !state.completed_tasks.is_empty() {
                    let total_duration: f64 = state.completed_tasks.iter()
                        .map(|r| r.duration.as_secs() as f64)
                        .sum();
                    total_duration / state.completed_tasks.len() as f64
                } else {
                    0.0
                };
                
                logger.info("📊 Agent Learning Statistics:");
                logger.info(format!("Success rate: {:.1}%", success_rate));
                logger.info(format!("Average task duration: {:.1}s", avg_duration));
                logger.info(format!("Safety violations: {}", state.safety_violations));
                logger.info(format!("Learning data points: {}", state.learning_data.len()));
                
                if !state.learning_data.is_empty() {
                    logger.info("\nLearning Data:");
                    for (key, value) in &state.learning_data {
                        logger.info(format!("  • {}: {:.3}", key, value));
                    }
                }
            } else if clear_tasks {
                agent.clear_all_tasks();
                logger.success("All tasks cleared from agent queue");
            } else {
                logger.info("🤖 Intelligent Agent System");
                logger.info("Use --start to begin the agent loop");
                logger.info("Use --status to show current agent status");
                logger.info("Use --stats to show learning statistics");
                logger.info("Use --add-task \"command\" to add a task");
                logger.info("Use --clear-tasks to clear all pending tasks");
                logger.info("Use --dry-run to enable dry-run mode");
            }
        }
        Commands::Verify { package, key_id: _, details, repo, trust_level: _ } => {
            use signing_verification::SigningVerificationManager;
            
            let config_dir = dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("linux-distro-agent");
            
            let manager = match SigningVerificationManager::new(&config_dir) {
                Ok(m) => m,
                Err(e) => {
                    logger.error(format!("Failed to initialize signing verification: {}", e));
                    return Ok(());
                }
            };
            
            if let Some(package_path) = package {
                match manager.get_signing_status(&package_path) {
                    Ok(status) => {
                        logger.info(format!("Package verification result: {}", status));
                        
                        if details {
                            match manager.verify_package_signature(&package_path, None) {
                                Ok(sig_info) => {
                                    logger.info(format!("Signature Type: {:?}", sig_info.signature_type));
                                    logger.info(format!("Key ID: {}", sig_info.key_id));
                                    logger.info(format!("Fingerprint: {}", sig_info.fingerprint));
                                    logger.info(format!("Valid: {}", sig_info.valid));
                                    logger.info(format!("Trust Level: {:?}", sig_info.trust_level));
                                    logger.info(format!("Timestamp: {}", sig_info.timestamp));
                                }
                                Err(e) => logger.error(format!("Failed to get signature details: {}", e)),
                            }
                        }
                    }
                    Err(e) => logger.error(format!("Package verification failed: {}", e)),
                }
            } else if repo {
                logger.info("Repository signature verification not yet implemented");
            } else {
                logger.info("Package Signing and Verification:");
                logger.info("Use --package <path> to verify a package signature");
                logger.info("Use --details for detailed signature information");
                logger.info("Use --repo to verify repository signatures");
            }
        }
    }

    Ok(())
}
