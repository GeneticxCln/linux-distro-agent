# Linux Distribution Agent - Command Reference

This document provides a comprehensive reference for all Linux Distribution Agent (LDA) commands.

## Global Options

All commands support these global options:

- `-v, --verbose` - Enable verbose output
- `-q, --quiet` - Quiet mode - suppress non-essential output  
- `-h, --help` - Show help information
- `-V, --version` - Show version information

## Package Management Commands

### `detect`

Detect and display current Linux distribution information.

**Syntax:**
```bash
lda detect [OPTIONS]
```

**Options:**
- `-e, --extended` - Show extended information including URLs and additional metadata

**Examples:**
```bash
# Basic detection
lda detect

# Extended information
lda detect --extended
```

### `install`

Generate or execute package installation commands.

**Syntax:**
```bash
lda install <PACKAGE> [OPTIONS]
```

**Options:**
- `-e, --execute` - Execute the command directly (requires confirmation)

**Examples:**
```bash
# Show install command
lda install firefox

# Execute install command directly  
lda install firefox --execute

# Install multiple packages
lda install "git vim curl" --execute
```

### `search`

Search for packages using the appropriate package manager.

**Syntax:**
```bash
lda search <QUERY> [OPTIONS]
```

**Options:**
- `-e, --execute` - Execute the search command directly

**Examples:**
```bash
# Search for packages
lda search "text editor"

# Execute search directly
lda search python --execute
```

### `remove`

Generate or execute package removal commands.

**Syntax:**
```bash
lda remove <PACKAGE> [OPTIONS]
```

**Options:**
- `-e, --execute` - Execute the command directly (requires confirmation)

**Examples:**
```bash
# Show remove command
lda remove firefox

# Execute remove command directly
lda remove firefox --execute
```

### `update`

Generate or execute system update commands.

**Syntax:**
```bash
lda update [OPTIONS]
```

**Options:**
- `-e, --execute` - Execute the command directly (requires confirmation)

**Examples:**
```bash
# Show update command
lda update

# Execute update directly
lda update --execute
```

### `list`

List installed packages with optional filtering.

**Syntax:**
```bash
lda list [OPTIONS]
```

**Options:**
- `-d, --detailed` - Show detailed package information
- `-f, --filter <PATTERN>` - Filter packages by name pattern

**Examples:**
```bash
# List all packages
lda list

# List with details
lda list --detailed

# Filter packages
lda list --filter "python"
```

### `package-info`

Show detailed information about a specific package.

**Syntax:**
```bash
lda package-info <PACKAGE>
```

**Examples:**
```bash
# Get package information
lda package-info firefox
```

## Information Commands

### `info`

Display comprehensive system information as JSON.

**Syntax:**
```bash
lda info [OPTIONS]
```

**Options:**
- `-p, --pretty` - Pretty print JSON output

**Examples:**
```bash
# Basic system info
lda info

# Pretty printed JSON
lda info --pretty
```

### `list-supported`

List all supported distributions and package managers.

**Syntax:**
```bash
lda list-supported
```

### `doctor`

Check system compatibility and provide recommendations.

**Syntax:**
```bash
lda doctor
```

## Linux Distribution Builder Commands

### `build-distro`

Build a custom Linux distribution ISO.

**Syntax:**
```bash
sudo lda build-distro [OPTIONS]
```

**Options:**
- `-n, --name <NAME>` - Distribution name
- `-c, --config <CONFIG>` - Configuration file path
- `-w, --work-dir <DIR>` - Work directory for build process
- `-o, --output-dir <DIR>` - Output directory for the ISO
- `--minimal` - Use default minimal configuration

**Examples:**
```bash
# Build with minimal config
sudo lda build-distro --minimal

# Build with custom config
sudo lda build-distro -c my-distro.toml

# Custom build locations
sudo lda build-distro -c config.toml -w /var/build -o ~/isos

# Named distribution
sudo lda build-distro --minimal --name "MyCustomLinux"
```

### `generate-config`

Generate a distro configuration template.

**Syntax:**
```bash
lda generate-config [OPTIONS]
```

**Options:**
- `-o, --output <FILE>` - Output file path
- `--template <TYPE>` - Configuration template type (default: minimal)

**Examples:**
```bash
# Generate to stdout
lda generate-config

# Generate to file
lda generate-config -o my-distro.toml

# Generate minimal template
lda generate-config --template minimal
```

## Management Commands

### `history`

Manage command history.

**Syntax:**
```bash
lda history [OPTIONS]
```

**Options:**
- `-l, --limit <N>` - Number of recent entries to show (default: 10)
- `-s, --search <TERM>` - Search history for specific terms
- `--clear` - Clear command history

**Examples:**
```bash
# Show recent history
lda history

# Show more entries
lda history --limit 20

# Search history
lda history --search "install"

# Clear history
lda history --clear
```

### `config`

Configuration management.

**Syntax:**
```bash
lda config <SUBCOMMAND>
```

**Subcommands:**
- `show` - Show current configuration
- `edit` - Edit configuration in default editor
- `reset` - Reset configuration to defaults
- `set <KEY> <VALUE>` - Set a configuration value

**Examples:**
```bash
# Show current config
lda config show

# Edit config file
lda config edit

# Set a config value
lda config set cache_duration 600

# Reset to defaults
lda config reset
```

### `cache`

Cache management.

**Syntax:**
```bash
lda cache <SUBCOMMAND>
```

**Subcommands:**
- `status` - Show cache status
- `clear` - Clear all cached data
- `list` - Show cached entries

**Examples:**
```bash
# Show cache status
lda cache status

# List cache entries
lda cache list

# Clear cache
lda cache clear
```

## Utility Commands

### `completions`

Generate shell completion scripts.

**Syntax:**
```bash
lda completions <SHELL>
```

**Supported shells:** `bash`, `zsh`, `fish`, `powershell`

**Examples:**
```bash
# Generate for bash
lda completions bash >> ~/.bashrc

# Generate for zsh
lda completions zsh >> ~/.zshrc

# Generate for fish
lda completions fish > ~/.config/fish/completions/lda.fish
```

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Command line argument error
- `3` - Configuration error
- `4` - System compatibility error
- `5` - Package manager error

## Environment Variables

- `LDA_CONFIG_DIR` - Override configuration directory
- `LDA_CACHE_DIR` - Override cache directory
- `EDITOR` - Default editor for config editing
