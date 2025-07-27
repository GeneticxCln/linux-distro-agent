#!/bin/bash

# Install completions helper script for linux-distro-agent
# This script automatically detects your shell and installs completions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect current shell
detect_shell() {
    local shell_name
    if [ -n "$ZSH_VERSION" ]; then
        shell_name="zsh"
    elif [ -n "$BASH_VERSION" ]; then
        shell_name="bash"
    elif [ -n "$FISH_VERSION" ]; then
        shell_name="fish"
    else
        # Fallback to checking $SHELL environment variable
        case "$(basename "$SHELL")" in
            zsh) shell_name="zsh" ;;
            bash) shell_name="bash" ;;
            fish) shell_name="fish" ;;
            *) shell_name="unknown" ;;
        esac
    fi
    echo "$shell_name"
}

# Check if linux-distro-agent is available
check_binary() {
    if ! command -v linux-distro-agent >/dev/null 2>&1; then
        print_error "linux-distro-agent not found in PATH"
        print_info "Please install linux-distro-agent first or add it to your PATH"
        exit 1
    fi
}

# Install completions for bash
install_bash_completions() {
    local completion_dir="$HOME/.local/share/bash-completion/completions"
    local completion_file="$completion_dir/linux-distro-agent"
    
    print_info "Installing bash completions..."
    
    # Create directory if it doesn't exist
    mkdir -p "$completion_dir"
    
    # Generate and save completion script
    linux-distro-agent completions bash > "$completion_file"
    
    print_success "Bash completions installed to $completion_file"
    print_info "You may need to restart your shell or run: source ~/.bashrc"
}

# Install completions for zsh
install_zsh_completions() {
    local completion_dir="$HOME/.local/share/zsh/site-functions"
    local completion_file="$completion_dir/_linux-distro-agent"
    
    print_info "Installing zsh completions..."
    
    # Create directory if it doesn't exist
    mkdir -p "$completion_dir"
    
    # Generate and save completion script
    linux-distro-agent completions zsh > "$completion_file"
    
    # Check if fpath needs to be updated
    if ! grep -q "fpath.*\.local/share/zsh/site-functions" "$HOME/.zshrc" 2>/dev/null; then
        print_warning "You may need to add the following line to your ~/.zshrc:"
        echo "fpath=(~/.local/share/zsh/site-functions \$fpath)"
        echo ""
        print_info "Would you like me to add this automatically? (y/N)"
        read -r response
        if [[ "$response" =~ ^[Yy]$ ]]; then
            echo 'fpath=(~/.local/share/zsh/site-functions $fpath)' >> "$HOME/.zshrc"
            print_success "Added fpath configuration to ~/.zshrc"
        fi
    fi
    
    print_success "Zsh completions installed to $completion_file"
    print_info "You may need to restart your shell or run: autoload -U compinit && compinit"
}

# Install completions for fish
install_fish_completions() {
    local completion_dir="$HOME/.config/fish/completions"
    local completion_file="$completion_dir/linux-distro-agent.fish"
    
    print_info "Installing fish completions..."
    
    # Create directory if it doesn't exist
    mkdir -p "$completion_dir"
    
    # Generate and save completion script
    linux-distro-agent completions fish > "$completion_file"
    
    print_success "Fish completions installed to $completion_file"
    print_info "Completions should be available immediately in new fish sessions"
}

# Main installation function
install_completions() {
    local shell="$1"
    
    case "$shell" in
        bash)
            install_bash_completions
            ;;
        zsh)
            install_zsh_completions
            ;;
        fish)
            install_fish_completions
            ;;
        *)
            print_error "Unsupported shell: $shell"
            print_info "Supported shells: bash, zsh, fish"
            print_info "You can manually generate completions using:"
            print_info "  linux-distro-agent completions [shell]"
            exit 1
            ;;
    esac
}

# Main script
main() {
    print_info "Linux Distro Agent - Completions Installer"
    echo ""
    
    # Check if binary exists
    check_binary
    
    # Allow shell override via argument
    if [ $# -eq 1 ]; then
        shell="$1"
        print_info "Using shell: $shell (from argument)"
    else
        shell="$(detect_shell)"
        if [ "$shell" = "unknown" ]; then
            print_error "Could not detect your shell"
            print_info "Please specify the shell as an argument:"
            print_info "  $0 [bash|zsh|fish]"
            exit 1
        fi
        print_info "Detected shell: $shell"
    fi
    
    echo ""
    
    # Install completions
    install_completions "$shell"
    
    echo ""
    print_success "Installation complete!"
    
    # Provide additional instructions
    case "$shell" in
        bash)
            print_info "To enable completions in your current session, run:"
            print_info "  source ~/.local/share/bash-completion/completions/linux-distro-agent"
            ;;
        zsh)
            print_info "To enable completions in your current session, run:"
            print_info "  autoload -U compinit && compinit"
            ;;
        fish)
            print_info "Completions should work immediately in new fish sessions"
            ;;
    esac
}

# Show help
show_help() {
    echo "Linux Distro Agent - Completions Installer"
    echo ""
    echo "Usage: $0 [SHELL]"
    echo ""
    echo "Arguments:"
    echo "  SHELL    Shell to install completions for (bash, zsh, fish)"
    echo "           If not specified, will auto-detect current shell"
    echo ""
    echo "Examples:"
    echo "  $0        # Auto-detect and install for current shell"
    echo "  $0 bash   # Install bash completions"
    echo "  $0 zsh    # Install zsh completions"
    echo "  $0 fish   # Install fish completions"
    echo ""
    echo "Options:"
    echo "  -h, --help    Show this help message"
}

# Handle arguments
if [ $# -gt 1 ]; then
    print_error "Too many arguments"
    show_help
    exit 1
elif [ $# -eq 1 ] && [[ "$1" =~ ^(-h|--help)$ ]]; then
    show_help
    exit 0
fi

# Run main function
main "$@"
