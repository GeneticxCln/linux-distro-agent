#!/bin/bash
# Linux Distribution Agent - Quick Install Script

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
PREFIX="/usr/local/bin"
GITHUB_REPO="GeneticxCln/linux-distro-agent"
BINARY_NAME="linux-distro-agent"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --prefix=*)
            PREFIX="${1#*=}"
            shift
            ;;
        --prefix)
            PREFIX="$2"
            shift 2
            ;;
        -h|--help)
            echo "Linux Distribution Agent - Quick Install Script"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --prefix=DIR    Install to DIR (default: /usr/local/bin)"
            echo "  -h, --help      Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                              # Install to /usr/local/bin"
            echo "  $0 --prefix=/usr/bin           # Install to /usr/bin"
            echo "  $0 --prefix=~/.local/bin       # Install to user directory"
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}" >&2
            exit 1
            ;;
    esac
done

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're on Linux
check_platform() {
    if [[ "$OSTYPE" != "linux-gnu"* ]]; then
        log_error "This script is designed for Linux systems only."
        exit 1
    fi
}

# Detect architecture
detect_arch() {
    local arch
    arch=$(uname -m)
    case $arch in
        x86_64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        armv7l)
            echo "armv7"
            ;;
        *)
            log_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

# Check dependencies
check_dependencies() {
    local deps=("curl" "tar")
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            log_error "Required dependency '$dep' is not installed."
            log_info "Please install $dep and try again."
            exit 1
        fi
    done
}

# Get latest release info
get_latest_release() {
    local release_url="https://api.github.com/repos/$GITHUB_REPO/releases/latest"
    log_info "Fetching latest release information..."
    
    # Try to get release info
    if ! curl -s "$release_url" | grep -o '"tag_name": *"[^"]*"' | cut -d'"' -f4 2>/dev/null; then
        log_warn "Could not fetch release info from GitHub API."
        echo "v0.1.0"  # Fallback version
    fi
}

# Download and install binary
install_binary() {
    local version arch temp_dir download_url
    
    version=$(get_latest_release)
    arch=$(detect_arch)
    temp_dir=$(mktemp -d)
    
    log_info "Installing Linux Distribution Agent $version for $arch"
    log_info "Install location: $PREFIX"
    
    # For now, we'll build from source since we don't have pre-built releases yet
    log_info "Building from source (pre-built releases coming soon)..."
    
    # Check if we have Rust/Cargo
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo (Rust) is required to build from source."
        log_info "Install Rust from: https://rustup.rs/"
        log_info "Or wait for pre-built releases."
        exit 1
    fi
    
    # Clone and build
    cd "$temp_dir"
    log_info "Cloning repository..."
    git clone "https://github.com/$GITHUB_REPO.git" lda
    cd lda
    
    log_info "Building optimized binary..."
    cargo build --release
    
    # Create destination directory if it doesn't exist
    if [[ ! -d "$PREFIX" ]]; then
        if [[ "$PREFIX" == "/usr/local/bin" ]] || [[ "$PREFIX" == "/usr/bin" ]]; then
            sudo mkdir -p "$PREFIX"
        else
            mkdir -p "$PREFIX"
        fi
    fi
    
    # Copy binary
    log_info "Installing binary to $PREFIX/$BINARY_NAME"
    if [[ "$PREFIX" == "/usr/local/bin" ]] || [[ "$PREFIX" == "/usr/bin" ]]; then
        sudo cp "target/release/$BINARY_NAME" "$PREFIX/"
        sudo chmod +x "$PREFIX/$BINARY_NAME"
    else
        cp "target/release/$BINARY_NAME" "$PREFIX/"
        chmod +x "$PREFIX/$BINARY_NAME"
    fi
    
    # Cleanup
    cd /
    rm -rf "$temp_dir"
    
    log_success "Linux Distribution Agent installed successfully!"
}

# Set up shell alias
setup_alias() {
    local shell_config
    
    # Detect shell and config file
    case $SHELL in
        */bash)
            shell_config="$HOME/.bashrc"
            ;;
        */zsh)
            shell_config="$HOME/.zshrc"
            ;;
        */fish)
            shell_config="$HOME/.config/fish/config.fish"
            ;;
        *)
            log_warn "Unknown shell: $SHELL. Manual alias setup may be required."
            return
            ;;
    esac
    
    # Check if alias already exists
    if [[ -f "$shell_config" ]] && grep -q "alias lda=" "$shell_config"; then
        log_info "LDA alias already exists in $shell_config"
        return
    fi
    
    # Add alias
    if [[ "$PREFIX" != "/usr/local/bin" ]] && [[ "$PREFIX" != "/usr/bin" ]]; then
        echo "" >> "$shell_config"
        echo "# Linux Distribution Agent alias" >> "$shell_config"
        echo "alias lda='$PREFIX/$BINARY_NAME'" >> "$shell_config"
        log_success "Added 'lda' alias to $shell_config"
        log_info "Restart your shell or run: source $shell_config"
    fi
}

# Generate shell completions
setup_completions() {
    local binary_path="$PREFIX/$BINARY_NAME"
    
    if [[ ! -x "$binary_path" ]]; then
        log_warn "Binary not found at $binary_path, skipping completions setup"
        return
    fi
    
    log_info "Setting up shell completions..."
    
    case $SHELL in
        */bash)
            local comp_dir="$HOME/.local/share/bash-completion/completions"
            mkdir -p "$comp_dir"
            "$binary_path" completions bash > "$comp_dir/$BINARY_NAME" 2>/dev/null || true
            log_success "Bash completions installed"
            ;;
        */zsh)
            local comp_dir="$HOME/.local/share/zsh/site-functions"
            mkdir -p "$comp_dir"
            "$binary_path" completions zsh > "$comp_dir/_$BINARY_NAME" 2>/dev/null || true
            log_success "Zsh completions installed"
            ;;
        */fish)
            local comp_dir="$HOME/.config/fish/completions"
            mkdir -p "$comp_dir"
            "$binary_path" completions fish > "$comp_dir/$BINARY_NAME.fish" 2>/dev/null || true
            log_success "Fish completions installed"
            ;;
    esac
}

# Verify installation
verify_installation() {
    local binary_path="$PREFIX/$BINARY_NAME"
    
    if [[ -x "$binary_path" ]]; then
        log_success "Installation verified!"
        log_info "Binary location: $binary_path"
        
        # Show version
        if "$binary_path" --version 2>/dev/null; then
            echo ""
            log_info "Quick start:"
            echo "  $binary_path detect          # Detect your distribution"
            echo "  $binary_path install vim     # Get install commands"
            echo "  $binary_path --help          # Show all commands"
            
            if [[ "$PREFIX" != "/usr/local/bin" ]] && [[ "$PREFIX" != "/usr/bin" ]]; then
                echo ""
                log_info "Since you installed to a custom location, you may want to:"
                echo "  1. Add $PREFIX to your PATH"
                echo "  2. Use the 'lda' alias (added to your shell config)"
            fi
        fi
    else
        log_error "Installation verification failed!"
        exit 1
    fi
}

# Main execution
main() {
    echo -e "${BLUE}Linux Distribution Agent - Quick Install${NC}"
    echo "========================================="
    echo ""
    
    check_platform
    check_dependencies
    install_binary
    setup_alias
    setup_completions
    verify_installation
    
    echo ""
    log_success "Installation complete! 🎉"
}

# Execute main function
main "$@"
