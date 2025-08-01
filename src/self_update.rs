use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::logger::Logger;

const GITHUB_API_BASE: &str = "https://api.github.com/repos/GeneticxCln/linux-distro-agent";
const DOWNLOAD_TIMEOUT_SECS: u64 = 300; // 5 minutes
const BACKUP_SUFFIX: &str = ".lda-backup";

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub check_interval: u64,           // Hours between automatic checks
    pub auto_update: bool,             // Enable automatic updates
    pub pre_release: bool,             // Include pre-releases
    pub backup_count: usize,           // Number of backups to keep
    pub fallback_to_source: bool,      // Build from source if binary unavailable
    pub verify_signature: bool,        // Verify release signatures (when available)
    pub update_channel: UpdateChannel, // Stable, beta, alpha channels
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UpdateChannel {
    Stable,
    Beta,
    Alpha,
    Nightly,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub published_at: String,
    pub prerelease: bool,
    pub assets: Vec<ReleaseAsset>,
    pub zipball_url: String,
    pub tarball_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    #[serde(rename = "browser_download_url")]
    pub download_url: String,
    pub size: u64,
    pub content_type: String,
}

#[derive(Debug)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub needs_update: bool,
    pub download_url: Option<String>,
    pub release_notes: String,
    pub asset_size: Option<u64>,
    pub is_prerelease: bool,
}

#[derive(Debug)]
pub struct PlatformInfo {
    pub target_triple: String,
    pub binary_name: String,
}

pub struct SelfUpdater {
    config: UpdateConfig,
    logger: Logger,
    client: Client,
    platform: PlatformInfo,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            check_interval: 24,                    // Check daily
            auto_update: false,                    // Manual updates by default
            pre_release: false,                    // Stable releases only
            backup_count: 3,                       // Keep 3 backups
            fallback_to_source: true,              // Build from source as fallback
            verify_signature: false,               // Signature verification disabled until available
            update_channel: UpdateChannel::Stable,
        }
    }
}

impl PlatformInfo {
    pub fn detect() -> Result<Self> {
        let os = if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            return Err(anyhow!("Unsupported operating system"));
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else if cfg!(target_arch = "arm") {
            "arm"
        } else {
            return Err(anyhow!("Unsupported architecture"));
        };

        let target_triple = match (os, arch) {
            ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
            ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
            ("linux", "arm") => "arm-unknown-linux-gnueabihf",
            ("macos", "x86_64") => "x86_64-apple-darwin",
            ("macos", "aarch64") => "aarch64-apple-darwin",
            ("windows", "x86_64") => "x86_64-pc-windows-msvc",
            _ => return Err(anyhow!("Unsupported platform combination: {}-{}", os, arch)),
        };

        let binary_name = if os == "windows" {
            "linux-distro-agent.exe"
        } else {
            "linux-distro-agent"
        };

        Ok(Self {
            target_triple: target_triple.to_string(),
            binary_name: binary_name.to_string(),
        })
    }
}

