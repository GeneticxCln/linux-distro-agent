mod config;
mod config_manager;
mod distro;
mod distro_builder;
mod executor;
mod logger;
mod history;
mod cache;

use clap::{Parser, Subcommand, CommandFactory};
use clap_complete::{generate, Generator, Shell};
use anyhow::Result;
use std::io;
use std::path::PathBuf;
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
}

fn print_completions<G: Generator>(generator: G, cmd: &mut clap::Command) {
    generate(generator, cmd, cmd.get_name().to_string(), &mut io::stdout());
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
    }

    Ok(())
}
