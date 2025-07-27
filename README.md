# Linux Distribution Agent

A comprehensive command-line tool written in Rust for detecting Linux distributions and providing distribution-specific package management commands.

## Features

- **Distribution Detection**: Automatically detects your Linux distribution from `/etc/os-release`
- **Package Manager Support**: Supports major package managers including:
  - `pacman` (Arch Linux, CachyOS, Manjaro, EndeavourOS)
  - `apt` (Ubuntu, Debian, Pop!_OS, Elementary OS)
  - `dnf` (Fedora, RHEL, CentOS, Rocky Linux, AlmaLinux)
  - `zypper` (openSUSE)
  - `portage` (Gentoo)
  - `nix` (NixOS)
  - `apk` (Alpine Linux)
- **Command Generation**: Generates appropriate commands for package installation, searching, and system updates
- **JSON Output**: Comprehensive system information export in JSON format

## Installation

### From Source

```bash
git clone <repository-url>
cd linux-distro-agent
cargo build --release
```

The binary will be available at `target/release/linux-distro-agent`.

## Usage

### Basic Commands

#### Detect Distribution
```bash
linux-distro-agent detect
```

#### Install a Package
```bash
linux-distro-agent install vim
# Output: To install 'vim', run: sudo pacman -S vim
```

#### Search for Packages
```bash
linux-distro-agent search git
# Output: To search for 'git', run: pacman -Ss git
```

#### System Update Command
```bash
linux-distro-agent update
# Output: To update the system, run: sudo pacman -Syu
```

#### System Information (JSON)
```bash
linux-distro-agent info
```

### Shell Completions

`linux-distro-agent` supports shell completions for bash, zsh, fish, PowerShell, and elvish.

#### Quick Setup for Zsh
```bash
linux-distro-agent completions zsh > ~/.local/share/zsh/site-functions/_linux-distro-agent
```

#### Quick Setup for Bash
```bash
linux-distro-agent completions bash > ~/.local/share/bash-completion/completions/linux-distro-agent
```

#### Quick Setup for Fish
```bash
linux-distro-agent completions fish > ~/.config/fish/completions/linux-distro-agent.fish
```

For detailed installation instructions and troubleshooting, see [COMPLETIONS.md](COMPLETIONS.md).

### Help
```bash
linux-distro-agent --help
linux-distro-agent <command> --help
```

## Example Output

### Detection on CachyOS
```
$ linux-distro-agent detect
Detected Linux distribution: CachyOS Linux
Package Manager: pacman
```

### JSON Information
```json
{
  "name": "CachyOS Linux",
  "version": null,
  "id": "cachyos",
  "id_like": null,
  "version_id": null,
  "pretty_name": "CachyOS",
  "home_url": "https://cachyos.org/",
  "support_url": "https://discuss.cachyos.org/",
  "bug_report_url": "https://github.com/cachyos",
  "package_manager": "pacman"
}
```

## Supported Distributions

- **Arch-based**: Arch Linux, CachyOS, Manjaro, EndeavourOS
- **Debian-based**: Ubuntu, Debian, Pop!_OS, Elementary OS
- **Red Hat-based**: Fedora, RHEL, CentOS, Rocky Linux, AlmaLinux
- **SUSE-based**: openSUSE Leap, openSUSE Tumbleweed
- **Others**: Gentoo, NixOS, Alpine Linux

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is open source and available under the [MIT License](LICENSE).
