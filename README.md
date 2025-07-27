# Linux Distribution Agent (LDA)

<div align="center">

![LDA Logo](https://img.shields.io/badge/LDA-Linux%20Distribution%20Agent-blue?style=for-the-badge)
[![Version](https://img.shields.io/badge/version-0.1.0-green?style=for-the-badge)](https://github.com/GeneticxCln/linux-distro-agent/releases)
[![License](https://img.shields.io/badge/license-MIT-orange?style=for-the-badge)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024-red?style=for-the-badge)](https://www.rust-lang.org/)

**A comprehensive Linux distribution management tool with custom distro building capabilities**

[Features](#-features) • [Installation](#-installation) • [Usage](#-usage) • [Distro Building](#-linux-distribution-builder) • [Documentation](#-documentation)

</div>

---

<div align="center">

**A comprehensive command-line tool written in Rust for detecting Linux distributions and providing distribution-specific package management commands.**

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub release](https://img.shields.io/github/release/GeneticxCln/linux-distro-agent.svg)](https://github.com/GeneticxCln/linux-distro-agent/releases)
[![GitHub issues](https://img.shields.io/github/issues/GeneticxCln/linux-distro-agent.svg)](https://github.com/GeneticxCln/linux-distro-agent/issues)
[![GitHub stars](https://img.shields.io/github/stars/GeneticxCln/linux-distro-agent.svg)](https://github.com/GeneticxCln/linux-distro-agent/stargazers)
[![Build Status](https://img.shields.io/github/actions/workflow/status/GeneticxCln/linux-distro-agent/ci.yml?branch=main)](https://github.com/GeneticxCln/linux-distro-agent/actions)

[🚀 Quick Start](#-quick-start) • [📖 Documentation](#-table-of-contents) • [💻 Demo](#-demo) • [🤝 Contributing](#-contributing)

</div>

---

## 🌟 What is Linux Distribution Agent?

Linux Distribution Agent is a powerful, cross-platform command-line utility that bridges the gap between different Linux distributions by providing unified package management commands. Whether you're switching between Ubuntu, Arch, Fedora, or any other major distribution, this tool ensures you never have to remember different package manager syntaxes again.

### ✨ Why Choose Linux Distribution Agent?

- 🎯 **Universal**: Works across all major Linux distributions
- ⚡ **Fast**: Built in Rust for maximum performance
- 🧠 **Smart**: Automatic distribution detection
- 📱 **Modern**: JSON output, shell completions, and rich CLI interface
- 🔧 **Configurable**: Extensive customization options
- 📚 **Well-documented**: Comprehensive documentation and examples

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