impl SelfUpdater {
    pub fn new(config: UpdateConfig, logger: Logger) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(DOWNLOAD_TIMEOUT_SECS))
            .user_agent(format!("linux-distro-agent/{}", env!("CARGO_PKG_VERSION")))
            .build()?;

        let platform = PlatformInfo::detect()?;

        Ok(Self {
            config,
            logger,
            client,
            platform,
        })
    }

    /// Check for available updates
    pub async fn check_for_updates(&self) -> Result<UpdateInfo> {
        self.logger.info("Checking for updates...");

        let releases = self.fetch_releases().await?;
        let current_version = env!("CARGO_PKG_VERSION");

        let latest_release = self.find_suitable_release(&releases)?;
        let latest_version = latest_release.tag_name.trim_start_matches('v');

        let needs_update = self.version_needs_update(current_version, latest_version)?;
        
        let download_info = if needs_update {
            self.find_binary_asset(&latest_release)?
        } else {
            None
        };

        Ok(UpdateInfo {
            current_version: current_version.to_string(),
            latest_version: latest_version.to_string(),
            needs_update,
            download_url: download_info.as_ref().map(|(url, _)| url.clone()),
            release_notes: latest_release.body.clone(),
            asset_size: download_info.as_ref().map(|(_, size)| *size),
            is_prerelease: latest_release.prerelease,
        })
    }

    /// Perform the update process
    pub async fn perform_update(&self, force: bool, dry_run: bool) -> Result<()> {
        let update_info = self.check_for_updates().await?;

        if !update_info.needs_update && !force {
            self.logger.success("🎉 You're already running the latest version!");
            return Ok(());
        }

        if dry_run {
            self.logger.info(&format!(
                "[DRY RUN] Would update from {} to {}",
                update_info.current_version, update_info.latest_version
            ));
            if let Some(size) = update_info.asset_size {
                self.logger.info(&format!("[DRY RUN] Download size: {:.2} MB", size as f64 / 1024.0 / 1024.0));
            }
            return Ok(());
        }

        self.logger.info(&format!(
            "🔄 Updating from {} to {}{}",
            update_info.current_version,
            update_info.latest_version,
            if update_info.is_prerelease { " (pre-release)" } else { "" }
        ));

        // Create backup of current binary
        let current_exe = env::current_exe()?;
        let backup_path = self.create_backup(&current_exe)?;
        self.logger.info(&format!("📦 Created backup: {}", backup_path.display()));

        match self.download_and_install_binary(&update_info).await {
            Ok(_) => {
                self.logger.success(&format!(
                    "🎉 Successfully updated to version {}!",
                    update_info.latest_version
                ));
                self.cleanup_old_backups(&current_exe)?;
                
                // Show release notes if available
                if !update_info.release_notes.trim().is_empty() {
                    self.logger.info("📝 Release Notes:");
                    for line in update_info.release_notes.lines().take(10) {
                        self.logger.info(&format!("   {}", line));
                    }
                }
            }
            Err(e) => {
                self.logger.error(&format!("❌ Update failed: {}", e));
                self.logger.info("🔄 Attempting to restore from backup...");
                
                match self.restore_from_backup(&backup_path, &current_exe) {
                    Ok(_) => {
                        self.logger.success("✅ Successfully restored from backup");
                        return Err(anyhow!("Update failed, but backup restored successfully"));
                    }
                    Err(restore_err) => {
                        self.logger.error(&format!("❌ Failed to restore backup: {}", restore_err));
                        return Err(anyhow!("Update failed and backup restoration failed: {}", e));
                    }
                }
            }
        }

        Ok(())
    }

    /// Download and install binary update
    async fn download_and_install_binary(&self, update_info: &UpdateInfo) -> Result<()> {
        if let Some(download_url) = &update_info.download_url {
            self.logger.info("📥 Downloading binary update...");
            
            // Download the binary
            let response = self.client.get(download_url).send().await?;
            if !response.status().is_success() {
                return Err(anyhow!("Failed to download binary: HTTP {}", response.status()));
            }

            let binary_data = response.bytes().await?;
            self.logger.info(&format!("✅ Downloaded {:.2} MB", binary_data.len() as f64 / 1024.0 / 1024.0));

            // Write to temporary file
            let temp_path = self.get_temp_binary_path()?;
            fs::write(&temp_path, &binary_data)?;

            // Make executable (Unix only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&temp_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&temp_path, perms)?;
            }

            // Verify the binary works
            self.verify_binary(&temp_path)?;

            // Replace current binary
            let current_exe = env::current_exe()?;
            self.replace_binary(&temp_path, &current_exe)?;

            Ok(())
        } else if self.config.fallback_to_source {
            self.logger.info("⚠️  No pre-built binary available, falling back to source build...");
            self.build_from_source(update_info).await
        } else {
            Err(anyhow!("No binary available for your platform and source fallback is disabled"))
        }
    }

    /// Build from source as fallback
    async fn build_from_source(&self, update_info: &UpdateInfo) -> Result<()> {
        self.logger.info("🔨 Building from source...");

        // Check if required tools are available
        self.check_build_prerequisites()?;

        // Create temporary directory
        let temp_dir = tempfile::tempdir()?;
        let repo_path = temp_dir.path().join("lda-source");

        // Clone repository
        self.logger.info("📥 Cloning repository...");
        let clone_output = Command::new("git")
            .args(&[
                "clone",
                "--depth", "1",
                "https://github.com/GeneticxCln/linux-distro-agent.git",
                repo_path.to_str().unwrap(),
            ])
            .output()?;

        if !clone_output.status.success() {
            let stderr = String::from_utf8_lossy(&clone_output.stderr);
            return Err(anyhow!("Failed to clone repository: {}", stderr));
        }

        // Checkout the specific tag
        self.logger.info(&format!("🏷️  Checking out tag v{}...", update_info.latest_version));
        let checkout_output = Command::new("git")
            .args(&[
                "checkout",
                &format!("v{}", update_info.latest_version),
            ])
            .current_dir(&repo_path)
            .output()?;

        if !checkout_output.status.success() {
            // If tag doesn't exist, try to use the main branch
            self.logger.warn(&format!("Tag v{} not found, using main branch", update_info.latest_version));
            let checkout_main = Command::new("git")
                .args(&["checkout", "main"])
                .current_dir(&repo_path)
                .output()?;
            
            if !checkout_main.status.success() {
                let stderr = String::from_utf8_lossy(&checkout_output.stderr);
                return Err(anyhow!("Failed to checkout version: {}", stderr));
            }
        }

        // Build the project
        self.logger.info("🔨 Building optimized binary (this may take a few minutes)...");
        let build_output = Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir(&repo_path)
            .output()?;

        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr);
            return Err(anyhow!("Failed to build from source: {}", stderr));
        }

        // Find the built binary
        let built_binary = repo_path.join("target/release").join(&self.platform.binary_name);
        if !built_binary.exists() {
            return Err(anyhow!("Built binary not found at: {}", built_binary.display()));
        }

        // Verify the binary
        self.verify_binary(&built_binary)?;

        // Replace current binary
        let current_exe = env::current_exe()?;
        self.replace_binary(&built_binary, &current_exe)?;

        Ok(())
    }

    /// Fetch releases from GitHub API
    async fn fetch_releases(&self) -> Result<Vec<ReleaseInfo>> {
        let url = format!("{}/releases", GITHUB_API_BASE);
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch releases: HTTP {}", response.status()));
        }

        let releases: Vec<ReleaseInfo> = response.json().await?;
        
        if releases.is_empty() {
            return Err(anyhow!("No releases found"));
        }

        Ok(releases)
    }

    /// Find the most suitable release based on configuration
    fn find_suitable_release<'a>(&self, releases: &'a [ReleaseInfo]) -> Result<&'a ReleaseInfo> {
        let suitable_releases: Vec<&ReleaseInfo> = releases
            .iter()
            .filter(|release| {
                // Filter based on pre-release preference
                if release.prerelease && !self.config.pre_release {
                    return false;
                }

                // Filter based on update channel
                match self.config.update_channel {
                    UpdateChannel::Stable => !release.prerelease,
                    UpdateChannel::Beta => {
                        release.prerelease && (
                            release.tag_name.contains("beta") ||
                            release.tag_name.contains("rc")
                        )
                    }
                    UpdateChannel::Alpha => {
                        release.prerelease && release.tag_name.contains("alpha")
                    }
                    UpdateChannel::Nightly => {
                        release.prerelease && release.tag_name.contains("nightly")
                    }
                }
            })
            .collect();

        suitable_releases
            .first()
            .copied()
            .ok_or_else(|| anyhow!("No suitable release found for the current configuration"))
    }

    /// Find binary asset for current platform
    fn find_binary_asset(&self, release: &ReleaseInfo) -> Result<Option<(String, u64)>> {
        // Look for platform-specific binary
        let platform_patterns = vec![
            format!("{}-{}", self.platform.target_triple, self.platform.binary_name),
            format!("{}.tar.gz", self.platform.target_triple),
            format!("{}.zip", self.platform.target_triple),
            self.platform.target_triple.clone(),
        ];

        for asset in &release.assets {
            for pattern in &platform_patterns {
                if asset.name.contains(pattern) {
                    return Ok(Some((asset.download_url.clone(), asset.size)));
                }
            }
        }

        Ok(None)
    }

    /// Check if version needs update
    fn version_needs_update(&self, current: &str, latest: &str) -> Result<bool> {
        // Simple version comparison (could be enhanced with proper semver)
        if current == latest {
            return Ok(false);
        }

        // Parse versions and compare
        let current_parts: Vec<u32> = current
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        
        let latest_parts: Vec<u32> = latest
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        for (i, &latest_part) in latest_parts.iter().enumerate() {
            let current_part = current_parts.get(i).unwrap_or(&0);
            
            if latest_part > *current_part {
                return Ok(true);
            } else if latest_part < *current_part {
                return Ok(false);
            }
        }

        // If all parts are equal but latest has more parts, it's newer
        Ok(latest_parts.len() > current_parts.len())
    }

    /// Create backup of current binary
    fn create_backup(&self, current_exe: &Path) -> Result<PathBuf> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = current_exe.with_extension(&format!("{}_{}", BACKUP_SUFFIX, timestamp));
        
        // Try direct copy first
        match fs::copy(current_exe, &backup_path) {
            Ok(_) => Ok(backup_path),
            Err(e) => {
                // If direct copy fails due to permissions, try with sudo
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    self.create_backup_with_sudo(current_exe, &backup_path)?;
                    Ok(backup_path)
                } else {
                    Err(anyhow!("Failed to create backup: {}", e))
                }
            }
        }
    }

    /// Create backup using sudo when elevated permissions are required
    #[cfg(unix)]
    fn create_backup_with_sudo(&self, current_exe: &Path, backup_path: &Path) -> Result<()> {
        let status = Command::new("sudo")
            .args(&[
                "cp",
                current_exe.to_str().ok_or_else(|| anyhow!("Invalid source path"))?,
                backup_path.to_str().ok_or_else(|| anyhow!("Invalid backup path"))?
            ])
            .status()?;

        if !status.success() {
            return Err(anyhow!("Failed to create backup with sudo: exit code {}", status));
        }

        Ok(())
    }

    /// Windows stub for create_backup_with_sudo
    #[cfg(windows)]
    fn create_backup_with_sudo(&self, _current_exe: &Path, _backup_path: &Path) -> Result<()> {
        // Windows doesn't need sudo, so this should never be called
        Err(anyhow!("Sudo operations not supported on Windows"))
    }

    /// Windows stub for restore_from_backup_with_sudo
    #[cfg(windows)]
    fn restore_from_backup_with_sudo(&self, _backup_path: &Path, _target_path: &Path) -> Result<()> {
        // Windows doesn't need sudo, so this should never be called
        Err(anyhow!("Sudo operations not supported on Windows"))
    }

    /// Restore from backup
    fn restore_from_backup(&self, backup_path: &Path, target_path: &Path) -> Result<()> {
        // Try direct copy first
        match fs::copy(backup_path, target_path) {
            Ok(_) => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(target_path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(target_path, perms)?;
                }
                Ok(())
            }
            Err(e) => {
                // If direct copy fails due to permissions, try with sudo
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    self.restore_from_backup_with_sudo(backup_path, target_path)
                } else {
                    Err(anyhow!("Failed to restore from backup: {}", e))
                }
            }
        }
    }

    /// Restore from backup using sudo when elevated permissions are required
    #[cfg(unix)]
    fn restore_from_backup_with_sudo(&self, backup_path: &Path, target_path: &Path) -> Result<()> {
        // First, remove the existing binary to avoid "Text file busy" error
        // This is consistent with the update process
        let rm_status = Command::new("sudo")
            .args(&[
                "rm",
                "-f",  // Force removal, don't fail if file doesn't exist
                target_path.to_str().ok_or_else(|| anyhow!("Invalid target path"))?
            ])
            .status()?;

        if !rm_status.success() {
            return Err(anyhow!("Failed to remove existing binary with sudo: exit code {}", rm_status));
        }

        // Use sudo to copy the backup file
        let status = Command::new("sudo")
            .args(&[
                "cp",
                backup_path.to_str().ok_or_else(|| anyhow!("Invalid backup path"))?,
                target_path.to_str().ok_or_else(|| anyhow!("Invalid target path"))?
            ])
            .status()?;

        if !status.success() {
            return Err(anyhow!("Failed to restore backup with sudo: exit code {}", status));
        }

        // Set executable permissions with sudo
        let chmod_status = Command::new("sudo")
            .args(&[
                "chmod",
                "755",
                target_path.to_str().ok_or_else(|| anyhow!("Invalid target path"))?
            ])
            .status()?;

        if !chmod_status.success() {
            return Err(anyhow!("Failed to set permissions on restored binary with sudo: exit code {}", chmod_status));
        }

        Ok(())
    }

    /// Clean up old backups
    fn cleanup_old_backups(&self, current_exe: &Path) -> Result<()> {
        let parent_dir = current_exe.parent().unwrap_or(Path::new("."));
        let base_name = current_exe.file_stem().unwrap_or_default().to_string_lossy();
        
        let mut backups = Vec::new();
        
        for entry in fs::read_dir(parent_dir)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            
            if file_name.starts_with(&format!("{}{}", base_name, BACKUP_SUFFIX)) {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        backups.push((entry.path(), modified));
                    }
                }
            }
        }
        
        // Sort by modification time (newest first)
        backups.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Remove old backups, keeping only the configured number
        for (path, _) in backups.iter().skip(self.config.backup_count) {
            if let Err(e) = fs::remove_file(path) {
                self.logger.warn(&format!("Failed to remove old backup {}: {}", path.display(), e));
            }
        }
        
        Ok(())
    }

    /// Get temporary binary path
    fn get_temp_binary_path(&self) -> Result<PathBuf> {
        let temp_dir = env::temp_dir();
        Ok(temp_dir.join(format!("lda_update_{}", std::process::id())))
    }

    /// Verify that a binary is valid and executable
    fn verify_binary(&self, binary_path: &Path) -> Result<()> {
        let output = Command::new(binary_path)
            .arg("--version")
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Binary verification failed: exit code {}", output.status));
        }

        let version_output = String::from_utf8(output.stdout)?;
        if !version_output.contains("linux-distro-agent") {
            return Err(anyhow!("Binary verification failed: unexpected version output"));
        }

        Ok(())
    }

    /// Replace current binary with new one
    fn replace_binary(&self, new_binary: &Path, target_path: &Path) -> Result<()> {
        // On Windows, we might need special handling for replacing running executables
        #[cfg(windows)]
        {
            // Try to copy directly first
            match fs::copy(new_binary, target_path) {
                Ok(_) => return Ok(()),
                Err(_) => {
                    // If direct copy fails, try the move-and-replace method
                    let temp_name = format!("{}.old", target_path.display());
                    fs::rename(target_path, &temp_name)?;
                    fs::copy(new_binary, target_path)?;
                    let _ = fs::remove_file(temp_name); // Ignore error if we can't remove old file
                    return Ok(());
                }
            }
        }

        // Unix systems
        #[cfg(unix)]
        {
            // First try direct copy (works if we have write permissions)
            match fs::copy(new_binary, target_path) {
                Ok(_) => {
                    // Ensure executable permissions
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(target_path)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(target_path, perms)?;
                    Ok(())
                }
                Err(e) => {
                    // If direct copy fails due to permissions, try with sudo
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        self.logger.info("🔐 Elevated permissions required. Please enter your password:");
                        self.replace_binary_with_sudo(new_binary, target_path)
                    } else {
                        Err(anyhow!("Failed to replace binary: {}", e))
                    }
                }
            }
        }
    }

    /// Replace binary using sudo when elevated permissions are required
    #[cfg(unix)]
    fn replace_binary_with_sudo(&self, new_binary: &Path, target_path: &Path) -> Result<()> {
        // First, remove the existing binary to avoid "Text file busy" error
        // This is safer than trying to overwrite a potentially running binary
        let rm_status = Command::new("sudo")
            .args(&[
                "rm",
                "-f",  // Force removal, don't fail if file doesn't exist
                target_path.to_str().ok_or_else(|| anyhow!("Invalid target path"))?
            ])
            .status()?;

        if !rm_status.success() {
            return Err(anyhow!("Failed to remove existing binary with sudo: exit code {}", rm_status));
        }

        // Now copy the new binary
        let status = Command::new("sudo")
            .args(&[
                "cp",
                new_binary.to_str().ok_or_else(|| anyhow!("Invalid path"))?,
                target_path.to_str().ok_or_else(|| anyhow!("Invalid target path"))?
            ])
            .status()?;

        if !status.success() {
            return Err(anyhow!("Failed to copy binary with sudo: exit code {}", status));
        }

        // Set executable permissions with sudo
        let chmod_status = Command::new("sudo")
            .args(&[
                "chmod",
                "755",
                target_path.to_str().ok_or_else(|| anyhow!("Invalid target path"))?
            ])
            .status()?;

        if !chmod_status.success() {
            return Err(anyhow!("Failed to set permissions with sudo: exit code {}", chmod_status));
        }

        self.logger.success("✅ Binary updated successfully with elevated permissions");
        Ok(())
    }

    /// Check build prerequisites for source fallback
    fn check_build_prerequisites(&self) -> Result<()> {
        let tools = vec!["git", "cargo", "rustc"];
        
        for tool in tools {
            if Command::new(tool).arg("--version").output().is_err() {
                return Err(anyhow!("Required tool '{}' not found. Please install Rust toolchain.", tool));
            }
        }
        
        Ok(())
    }

}

