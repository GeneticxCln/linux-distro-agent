#!/bin/bash
# Linux Distro Agent - Home Manager Setup Script
# This script helps users integrate LDA with their Home Manager configuration

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
HM_CONFIG_DIR="$HOME/.config/home-manager"
HM_CONFIG="$HM_CONFIG_DIR/home.nix"
LDA_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKUP_DIR="$HOME/.config/home-manager/backups"

# Helper functions
log_info() {
    echo -e "${BLUE}🔍${NC} $1"
}

log_success() {
    echo -e "${GREEN}✅${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠️${NC} $1"
}

log_error() {
    echo -e "${RED}❌${NC} $1"
}

log_step() {
    echo -e "${PURPLE}📋${NC} $1"
}

log_note() {
    echo -e "${CYAN}💡${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Create backup of existing configuration
create_backup() {
    if [[ -f "$HM_CONFIG" ]]; then
        mkdir -p "$BACKUP_DIR"
        local backup_file="$BACKUP_DIR/home.nix.backup.$(date +%Y%m%d_%H%M%S)"
        cp "$HM_CONFIG" "$backup_file"
        log_success "Created backup: $backup_file"
    fi
}

# Check prerequisites
check_prerequisites() {
    log_step "Checking prerequisites..."
    
    # Check if nix is installed
    if ! command_exists nix; then
        log_error "Nix package manager is not installed"
        echo "Please install Nix first: https://nixos.org/download.html"
        exit 1
    fi
    
    # Check if Home Manager is installed
    if ! command_exists home-manager; then
        log_warning "Home Manager is not installed"
        echo "Would you like to install Home Manager? (y/N)"
        read -r response
        if [[ "$response" =~ ^[Yy]$ ]]; then
            install_home_manager
        else
            log_error "Home Manager is required for this setup"
            exit 1
        fi
    fi
    
    log_success "Prerequisites check passed"
}

# Install Home Manager
install_home_manager() {
    log_step "Installing Home Manager..."
    
    # Check if flakes are available
    if nix --help 2>/dev/null | grep -q "experimental-features"; then
        log_info "Installing Home Manager with flakes support..."
        nix run home-manager/master -- init --switch
    else
        log_info "Installing Home Manager (legacy)..."
        nix-channel --add https://github.com/nix-community/home-manager/archive/master.tar.gz home-manager
        nix-channel --update
        nix-shell '<home-manager>' -A install
    fi
    
    log_success "Home Manager installed"
}

# Create initial Home Manager configuration
create_initial_config() {
    log_step "Creating initial Home Manager configuration..."
    
    mkdir -p "$HM_CONFIG_DIR"
    
    cat > "$HM_CONFIG" << 'EOF'
{ config, pkgs, ... }:

{
  # Home Manager needs a bit of information about you and the
  # paths it should manage.
  home.username = builtins.getEnv "USER";
  home.homeDirectory = builtins.getEnv "HOME";

  # This value determines the Home Manager release that your
  # configuration is compatible with.
  home.stateVersion = "23.11";

  # Let Home Manager install and manage itself.
  programs.home-manager.enable = true;

  # The home.packages option allows you to install Nix packages into your
  # environment.
  home.packages = with pkgs; [
    # Add your packages here
  ];

  # Home Manager shell integration
  programs.bash.enable = true;
  programs.zsh.enable = true;
}
EOF
    
    log_success "Created initial Home Manager configuration"
}

# Add LDA integration to Home Manager config
add_lda_integration() {
    log_step "Adding Linux Distro Agent integration..."
    
    # Create backup first
    create_backup
    
    # Check if LDA is already integrated
    if grep -q "linux-distro-agent" "$HM_CONFIG"; then
        log_warning "Linux Distro Agent integration already exists in configuration"
        echo "Would you like to update it? (y/N)"
        read -r response
        if [[ ! "$response" =~ ^[Yy]$ ]]; then
            return
        fi
    fi
    
    # Create temporary file with integrated configuration
    local temp_file=$(mktemp)
    
    # Read the current config and add LDA integration
    awk -v lda_path="$LDA_DIR" '
    BEGIN { 
        in_packages = 0
        added_lda = 0
        added_aliases = 0
    }
    
    # Add LDA import at the top after the initial line
    /^{ config, pkgs, ... }:$/ {
        print $0
        print ""
        print "let"
        print "  # Linux Distro Agent integration"
        print "  linux-distro-agent = (import " lda_path ").packages.${pkgs.system}.default;"
        print "in"
        next
    }
    
    # Add LDA to packages list
    /home\.packages = with pkgs; \[/ {
        print $0
        in_packages = 1
        next
    }
    
    in_packages && /\];$/ && !added_lda {
        print "    # Linux Distro Agent"
        print "    linux-distro-agent"
        print ""
        print $0
        in_packages = 0
        added_lda = 1
        next
    }
    
    # Add shell aliases
    /^}$/ && !added_aliases {
        print ""
        print "  # Linux Distro Agent aliases"
        print "  home.shellAliases = {"
        print "    \"lda\" = \"linux-distro-agent\";"
        print "    \"detect\" = \"linux-distro-agent detect\";"
        print "    \"install-pkg\" = \"linux-distro-agent install\";"
        print "    \"search-pkg\" = \"linux-distro-agent search\";"
        print "    \"update-pkg\" = \"linux-distro-agent update\";"
        print "    \"remove-pkg\" = \"linux-distro-agent remove\";"
        print "    \"list-distros\" = \"linux-distro-agent list-supported\";"
        print "    \"sys-info\" = \"linux-distro-agent info --pretty\";"
        print "  };"
        print ""
        print "  # Shell completion integration"
        print "  programs.zsh.initExtra = '\''"
        print "    # Linux Distribution Agent completions"
        print "    if command -v linux-distro-agent >/dev/null 2>&1; then"
        print "      source <(linux-distro-agent completions zsh)"
        print "    fi"
        print "  '\'';"
        print ""
        print "  programs.bash.initExtra = '\''"
        print "    # Linux Distribution Agent completions"
        print "    if command -v linux-distro-agent >/dev/null 2>&1; then"
        print "      source <(linux-distro-agent completions bash)"
        print "    fi"
        print "  '\'';"
        added_aliases = 1
    }
    
    { print }
    ' "$HM_CONFIG" > "$temp_file"
    
    # Validate the new configuration
    if nix-instantiate --parse "$temp_file" >/dev/null 2>&1; then
        mv "$temp_file" "$HM_CONFIG"
        log_success "Added Linux Distro Agent integration to Home Manager"
    else
        rm -f "$temp_file"
        log_error "Generated configuration is invalid. Backup preserved."
        exit 1
    fi
}

