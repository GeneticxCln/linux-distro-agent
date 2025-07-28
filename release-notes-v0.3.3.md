# Linux Distribution Agent v0.3.3 - Enhanced Self-Update System

## 🚀 Major Self-Update System Overhaul

This release introduces a completely redesigned self-update system that replaces the previous basic bash script approach with a robust, feature-rich Rust-based updater.

### ✨ New Features

#### 🔄 Enhanced Self-Update Capabilities
- **Pre-built Binary Support**: Downloads optimized pre-built binaries from GitHub releases
- **Intelligent Fallback**: Automatically falls back to source compilation when binaries aren't available
- **Platform Detection**: Automatically detects OS and architecture (Linux x86_64, aarch64, arm; macOS; Windows)
- **Update Channels**: Support for stable, beta, alpha, and nightly release channels
- **Binary Verification**: Validates downloaded binaries before installation

#### 🛡️ Safety and Reliability
- **Backup System**: Creates timestamped backups before updates
- **Rollback Support**: Automatic restoration from backup on update failure
- **Atomic Updates**: Safe binary replacement to prevent corruption
- **Integrity Checks**: Verifies binary functionality before replacement
- **Error Recovery**: Comprehensive error handling with automatic rollback

#### 🎛️ New Command-Line Options
- `--check`: Check for updates without installing
- `--pre-release`: Include pre-release versions in update checks
- `--channel <CHANNEL>`: Specify update channel (stable/beta/alpha/nightly)
- `--config`: Show current update configuration
- `--dry-run`: Enhanced dry-run mode with detailed update information
- `--force`: Force reinstallation of current version

### 🔧 Technical Improvements

#### Performance and Reliability
- **Async HTTP Client**: Uses reqwest for reliable GitHub API communication
- **Memory Efficient**: Streams large downloads instead of loading entirely into memory
- **Timeout Handling**: Configurable timeouts for network operations
- **Version Comparison**: Proper semantic version comparison for update detection
- **Cross-Platform**: Works seamlessly on Linux, macOS, and Windows

#### Configuration and Management
- **Configurable Behavior**: Extensive configuration options for update behavior
- **Backup Retention**: Configurable number of backups to keep (default: 3)
- **Prerequisite Checking**: Validates build tools availability for source fallback
- **Progress Feedback**: Clear progress indicators and status messages

### 📋 Usage Examples

```bash
# Check for updates without installing
lda self-update --check

# Show current update configuration
lda self-update --config

# Update using beta channel
lda self-update --channel beta

# Check for pre-release versions
lda self-update --check --pre-release

# Dry run to see what would be updated
lda self-update --dry-run

# Force update even if on latest version
lda self-update --force
```

### 🏗️ Implementation Details

- **New Module**: Added `src/self_update.rs` with comprehensive update functionality
- **Enhanced CLI**: Extended command-line interface with new options
- **Improved Logging**: Better progress reporting and error messages
- **Modular Design**: Clean separation of concerns for maintainability

### 🔒 Security Enhancements

- Binary integrity verification
- Safe temporary file handling
- Secure backup and restore operations
- Protection against partial updates

---

## 🛠️ Installation

### From Source
```bash
git clone https://github.com/GeneticxCln/linux-distro-agent.git
cd linux-distro-agent
cargo build --release
```

### Using the Install Script
```bash
curl -sSL https://raw.githubusercontent.com/GeneticxCln/linux-distro-agent/main/install.sh | bash
```

### Update Existing Installation
```bash
lda self-update
```

---

## 🆕 What's New Since v0.3.2

- Complete self-update system redesign
- Pre-built binary download support
- Enhanced command-line interface
- Improved error handling and recovery
- Cross-platform compatibility improvements
- Comprehensive backup and rollback system

---

## 📊 Compatibility

- **Linux**: x86_64, aarch64, arm (GNU libc)
- **macOS**: x86_64, Apple Silicon (M1/M2)
- **Windows**: x86_64 (MSVC)

---

## 🐛 Bug Reports & Feedback

If you encounter any issues with the new self-update system, please [open an issue](https://github.com/GeneticxCln/linux-distro-agent/issues) with:
- Your operating system and architecture
- LDA version (`lda --version`)
- Command that failed
- Error message or unexpected behavior

---

**Full Changelog**: https://github.com/GeneticxCln/linux-distro-agent/compare/v0.3.2...v0.3.3
