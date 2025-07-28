# 🐧 Linux Distribution Agent (LDA) v4.6.0

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-4.6.0-blue.svg)]()
[![Rust](https://img.shields.io/badge/rust-2024-red.svg)](https://www.rust-lang.org/)

**One tool to rule them all - manage packages on any Linux distro with the same commands! 🚀**

Tired of remembering `apt install` vs `pacman -S` vs `dnf install`? LDA is your friend!
It detects your distro and gives you the right commands, every time.

✨ **Why you'll love it:**
- 🎯 Works on **any** Linux distro (Ubuntu, Arch, Fedora, openSUSE...)
- ⚡ Blazing fast (written in Rust)
- 🧠 Smart auto-detection
- 🤖 **NEW!** AI agent for intelligent task automation
- 🏗️ **NEW!** Build your own custom Linux distro

[🚀 Quick Start](#-quick-start) • [📖 Documentation](#-table-of-contents) • [💻 Demo](#-demo) • [🤝 Contributing](#-contributing)

## 🚀 Features

### 📦 **Package Management**
- **Unified Commands**: Works across all major Linux distributions
- **Auto-Detection**: Automatically detects your distribution and package manager
- **Direct Execution**: Execute commands directly or get the command string
- **Package Search**: Search packages across different repositories
- **Package Information**: Get detailed information about packages

### 🏗️ **Linux Distribution Builder** ⭐ **NEW**
- **Custom ISO Creation**: Build complete bootable Linux distributions
- **Multiple Base Systems**: Support for Arch, Debian, Ubuntu, and scratch builds
- **Desktop Environments**: GNOME, KDE, XFCE, LXDE, Mate, Cinnamon, Sway, i3
- **Kernel Options**: Vanilla, LTS, Hardened, Real-time, or custom kernels
- **Bootloader Support**: Syslinux, GRUB, systemd-boot, rEFInd
- **Custom Branding**: Logos, wallpapers, themes, and color schemes
- **Compression Options**: Gzip, XZ, Zstd, LZ4, or no compression
- **Configuration Templates**: Generate and customize build configurations

### 🔧 **System Management**
- **Distribution Detection**: Comprehensive system information gathering
- **Compatibility Check**: System compatibility analysis and recommendations
- **Command History**: Track and search through command history
- **Configuration Management**: Manage tool settings and preferences
- **Cache System**: Efficient caching for improved performance
- **Shell Completions**: Support for bash, zsh, fish, and PowerShell

### 🆕 **New in v4.6.0** ⭐
- **🔒 Security Auditing**: Comprehensive security analysis and system hardening recommendations
- **🔌 Plugin System**: Extensible plugin architecture for custom functionality
- **📊 System Monitoring**: Real-time system metrics, health checks, and performance monitoring
- **🌐 Remote Control**: Manage remote Linux systems and execute commands remotely
- **🪟 Window System Manager (WSM)**: Manage display managers, window systems, and desktop environments
- **⚙️ System Configuration**: Advanced system configuration management and automation
- **📜 Enhanced Logging**: Comprehensive system activity logging and audit trails
- **🎯 Self-Update**: Built-in self-update mechanism to stay current with latest features
- **🤖 AI Agent System**: Intelligent task planning, execution safety, adaptive learning, and smart automation with comprehensive dry-run capabilities
- **🔧 Fixed Self-Update "Text file busy" Issue**: Resolved critical issue where self-updates would fail with "Text file busy" errors

- **🔍 Distribution Detection**: Automatically detects your Linux distribution from `/etc/os-release`
- **📦 Package Manager Support**: Supports major package managers including:
  - `pacman` (Arch Linux, CachyOS, Manjaro, EndeavourOS)
  - `apt` (Ubuntu, Debian, Pop!_OS, Elementary OS)
  - `dnf` (Fedora, RHEL, CentOS, Rocky Linux, AlmaLinux)
  - `zypper` (openSUSE)
  - `portage` (Gentoo)
  - `nix` (NixOS)
  - `apk` (Alpine Linux)
- **⚡ Command Generation**: Generates appropriate commands for package installation, searching, and system updates
- **📊 JSON Output**: Comprehensive system information export in JSON format
- **🗂️ History Management**: Track and search command history
- **⚙️ Configuration Management**: Customizable settings with TOML configuration
- **💾 Caching System**: Intelligent caching for improved performance
- **🔧 Shell Completions**: Support for bash, zsh, fish, PowerShell, and elvish

## 💻 Demo

### Interactive Demo

```bash
# Try these commands to see Linux Distribution Agent in action:

# 1. Detect your current distribution
linux-distro-agent detect --extended

# 2. Get installation commands for popular packages
linux-distro-agent install git
linux-distro-agent install docker
linux-distro-agent install neovim

# 3. Search for development tools
linux-distro-agent search python
linux-distro-agent search rust

# 4. Get detailed system information
linux-distro-agent info --pretty

# 5. Check system compatibility
linux-distro-agent doctor
```

### Sample Output

<details>
<summary>🎯 Click to see example output on different distributions</summary>

#### Ubuntu/Debian Example
```bash
$ linux-distro-agent detect
Detected Linux distribution: Ubuntu 22.04.3 LTS
Package Manager: apt

$ linux-distro-agent install docker
To install 'docker', run: sudo apt update && sudo apt install docker.io
```

#### Arch Linux Example
```bash
$ linux-distro-agent detect
Detected Linux distribution: Arch Linux
Package Manager: pacman

$ linux-distro-agent install docker
To install 'docker', run: sudo pacman -S docker
```

#### Fedora Example
```bash
$ linux-distro-agent detect
Detected Linux distribution: Fedora Linux 38
Package Manager: dnf

$ linux-distro-agent install docker
To install 'docker', run: sudo dnf install docker
```

</details>

## 📋 Table of Contents

- [What is Linux Distribution Agent?](#-what-is-linux-distribution-agent)
- [Features](#-features)
- [Demo](#-demo)
- [Installation](#-installation)
- [Quick Start](#-quick-start)
- [Commands](#-commands)
- [Configuration](#-configuration)
- [Shell Completions](#-shell-completions)
- [Examples](#-examples)
- [Supported Distributions](#-supported-distributions)
- [Architecture](#️-architecture)
- [Troubleshooting](#-troubleshooting)
- [Contributing](#-contributing)
- [License](#-license)
- [Acknowledgments](#-acknowledgments)

## 🛠️ Installation

### Quick Install (One-liner)

```bash
# Install latest release (recommended)
curl -fsSL https://raw.githubusercontent.com/GeneticxCln/linux-distro-agent/main/install.sh | bash

# Or install to custom location
curl -fsSL https://raw.githubusercontent.com/GeneticxCln/linux-distro-agent/main/install.sh | bash -s -- --prefix=/usr/local
```

### Dependencies

**Runtime Dependencies:**
- Linux kernel 3.10+ (systemd recommended)
- glibc 2.17+ or musl libc
- Package manager: `pacman`, `apt`, `dnf`, `zypper`, `portage`, `nix`, or `apk`

**Build Dependencies (for custom distros):**
- `pacstrap` (Arch-based builds)
- `debootstrap` (Debian/Ubuntu builds)
- `mksquashfs` (SquashFS creation)
- `xorriso` (ISO generation)
- `syslinux` (bootloader setup)

### From Source

```bash
git clone https://github.com/GeneticxCln/linux-distro-agent.git
cd linux-distro-agent
cargo build --release
```

The binary will be available at `target/release/linux-distro-agent`.

### System Installation

```bash
# Copy to a directory in your PATH
sudo cp target/release/linux-distro-agent /usr/local/bin/
```

### 🏠 Home Manager Integration

For Nix users, LDA integrates seamlessly with Home Manager:

```bash
# Quick setup with automated script
./scripts/setup-home-manager.sh

# Or manually add to your home.nix (see HOME_MANAGER_INTEGRATION.md)
```

**Benefits:**
- 📦 Hybrid package management (native + Nix packages)
- 🔄 Reproducible user environments
- 🎯 Declarative configuration management
- 🛠️ `lda-hm` helper script for Home Manager operations

See [HOME_MANAGER_INTEGRATION.md](HOME_MANAGER_INTEGRATION.md) for detailed setup instructions.

## 🚀 Quick Start

```bash
# Detect your Linux distribution
linux-distro-agent detect

# Get install command for a package
linux-distro-agent install vim

# Search for packages
linux-distro-agent search git

# Get system update command
linux-distro-agent update

# Show system information as JSON
linux-distro-agent info --pretty
```

## 📚 Commands

### Core Commands

#### `detect`
Detect and display current Linux distribution information.

```bash
linux-distro-agent detect [--extended]
```

**Options:**
- `-e, --extended`: Show extended information including URLs and additional metadata

**Example:**
```bash
$ linux-distro-agent detect
Detected Linux distribution: CachyOS Linux
Package Manager: pacman
```

#### `install <package>`
Generate package installation command.

```bash
linux-distro-agent install <PACKAGE> [--execute]
```

**Options:**
- `-e, --execute`: Execute the command directly (requires confirmation)

**Example:**
```bash
$ linux-distro-agent install vim
To install 'vim', run: sudo pacman -S vim
```

#### `search <query>`
Generate package search command.

```bash
linux-distro-agent search <QUERY> [--execute]
```

**Options:**
- `-e, --execute`: Execute the command directly

**Example:**
```bash
$ linux-distro-agent search git
To search for 'git', run: pacman -Ss git
```

#### `update`
Generate system update command.

```bash
linux-distro-agent update [--execute]
```

**Options:**
- `-e, --execute`: Execute the command directly (requires confirmation)

**Example:**
```bash
$ linux-distro-agent update
To update the system, run: sudo pacman -Syu
```

#### `remove <package>`
Generate package removal command.

```bash
linux-distro-agent remove <PACKAGE> [--execute]
```

**Options:**
- `-e, --execute`: Execute the command directly (requires confirmation)

**Example:**
```bash
$ linux-distro-agent remove vim
To remove 'vim', run: sudo pacman -R vim
```

### Information Commands

#### `info`
Display comprehensive system information as JSON.

```bash
linux-distro-agent info [--pretty]
```

**Options:**
- `-p, --pretty`: Pretty print JSON output

**Example:**
```bash
$ linux-distro-agent info --pretty
{
  "name": "CachyOS Linux",
  "version": null,
  "id": "cachyos",
  "package_manager": "pacman",
  "home_url": "https://cachyos.org/"
}
```

#### `list`
List installed packages.

```bash
linux-distro-agent list [--detailed] [--filter <PATTERN>]
```

**Options:**
- `-d, --detailed`: Show detailed package information
- `-f, --filter <PATTERN>`: Filter packages by name pattern

#### `package-info <package>`
Show detailed information about a package.

```bash
linux-distro-agent package-info <PACKAGE>
```

#### `list-supported`
List all supported distributions and package managers.

```bash
linux-distro-agent list-supported
```

#### `doctor`
Check system compatibility and provide recommendations.

```bash
linux-distro-agent doctor
```

### Management Commands

#### `history`
Manage command history.

```bash
linux-distro-agent history [--limit <N>] [--search <TERM>] [--clear]
```

**Options:**
- `-l, --limit <N>`: Number of recent entries to show (default: 10)
- `-s, --search <TERM>`: Search history for specific terms
- `--clear`: Clear command history

**Examples:**
```bash
# Show recent history
linux-distro-agent history

# Search history
linux-distro-agent history --search "install"

# Clear history
linux-distro-agent history --clear
```

#### `config`
Configuration management.

```bash
linux-distro-agent config <SUBCOMMAND>
```

**Subcommands:**
- `show`: Show current configuration
- `edit`: Edit configuration in default editor
- `reset`: Reset configuration to defaults
- `set <KEY> <VALUE>`: Set a configuration value

**Examples:**
```bash
# Show current config
linux-distro-agent config show

# Edit config file
linux-distro-agent config edit

# Set a config value
linux-distro-agent config set cache_duration 600
```

#### `cache`
Cache management.

```bash
linux-distro-agent cache <SUBCOMMAND>
```

**Subcommands:**
- `status`: Show cache status
- `clear`: Clear all cached data
- `list`: Show cached entries

**Examples:**
```bash
# Show cache status
linux-distro-agent cache status

# Clear cache
linux-distro-agent cache clear
```

### Utility Commands

#### `completions`
Generate shell completion scripts.

```bash
linux-distro-agent completions <SHELL>
```

**Supported shells:** `bash`, `zsh`, `fish`, `powershell`, `elvish`

### 🆕 New Commands in v0.3.0

#### `monitor`
System monitoring and health checks.

```bash
linux-distro-agent monitor [--metrics] [--health] [--history]
```

**Options:**
- `-m, --metrics`: Show current system metrics (CPU, memory, disk)
- `--health`: Run comprehensive health checks
- `--history`: Show metrics history and trends

**Example:**
```bash
$ linux-distro-agent monitor --metrics
System Metrics:
• CPU Usage: 45.2%
• Memory: 8.1GB / 16GB (50.6%)
• Disk: 120GB / 500GB (24%)
• Load Average: 1.2, 1.5, 1.8
```

#### `remote`
Remote host management and command execution.

```bash
linux-distro-agent remote --host <HOST> --command <COMMAND> [--sudo] [--test]
```

**Options:**
- `-h, --host <HOST>`: Remote host name or IP address
- `-c, --command <COMMAND>`: Command to execute remotely
- `--sudo`: Run command with sudo privileges
- `--test`: Test connectivity only

**Example:**
```bash
$ linux-distro-agent remote --host server01 --command "uptime" 
[server01] up 15 days, 3:42, 2 users, load average: 0.25, 0.15, 0.10
```

#### `wsm`
Window System Manager - manage display systems and desktop environments.

```bash
linux-distro-agent wsm [--detect] [--sessions] [--displays] [--restart <COMPONENT>] [--switch <TYPE>]
```

**Options:**
- `-d, --detect`: Detect current window system information
- `--sessions`: Show available desktop sessions
- `--displays`: Show display configuration
- `--restart <COMPONENT>`: Restart component (gdm, sddm, lightdm, x11)
- `--switch <TYPE>`: Switch session type (x11, wayland)

**Example:**
```bash
$ linux-distro-agent wsm --detect
Window System: X11
Display Manager: GDM
Desktop Environment: GNOME
Resolution: 1920x1080 @ 60Hz
```

#### `security`
Security auditing and system hardening.

```bash
linux-distro-agent security [--audit] [--json] [--severity <LEVEL>] [--category <CATEGORY>]
```

**Options:**
- `--audit`: Run full security audit
- `--json`: Output results in JSON format
- `--severity <LEVEL>`: Filter by severity (low, medium, high, critical)
- `--category <CATEGORY>`: Filter by category (network, filesystem, users, etc.)

**Example:**
```bash
$ linux-distro-agent security --audit --severity high
Security Audit Results:
⚠️  HIGH: SSH root login enabled
⚠️  HIGH: Firewall not configured
✅ PASS: Strong password policy enabled
```

#### `plugin`
Plugin management system.

```bash
linux-distro-agent plugin [--list] [--info <NAME>] [--enable <NAME>] [--disable <NAME>] [--exec <NAME>] [--install <PATH>]
```

**Options:**
- `-l, --list`: List all available plugins
- `--info <NAME>`: Show plugin information
- `--enable <NAME>`: Enable a plugin
- `--disable <NAME>`: Disable a plugin
- `--exec <NAME>`: Execute a plugin
- `--install <PATH>`: Install plugin from directory
- `--create <NAME>`: Create new plugin template

**Example:**
```bash
$ linux-distro-agent plugin --list
Available Plugins:
• backup-manager (enabled) - Automated system backups
• network-monitor (disabled) - Network traffic monitoring
• custom-commands (enabled) - User-defined command shortcuts
```

#### `system-config`
System configuration management.

```bash
linux-distro-agent system-config [--show] [--sample]
```

**Options:**
- `-s, --show`: Show current system configuration
- `--sample`: Generate sample configuration template

**Example:**
```bash
$ linux-distro-agent system-config --show
System Configuration:
• Hostname: workstation-01
• Timezone: America/New_York
• Locale: en_US.UTF-8
• Kernel: 6.1.0-lts
```

#### `self-update`
Update LDA to the latest version.

```bash
linux-distro-agent self-update [--force] [--dry-run]
```

**Options:**
- `-f, --force`: Force update even if already on latest version
- `--dry-run`: Show what would be updated without updating

**Example:**
```bash
$ linux-distro-agent self-update
Current LDA version: 0.3.0
Checking for updates...
Latest LDA version: 0.3.1
Downloading and installing the latest version...
🎉 LDA has been successfully updated!
```

#### `agent` ⭐ **NEW in v0.3.1**
AI Agent - Intelligent task planning and execution system.

```bash
linux-distro-agent agent [--start] [--add-task <TASK>] [--status] [--stats] [--clear-tasks] [--dry-run]
```

**Options:**
- `-s, --start`: Start the intelligent agent loop
- `--add-task <TASK>`: Add a task to the agent queue
- `--status`: Show agent status and current tasks
- `--stats`: Show agent learning data and statistics
- `--clear-tasks`: Clear all tasks from the agent queue
- `--dry-run`: Enable dry-run mode (tasks won't be executed)

**Example:**
```bash
# Start the AI agent
$ linux-distro-agent agent --start
🤖 AI Agent started - Intelligent task planning and execution

# Add a task for the agent
$ linux-distro-agent agent --add-task "Update system packages"
✅ Task added to agent queue

# Check agent status
$ linux-distro-agent agent --status
Agent Status: Active
Queued Tasks: 3
Completed Tasks: 15
Last Activity: 2 minutes ago
```

## 🛡️ Security

The Linux Distribution Agent follows the principle of least privilege and recommends running commands with necessary permissions only. Always review generated commands before execution to ensure they meet your security requirements. Regular updates are encouraged to keep your installation secure.

## ⚙️ Configuration

The tool uses a TOML configuration file located at `~/.config/linux-distro-agent/config.toml`.

### Configuration Options

```toml
cache_duration = 300          # Cache duration in seconds (default: 300)
enable_aur = true            # Enable AUR support for Arch-based systems
enable_flatpak = true        # Enable Flatpak support
enable_snap = false          # Enable Snap support
default_editor = "vim"       # Default editor for config editing
auto_update_cache = true     # Automatically update cache
history_enabled = true       # Enable command history tracking
backup_before_install = false # Create backups before package installation
preferred_aur_helper = "paru" # Preferred AUR helper (paru, yay, etc.)
```

### Managing Configuration

```bash
# View current configuration
linux-distro-agent config show

# Edit configuration interactively
linux-distro-agent config edit

# Set individual values
linux-distro-agent config set cache_duration 600
linux-distro-agent config set enable_aur true

# Reset to defaults
linux-distro-agent config reset
```

## 🐚 Shell Completions

### Quick Setup

#### Zsh
```bash
linux-distro-agent completions zsh > ~/.local/share/zsh/site-functions/_linux-distro-agent
```

#### Bash
```bash
linux-distro-agent completions bash > ~/.local/share/bash-completion/completions/linux-distro-agent
```

#### Fish
```bash
linux-distro-agent completions fish > ~/.config/fish/completions/linux-distro-agent.fish
```

### Manual Installation

For detailed installation instructions and troubleshooting, see [COMPLETIONS.md](COMPLETIONS.md).

## 🆕 New in v0.3.1 ⭐
- **🤖 AI Agent System**: Intelligent task planning, execution safety, adaptive learning, and smart automation with extensive dry-run capabilities
- **Self-Update**: Automatic update checking and installation for easy version management
- **Enhanced Plugin System**: More powerful plugin management and execution capabilities
- **Remote System Management**: Execute commands on remote systems via SSH
- **Comprehensive Monitoring**: Real-time metrics and comprehensive health checks

## 📖 Examples

### Basic Usage

```bash
# Detect your distribution
$ linux-distro-agent detect
Detected Linux distribution: Ubuntu
Package Manager: apt

# Install a package (show command only)
$ linux-distro-agent install neovim
To install 'neovim', run: sudo apt install neovim

# Install and execute immediately
$ linux-distro-agent install neovim --execute
[sudo] password for user:
Reading package lists... Done
...

# Search for packages
$ linux-distro-agent search python
To search for 'python', run: apt search python

# System update
$ linux-distro-agent update
To update the system, run: sudo apt update && sudo apt upgrade
```

### Advanced Usage

```bash
# Get detailed system info
$ linux-distro-agent info --pretty
{
  "name": "Ubuntu",
  "version": "22.04.3 LTS (Jammy Jellyfish)",
  "id": "ubuntu",
  "id_like": "debian",
  "version_id": "22.04",
  "package_manager": "apt"
}

# Self-update the tool
$ linux-distro-agent self-update
Checking for updates...
Latest version is 0.2.1.
Updating to the latest version...
Update complete!

# List installed packages with filter
$ linux-distro-agent list --filter python
python3
python3-pip
python3-dev

# Check system compatibility
$ linux-distro-agent doctor
System Compatibility Check:

✓ Distribution: Ubuntu
✓ Package Manager: apt
✓ Version information available

Recommendations:
✓ Your system is fully supported!
• All package management commands should work correctly
```

### History and Cache Management

```bash
# View command history
$ linux-distro-agent history --limit 5
Command History:
2024-01-15 10:30:15 - sudo apt install vim - vim
2024-01-15 10:25:10 - apt search git - N/A

# Search history
$ linux-distro-agent history --search install
2024-01-15 10:30:15 - sudo apt install vim - vim
2024-01-14 15:20:30 - sudo apt install curl - curl

# Check cache status
$ linux-distro-agent cache status
Cache entries: 3
Cache size: 1024 bytes
Last updated: 2024-01-15 10:30:15
```

## 🖥️ Supported Distributions

| Distribution Family | Package Manager | Distributions |
|-------------------|----------------|---------------|
| **Arch-based** | `pacman` | Arch Linux, CachyOS, Manjaro, EndeavourOS |
| **Debian-based** | `apt` | Ubuntu, Debian, Pop!_OS, Elementary OS |
| **Red Hat-based** | `dnf` | Fedora, RHEL, CentOS, Rocky Linux, AlmaLinux |
| **SUSE-based** | `zypper` | openSUSE Leap, openSUSE Tumbleweed |
| **Gentoo** | `portage` | Gentoo |
| **NixOS** | `nix` | NixOS |
| **Alpine** | `apk` | Alpine Linux |

### Adding Support for New Distributions

The tool automatically detects distributions using `/etc/os-release`. If your distribution isn't supported, it may still work if it's based on a supported distribution (using the `ID_LIKE` field).

## 🔧 Global Options

All commands support these global options:

- `-v, --verbose`: Enable verbose output
- `-q, --quiet`: Quiet mode - suppress non-essential output
- `-h, --help`: Show help information
- `-V, --version`: Show version information

## 🏗️ Architecture

The project is structured as follows:

```
src/
├── main.rs           # CLI interface and command routing
├── distro.rs         # Distribution detection and package manager mapping
├── executor.rs       # Command execution utilities
├── logger.rs         # Logging and output formatting
├── config.rs         # Legacy configuration (unused)
├── config_manager.rs # Configuration management
├── history.rs        # Command history management
└── cache.rs          # Caching system
```

## 🏗️ Linux Distribution Builder

### Quick Start

```bash
# Build a minimal distribution
sudo lda build-distro --minimal

# Generate configuration template
lda generate-config > my-distro.toml

# Edit the configuration
nano my-distro.toml

# Build with custom configuration
sudo lda build-distro -c my-distro.toml
```

### Configuration Options

```toml
name = "MyLinux"
version = "1.0"
description = "A custom Linux distribution"
architecture = "x86_64"
base_system = "Arch"

[packages]
essential = ["base", "linux", "linux-firmware", "networkmanager"]
desktop_environment = "Xfce"  # Options: Gnome, KDE, Xfce, LXDE, Mate, Cinnamon, Sway, I3, None
additional_packages = ["firefox", "vim", "git"]

[kernel]
kernel_type = "Vanilla"  # Options: Vanilla, LTS, Hardened, RT, Custom("name")

[bootloader]
bootloader = "Syslinux"  # Options: Syslinux, GRUB, Systemd, rEFInd
timeout = 30
default_entry = "linux"

[branding.colors]
primary = "#0078d4"
secondary = "#005a9e"
accent = "#00bcf2"

[filesystem]
root_fs = "SquashFS"     # Options: SquashFS, Ext4, Btrfs, XFS
compression = "Xz"       # Options: Gzip, Xz, Zstd, Lz4, None
size_limit = 4096        # MB
```

### Advanced Building

```bash
# Custom build directories
sudo lda build-distro \
  --config my-distro.toml \
  --work-dir /var/build \
  --output-dir ~/isos

# Build different base systems
lda generate-config --template minimal > arch-minimal.toml
# Edit to change base_system to "Debian" or "Ubuntu"
sudo lda build-distro -c debian-distro.toml

# Generate different templates
lda generate-config --template minimal     # Minimal configuration
lda generate-config -o desktop.toml        # Save to file
```

### Build Process

The build process includes:

1. **🏗️ Root Filesystem Creation** - Bootstrap base system
2. **🐧 Kernel Installation** - Install and configure kernel
3. **📦 Package Installation** - Install essential and additional packages  
4. **⚙️ System Configuration** - Configure hostname, services, users
5. **🎨 Branding Application** - Apply custom themes and branding
6. **🥾 Bootloader Setup** - Configure bootloader and boot entries
7. **💿 ISO Generation** - Create bootable ISO image

### Common Issues

1. **"Unable to determine package manager"**
   - Your distribution might not be in the supported list
   - Check if `/etc/os-release` exists and contains proper ID fields
   - Run `linux-distro-agent doctor` for system compatibility check

2. **Configuration file errors**
   - Reset configuration: `linux-distro-agent config reset`
   - Check file permissions in `~/.config/linux-distro-agent/`

3. **Cache issues**
   - Clear cache: `linux-distro-agent cache clear`
   - Check available disk space

### Getting Help

```bash
# General help
linux-distro-agent --help

# Command-specific help
linux-distro-agent install --help

# System compatibility check
linux-distro-agent doctor
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

```bash
git clone https://github.com/GeneticxCln/linux-distro-agent.git
cd linux-distro-agent
cargo build
cargo test
```

### Running Tests

```bash
cargo test
```

### Code Style

This project uses standard Rust formatting:

```bash
cargo fmt
cargo clippy
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- CLI powered by [clap](https://github.com/clap-rs/clap)
- Configuration management with [serde](https://serde.rs/) and [toml](https://github.com/toml-rs/toml)
- File system utilities from [dirs](https://github.com/dirs-dev/dirs-rs)

---

**Made with ❤️ for the Linux community**
