mod distro;
mod config;
mod executor;
mod logger;

use clap::{Parser, Subcommand, CommandFactory};
use clap_complete::{generate, Generator, Shell};
use anyhow::Result;
use std::io;
use distro::DistroInfo;
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
    /// Generate shell completion scripts
    Completions {
        /// The shell to generate completions for
        #[clap(value_enum)]
        shell: Shell,
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
        Commands::Completions { .. } => {
            // This case is handled early in the function
            unreachable!()
        }
    }

    Ok(())
}
