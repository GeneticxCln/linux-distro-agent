# Shell Completions

`linux-distro-agent` supports shell completions for bash, zsh, fish, PowerShell, and elvish. This document explains how to generate and install completion scripts for your shell.

## Generating Completions

To generate completion scripts, use the `completions` subcommand:

```bash
linux-distro-agent completions [SHELL]
```

Where `[SHELL]` is one of: `bash`, `zsh`, `fish`, `powershell`, or `elvish`.

## Installation Instructions

### Bash

#### System-wide installation (requires root):
```bash
linux-distro-agent completions bash | sudo tee /etc/bash_completion.d/linux-distro-agent
```

#### User-specific installation:
```bash
# Create the directory if it doesn't exist
mkdir -p ~/.local/share/bash-completion/completions

# Generate and save the completion script
linux-distro-agent completions bash > ~/.local/share/bash-completion/completions/linux-distro-agent
```

#### Temporary installation (current session only):
```bash
source <(linux-distro-agent completions bash)
```

### Zsh

#### System-wide installation (requires root):
```bash
linux-distro-agent completions zsh | sudo tee /usr/share/zsh/site-functions/_linux-distro-agent
```

#### User-specific installation:
```bash
# Create the directory if it doesn't exist
mkdir -p ~/.local/share/zsh/site-functions

# Generate and save the completion script
linux-distro-agent completions zsh > ~/.local/share/zsh/site-functions/_linux-distro-agent

# Add the directory to your fpath in ~/.zshrc if not already present
echo 'fpath=(~/.local/share/zsh/site-functions $fpath)' >> ~/.zshrc

# Reload completions
autoload -U compinit && compinit
```

#### Using Oh My Zsh:
```bash
# Create the completions directory if it doesn't exist
mkdir -p ~/.oh-my-zsh/completions

# Generate and save the completion script
linux-distro-agent completions zsh > ~/.oh-my-zsh/completions/_linux-distro-agent

# Reload your shell or run:
exec zsh
```

#### Temporary installation (current session only):
```bash
source <(linux-distro-agent completions zsh)
```

### Fish

#### System-wide installation (requires root):
```bash
linux-distro-agent completions fish | sudo tee /usr/share/fish/vendor_completions.d/linux-distro-agent.fish
```

#### User-specific installation:
```bash
# Create the directory if it doesn't exist
mkdir -p ~/.config/fish/completions

# Generate and save the completion script
linux-distro-agent completions fish > ~/.config/fish/completions/linux-distro-agent.fish
```

#### Temporary installation (current session only):
```bash
linux-distro-agent completions fish | source
```

### PowerShell

#### For the current user:
```powershell
# Create the profile directory if it doesn't exist
if (!(Test-Path -Path $PROFILE)) {
    New-Item -ItemType File -Path $PROFILE -Force
}

# Add the completion script to your profile
linux-distro-agent completions powershell | Add-Content $PROFILE
```

#### Temporary installation (current session only):
```powershell
linux-distro-agent completions powershell | Invoke-Expression
```

### Elvish

#### User-specific installation:
```bash
# Create the directory if it doesn't exist
mkdir -p ~/.config/elvish/lib

# Generate and save the completion script
linux-distro-agent completions elvish > ~/.config/elvish/lib/linux-distro-agent.elv

# Add this line to your ~/.config/elvish/rc.elv:
# use ./lib/linux-distro-agent
```

## Testing Completions

After installing completions, you can test them by typing:

```bash
linux-distro-agent [TAB][TAB]
```

You should see available subcommands like:
- `detect`
- `install`
- `search`
- `update`
- `info`
- `list-supported`
- `doctor`
- `remove`
- `completions`

For subcommands with options, you can also test:

```bash
linux-distro-agent install [TAB][TAB]  # Should show available options
linux-distro-agent install --[TAB][TAB]  # Should show --execute and --help options
```

## Troubleshooting

### Completions not working

1. **Verify installation**: Make sure the completion script is in the correct location for your shell.

2. **Check permissions**: Ensure the completion script has read permissions:
   ```bash
   ls -la /path/to/completion/script
   ```

3. **Reload your shell**: 
   - For bash: `exec bash` or `source ~/.bashrc`
   - For zsh: `exec zsh` or `source ~/.zshrc`
   - For fish: `exec fish`

4. **Check shell configuration**: Some shells require additional configuration to load completions.

### Bash-specific issues

- Ensure bash-completion is installed:
  ```bash
  # On Ubuntu/Debian
  sudo apt install bash-completion
  
  # On CentOS/RHEL/Fedora
  sudo dnf install bash-completion  # or yum install bash-completion
  
  # On Arch Linux
  sudo pacman -S bash-completion
  ```

### Zsh-specific issues

- Make sure the completion directory is in your `fpath`:
  ```bash
  echo $fpath
  ```

- Try rebuilding the completion cache:
  ```bash
  rm -f ~/.zcompdump && compinit
  ```

### Fish-specific issues

- Check if completions are being loaded:
  ```bash
  complete -c linux-distro-agent
  ```

## Updating Completions

When you update `linux-distro-agent`, regenerate and reinstall the completion scripts to ensure they include any new commands or options:

```bash
# Re-run the installation command for your shell
# For example, for zsh user-specific installation:
linux-distro-agent completions zsh > ~/.local/share/zsh/site-functions/_linux-distro-agent
```

## Advanced Usage

### Custom Installation Locations

You can install completions to custom locations by modifying the paths in the installation commands above. Just make sure your shell knows to look in those locations.

### Multiple Shell Support

You can install completions for multiple shells simultaneously. Each shell will use its own completion script format.

### Integration with Package Managers

If you're packaging `linux-distro-agent` for distribution, consider including the completion scripts in separate packages (e.g., `linux-distro-agent-bash-completion`, `linux-distro-agent-zsh-completion`, etc.).
