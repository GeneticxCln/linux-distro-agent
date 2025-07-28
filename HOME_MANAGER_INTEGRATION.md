# 🏠 Home Manager Integration

This document describes how to integrate Linux Distro Agent with [Home Manager](https://github.com/nix-community/home-manager), providing a seamless Nix-based package management experience alongside traditional package managers.

## 🚀 Quick Setup

Run the automated setup script:

```bash
./scripts/setup-home-manager.sh
```

This script will:
1. Check prerequisites (Nix, Home Manager)
2. Backup your existing Home Manager configuration
3. Add Linux Distro Agent integration
4. Install the `lda-hm` helper script
5. Apply the new configuration

## 📋 Manual Setup

If you prefer to set up the integration manually, follow these steps:

### Prerequisites

- [Nix package manager](https://nixos.org/download.html)
- [Home Manager](https://github.com/nix-community/home-manager#installation)

### 1. Add to Home Manager Configuration

Edit your `~/.config/home-manager/home.nix`:

```nix
{ config, pkgs, ... }:

let
  # Linux Distro Agent integration
  linux-distro-agent = (import /path/to/linux-distro-agent).packages.${pkgs.system}.default;
in
{
  # Your existing configuration...

  # Add LDA to your packages
  home.packages = with pkgs; [
    # Your existing packages...
    linux-distro-agent
  ];

  # Useful aliases
  home.shellAliases = {
    "lda" = "linux-distro-agent";
    "detect" = "linux-distro-agent detect";
    "install-pkg" = "linux-distro-agent install";
    "search-pkg" = "linux-distro-agent search";
    "update-pkg" = "linux-distro-agent update";
    "remove-pkg" = "linux-distro-agent remove";
    "list-distros" = "linux-distro-agent list-supported";
    "sys-info" = "linux-distro-agent info --pretty";
  };

  # Shell completion integration
  programs.zsh.initExtra = ''
    # Linux Distribution Agent completions
    if command -v linux-distro-agent >/dev/null 2>&1; then
      source <(linux-distro-agent completions zsh)
    fi
  '';

  programs.bash.initExtra = ''
    # Linux Distribution Agent completions
    if command -v linux-distro-agent >/dev/null 2>&1; then
      source <(linux-distro-agent completions bash)
    fi
  '';
}
```

### 2. Install the Helper Script

Copy the `lda-hm` script to your local bin directory:

```bash
mkdir -p ~/.local/bin
cp scripts/lda-hm ~/.local/bin/
chmod +x ~/.local/bin/lda-hm
```

Ensure `~/.local/bin` is in your PATH by adding this to your shell profile:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

### 3. Apply Configuration

```bash
home-manager switch
```

## 🔧 Using the Integration

### Linux Distro Agent Commands

All standard LDA commands work as expected:

```bash
# Detect current distribution
lda detect

# Install a package using the native package manager
lda install ripgrep

# Search for packages
lda search terminal

# Update system packages
lda update

# Remove a package
lda remove old-package

# Show system information
lda info --pretty
```

### Home Manager Integration (`lda-hm`)

The `lda-hm` script provides Home Manager-specific package management:

```bash
# Install a package via Home Manager (from nixpkgs)
lda-hm install ripgrep

# Remove a package from Home Manager
lda-hm remove ripgrep

# Search nixpkgs for packages
lda-hm search terminal

# Update all Home Manager packages
lda-hm update

# List installed Home Manager packages
lda-hm list

# Edit Home Manager configuration
lda-hm edit

# Apply Home Manager configuration
lda-hm switch
```

### Shell Aliases

The integration adds several convenient aliases:

| Alias | Command | Description |
|-------|---------|-------------|
| `lda` | `linux-distro-agent` | Main LDA command |
| `detect` | `linux-distro-agent detect` | Detect distribution |
| `install-pkg` | `linux-distro-agent install` | Install via native PM |
| `search-pkg` | `linux-distro-agent search` | Search packages |
| `update-pkg` | `linux-distro-agent update` | Update system |
| `remove-pkg` | `linux-distro-agent remove` | Remove package |
| `list-distros` | `linux-distro-agent list-supported` | List supported distros |
| `sys-info` | `linux-distro-agent info --pretty` | System information |

## 🎯 Use Cases

### Hybrid Package Management

Use both traditional package managers and Nix packages:

```bash
# Install system packages via native package manager
lda install docker git

# Install development tools via Home Manager
lda-hm install rustc cargo nodejs

# Install user-specific tools via Home Manager
lda-hm install fzf bat exa
```

### Development Environment Setup

Create reproducible development environments:

```bash
# Install system dependencies
lda install build-essential

# Install development tools via Home Manager
lda-hm install git gh neovim tmux

# System updates via native package manager
lda update

# Home Manager packages update
lda-hm update
```

### Cross-Distribution Compatibility

Maintain consistent user environments across different distributions:

```bash
# Works on any supported distribution
detect                    # Shows: "Ubuntu 22.04 LTS"
lda-hm install ripgrep   # Installs from nixpkgs
lda install curl         # Installs via apt/pacman/dnf/etc
```

## 📂 File Structure

After setup, you'll have:

```
~/.config/home-manager/
├── home.nix                    # Your Home Manager configuration
└── backups/                    # Automatic backups
    └── home.nix.backup.*       # Timestamped backups

~/.local/bin/
└── lda-hm                      # Home Manager integration script
```

## 🔍 Troubleshooting

### Home Manager Not Found

```bash
# Install Home Manager
nix run home-manager/master -- init --switch
```

### Configuration Errors

Check syntax:
```bash
nix-instantiate --parse ~/.config/home-manager/home.nix
```

### Path Issues

Ensure `~/.local/bin` is in PATH:
```bash
echo $PATH | grep -q "$HOME/.local/bin" || echo "Add ~/.local/bin to PATH"
```

### Flake Issues

If using flakes, ensure experimental features are enabled:
```bash
# Add to ~/.config/nix/nix.conf or /etc/nix/nix.conf
experimental-features = nix-command flakes
```

## 🎛️ Advanced Configuration

### Custom Package Sets

Add custom package derivations:

```nix
let
  myCustomPackages = with pkgs; [
    # Your custom packages
  ];
in
{
  home.packages = with pkgs; [
    linux-distro-agent
  ] ++ myCustomPackages;
}
```

### Per-Machine Configuration

Use different configurations per machine:

```nix
let
  hostname = builtins.readFile /proc/sys/kernel/hostname;
  isWorkMachine = builtins.match ".*work.*" hostname != null;
in
{
  home.packages = with pkgs; [
    linux-distro-agent
  ] ++ lib.optionals isWorkMachine [
    # Work-specific packages
    slack
    teams
  ];
}
```

### Shell-Specific Configuration

Different shells can have different setups:

```nix
{
  programs.zsh = {
    enable = true;
    initExtra = ''
      # ZSH-specific LDA setup
      source <(linux-distro-agent completions zsh)
      bindkey '^[l' 'lda detect\n'  # Alt+L for quick detect
    '';
  };

  programs.bash = {
    enable = true;
    initExtra = ''
      # Bash-specific LDA setup
      source <(linux-distro-agent completions bash)
    '';
  };
}
```

## 🤝 Integration Benefits

1. **Reproducible Environments**: Home Manager ensures consistent user environments
2. **Declarative Configuration**: Version-controlled package management
3. **Rollback Support**: Easy rollback to previous configurations
4. **Per-User Packages**: No need for sudo to install user packages
5. **Cross-Platform**: Same configuration works across different Linux distributions
6. **Complementary Approach**: Works alongside system package managers

## 📚 Additional Resources

- [Home Manager Manual](https://nix-community.github.io/home-manager/)
- [Nix Package Search](https://search.nixos.org/packages)
- [Home Manager Options](https://mipmip.github.io/home-manager-option-search/)
- [Linux Distro Agent Documentation](README.md)

## 🆘 Support

If you encounter issues with the Home Manager integration:

1. Check the [troubleshooting section](#-troubleshooting)
2. Review your Home Manager configuration syntax
3. Ensure all prerequisites are installed
4. Create an issue in the [Linux Distro Agent repository](https://github.com/your-repo/linux-distro-agent/issues)

---

*This integration enhances Linux Distro Agent with the power of Nix and Home Manager, providing the best of both worlds: native package manager compatibility and reproducible user environments.*
