use std::process::Command;
use std::io::{self, Write};
use anyhow::{Result, Context};

pub struct CommandExecutor;

impl CommandExecutor {
    pub fn execute_command(command: &str, requires_confirmation: bool) -> Result<bool> {
        if requires_confirmation
            && !Self::confirm_execution(command)? {
                println!("Command execution cancelled by user.");
                return Ok(false);
            }

        println!("Executing: {command}");
        
        let output = if command.contains("&&") {
            // Handle compound commands with shell
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .with_context(|| format!("Failed to execute command: {command}"))?
        } else {
            // Handle simple commands
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.is_empty() {
                return Err(anyhow::anyhow!("Empty command"));
            }

            let mut cmd = Command::new(parts[0]);
            for arg in &parts[1..] {
                cmd.arg(arg);
            }

            cmd.output()
                .with_context(|| format!("Failed to execute command: {command}"))?
        };

        // Print stdout
        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }

        // Print stderr
        if !output.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
        }

        if output.status.success() {
            println!("Command executed successfully.");
            Ok(true)
        } else {
            let exit_code = output.status.code().unwrap_or(-1);
            println!("Command failed with exit code: {exit_code}");
            Ok(false)
        }
    }

    fn confirm_execution(command: &str) -> Result<bool> {
        print!("Do you want to execute the following command? [y/N]: {command}\n> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim().to_lowercase();
        Ok(matches!(input.as_str(), "y" | "yes"))
    }

    pub fn is_safe_to_execute(command: &str) -> bool {
        // Define patterns that are generally safe to execute
        let safe_patterns = [
            "pacman -Ss",    // search packages
            "apt search",    // search packages
            "dnf search",    // search packages
            "zypper search", // search packages
            "emerge --search", // search packages
            "nix-env -qaP | grep", // search packages
            "apk search",    // search packages
        ];

        // Check if command starts with any safe pattern
        safe_patterns.iter().any(|pattern| command.starts_with(pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_to_execute() {
        assert!(CommandExecutor::is_safe_to_execute("pacman -Ss vim"));
        assert!(CommandExecutor::is_safe_to_execute("apt search git"));
        assert!(!CommandExecutor::is_safe_to_execute("sudo rm -rf /"));
        assert!(!CommandExecutor::is_safe_to_execute("sudo pacman -S vim"));
    }
}