# Install lda-hm script to user's PATH
install_lda_hm_script() {
    log_step "Installing lda-hm script..."
    
    local target_dir="$HOME/.local/bin"
    mkdir -p "$target_dir"
    
    if [[ -f "$LDA_DIR/scripts/lda-hm" ]]; then
        cp "$LDA_DIR/scripts/lda-hm" "$target_dir/"
        chmod +x "$target_dir/lda-hm"
        log_success "Installed lda-hm script to $target_dir"
        
        # Check if ~/.local/bin is in PATH
        if [[ ":$PATH:" != *":$target_dir:"* ]]; then
            log_warning "$target_dir is not in your PATH"
            log_note "Add 'export PATH=\"\$HOME/.local/bin:\$PATH\"' to your shell profile"
        fi
    else
        log_error "lda-hm script not found at $LDA_DIR/scripts/lda-hm"
        exit 1
    fi
}

# Apply Home Manager configuration
apply_configuration() {
    log_step "Applying Home Manager configuration..."
    
    if home-manager switch; then
        log_success "Home Manager configuration applied successfully!"
    else
        log_error "Failed to apply Home Manager configuration"
        log_note "Check the configuration at $HM_CONFIG"
        exit 1
    fi
}

# Show post-installation information
show_completion_info() {
    echo
    echo -e "${GREEN}🎉 Setup Complete!${NC}"
    echo
    echo "Linux Distro Agent has been integrated with Home Manager!"
    echo
    echo -e "${CYAN}Available commands:${NC}"
    echo "  lda                    - Linux Distro Agent CLI"
    echo "  lda-hm                 - Home Manager integration script"
    echo
    echo -e "${CYAN}Useful aliases added:${NC}"
    echo "  detect                 - Detect current distribution"
    echo "  install-pkg <pkg>      - Install a package"
    echo "  search-pkg <query>     - Search for packages"
    echo "  sys-info               - Show system information"
    echo
    echo -e "${CYAN}Home Manager integration:${NC}"
    echo "  lda-hm install <pkg>   - Install via Home Manager"
    echo "  lda-hm search <query>  - Search nixpkgs"
    echo "  lda-hm edit            - Edit Home Manager config"
    echo
    echo -e "${YELLOW}Note:${NC} Restart your shell or run 'source ~/.bashrc' (or ~/.zshrc)"
    echo "      to activate the new aliases and completions."
    echo
    echo -e "${BLUE}Configuration files:${NC}"
    echo "  Home Manager: $HM_CONFIG"
    echo "  LDA Script:   $HOME/.local/bin/lda-hm"
    echo "  Backups:      $BACKUP_DIR"
}

# Main setup function
main() {
    echo -e "${PURPLE}🏠 Linux Distro Agent - Home Manager Setup${NC}"
    echo "============================================"
    echo
    
    # Check if Home Manager config exists
    if [[ ! -f "$HM_CONFIG" ]]; then
        log_info "No Home Manager configuration found"
        echo "Would you like to create an initial configuration? (Y/n)"
        read -r response
        if [[ ! "$response" =~ ^[Nn]$ ]]; then
            create_initial_config
        fi
    fi
    
    # Run setup steps
    check_prerequisites
    add_lda_integration
    install_lda_hm_script
    apply_configuration
    show_completion_info
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "Linux Distro Agent - Home Manager Setup Script"
        echo
        echo "Usage: $0 [options]"
        echo
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --backup-only  Only create a backup of current config"
        echo
        echo "This script will:"
        echo "  1. Check prerequisites (Nix, Home Manager)"
        echo "  2. Backup existing Home Manager configuration"
        echo "  3. Add Linux Distro Agent integration"
        echo "  4. Install the lda-hm helper script"
        echo "  5. Apply the new configuration"
        exit 0
        ;;
    --backup-only)
        create_backup
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac
