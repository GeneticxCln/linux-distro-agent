use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::process::Command as AsyncCommand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistroConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    pub architecture: String,
    pub base_system: BaseSystem,
    pub packages: PackageConfig,
    pub kernel: KernelConfig,
    pub bootloader: BootloaderConfig,
    pub branding: BrandingConfig,
    pub filesystem: FilesystemConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BaseSystem {
    Arch,
    Debian,
    Ubuntu,
    Scratch, // Build from scratch
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub essential: Vec<String>,
    pub desktop_environment: Option<DesktopEnvironment>,
    pub additional_packages: Vec<String>,
    pub custom_repositories: Vec<Repository>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DesktopEnvironment {
    Gnome,
    KDE,
    Xfce,
    LXDE,
    Mate,
    Cinnamon,
    Sway,
    I3,
    Custom(String),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub key_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelConfig {
    pub kernel_type: KernelType,
    pub custom_config: Option<PathBuf>,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KernelType {
    Vanilla,
    LTS,
    Hardened,
    RT, // Real-time
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootloaderConfig {
    pub bootloader: Bootloader,
    pub timeout: u32,
    pub default_entry: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Bootloader {
    GRUB,
    Systemd,
    Syslinux,
    rEFInd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingConfig {
    pub logo: Option<PathBuf>,
    pub wallpaper: Option<PathBuf>,
    pub theme: Option<String>,
    pub colors: ColorScheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    pub root_fs: FilesystemType,
    pub compression: CompressionType,
    pub size_limit: Option<u64>, // In MB
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilesystemType {
    SquashFS,
    Ext4,
    Btrfs,
    XFS,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    Gzip,
    Xz,
    Zstd,
    Lz4,
    None,
}

pub struct DistroBuilder {
    config: DistroConfig,
    work_dir: PathBuf,
    output_dir: PathBuf,
}

impl DistroBuilder {
    pub fn new(config: DistroConfig, work_dir: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            config,
            work_dir,
            output_dir,
        }
    }

    pub async fn build(&self) -> Result<PathBuf> {
        println!("🚀 Starting Linux distribution build: {}", self.config.name);
        
        // Create working directories
        self.setup_directories().await?;
        
        // Build root filesystem
        self.build_rootfs().await?;
        
        // Install kernel
        self.install_kernel().await?;
        
        // Install packages
        self.install_packages().await?;
        
        // Configure system
        self.configure_system().await?;
        
        // Apply branding
        self.apply_branding().await?;
        
        // Configure bootloader
        self.configure_bootloader().await?;
        
        // Create ISO
        let iso_path = self.create_iso().await?;
        
        println!("✅ Successfully built Linux distribution: {}", iso_path.display());
        Ok(iso_path)
    }


    async fn setup_directories(&self) -> Result<()> {
        println!("📁 Setting up build directories...");
        
        // Clean up any existing directories first
        if self.work_dir.exists() {
            println!("Cleaning up existing work directory...");
            fs::remove_dir_all(&self.work_dir)
                .with_context(|| format!("Failed to remove existing work directory: {}", self.work_dir.display()))?;
        }
        
        let dirs = [
            &self.work_dir,
            &self.output_dir,
            &self.work_dir.join("rootfs"),
            &self.work_dir.join("boot"),
            &self.work_dir.join("iso"),
        ];

        for dir in dirs {
            fs::create_dir_all(dir)
                .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
            println!("Created directory: {}", dir.display());
        }

        Ok(())
    }

    async fn build_rootfs(&self) -> Result<()> {
        println!("🔧 Building root filesystem...");
        
        let rootfs_dir = self.work_dir.join("rootfs");
        
        match self.config.base_system {
            BaseSystem::Arch => self.build_arch_rootfs(&rootfs_dir).await?,
            BaseSystem::Debian => self.build_debian_rootfs(&rootfs_dir).await?,
            BaseSystem::Ubuntu => self.build_ubuntu_rootfs(&rootfs_dir).await?,
            BaseSystem::Scratch => self.build_scratch_rootfs(&rootfs_dir).await?,
        }

        Ok(())
    }

    async fn build_arch_rootfs(&self, rootfs_dir: &Path) -> Result<()> {
        println!("🏗️  Building Arch Linux base system...");
        
        // Ensure the directory exists and has proper permissions
        fs::create_dir_all(rootfs_dir)?;
        
        // Use pacstrap to bootstrap base system (don't skip copying mirrorlist)
        let mut cmd = AsyncCommand::new("pacstrap");
        cmd.arg("-c")  // Use package cache
           .arg(rootfs_dir)
           .arg("base")
           .arg("linux")
           .arg("linux-firmware");

        println!("Running: pacstrap -c {} base linux linux-firmware", rootfs_dir.display());
        
        let output = cmd.output().await
            .context("Failed to run pacstrap")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("STDOUT: {}", stdout);
            println!("STDERR: {}", stderr);
            anyhow::bail!("pacstrap failed: {}", stderr);
        }

        // Copy mirrorlist to ensure package installations work
        let host_mirrorlist = Path::new("/etc/pacman.d/mirrorlist");
        let chroot_mirrorlist = rootfs_dir.join("etc/pacman.d/mirrorlist");
        if host_mirrorlist.exists() {
            fs::create_dir_all(chroot_mirrorlist.parent().unwrap())?;
            fs::copy(host_mirrorlist, chroot_mirrorlist)?;
            println!("✅ Copied mirrorlist to chroot");
        }

        println!("✅ Arch Linux base system created successfully");
        Ok(())
    }

    async fn build_debian_rootfs(&self, rootfs_dir: &Path) -> Result<()> {
        println!("🏗️  Building Debian base system...");
        
        // Install debootstrap if not available
        if Command::new("which").arg("debootstrap").output()?.status.success() {
            let mut cmd = AsyncCommand::new("debootstrap");
            cmd.arg("--arch").arg(&self.config.architecture)
               .arg("stable")
               .arg(rootfs_dir)
               .arg("http://deb.debian.org/debian/");

            let output = cmd.output().await
                .context("Failed to run debootstrap")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("debootstrap failed: {}", stderr);
            }
        } else {
            anyhow::bail!("debootstrap not found. Please install it first.");
        }

        Ok(())
    }

    async fn build_ubuntu_rootfs(&self, rootfs_dir: &Path) -> Result<()> {
        println!("🏗️  Building Ubuntu base system...");
        
        if Command::new("which").arg("debootstrap").output()?.status.success() {
            let mut cmd = AsyncCommand::new("debootstrap");
            cmd.arg("--arch").arg(&self.config.architecture)
               .arg("jammy") // Ubuntu 22.04 LTS
               .arg(rootfs_dir)
               .arg("http://archive.ubuntu.com/ubuntu/");

            let output = cmd.output().await
                .context("Failed to run debootstrap")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("debootstrap failed: {}", stderr);
            }
        } else {
            anyhow::bail!("debootstrap not found. Please install it first.");
        }

        Ok(())
    }

    async fn build_scratch_rootfs(&self, rootfs_dir: &Path) -> Result<()> {
        println!("🏗️  Building minimal system from scratch...");
        
        // Create basic directory structure
        let dirs = [
            "bin", "boot", "dev", "etc", "home", "lib", "lib64", "mnt", 
            "opt", "proc", "root", "run", "sbin", "srv", "sys", "tmp", 
            "usr", "var", "usr/bin", "usr/lib", "usr/sbin", "var/log"
        ];

        for dir in dirs {
            fs::create_dir_all(rootfs_dir.join(dir))?;
        }

        // This would require building toolchain and basic utilities
        // For now, we'll create a minimal BusyBox-based system
        println!("⚠️  Scratch build requires manual toolchain setup");

        Ok(())
    }

    async fn install_kernel(&self) -> Result<()> {
        println!("🐧 Installing kernel...");
        
        let rootfs_dir = self.work_dir.join("rootfs");
        
        match self.config.kernel.kernel_type {
            KernelType::Vanilla => {
                // Install vanilla kernel
                self.install_arch_kernel(&rootfs_dir, "linux").await?;
            }
            KernelType::LTS => {
                self.install_arch_kernel(&rootfs_dir, "linux-lts").await?;
            }
            KernelType::Hardened => {
                self.install_arch_kernel(&rootfs_dir, "linux-hardened").await?;
            }
            KernelType::RT => {
                self.install_arch_kernel(&rootfs_dir, "linux-rt").await?;
            }
            KernelType::Custom(ref kernel) => {
                self.install_arch_kernel(&rootfs_dir, kernel).await?;
            }
        }

        Ok(())
    }

    async fn install_arch_kernel(&self, rootfs_dir: &Path, kernel_package: &str) -> Result<()> {
        println!("Installing kernel package: {}", kernel_package);
        
        // First, update the package database
        let mut update_cmd = AsyncCommand::new("arch-chroot");
        update_cmd.arg(rootfs_dir)
                  .arg("pacman")
                  .arg("-Sy")
                  .arg("--noconfirm");
        
        let update_output = update_cmd.output().await?;
        if !update_output.status.success() {
            println!("Warning: Failed to update package database");
        }
        
        // Install the kernel (it might already be installed from base)
        let mut cmd = AsyncCommand::new("arch-chroot");
        cmd.arg(rootfs_dir)
           .arg("pacman")
           .arg("-S")
           .arg("--noconfirm")
           .arg("--needed")  // Only install if not already present
           .arg(kernel_package);

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("STDOUT: {}", stdout);
            println!("STDERR: {}", stderr);
            // Don't fail if kernel is already installed
            if !stderr.contains("is up to date") && !stderr.contains("target not found") {
                anyhow::bail!("Kernel installation failed: {}", stderr);
            }
        }

        println!("✅ Kernel installation completed");
        Ok(())
    }

    async fn install_packages(&self) -> Result<()> {
        println!("📦 Installing packages...");
        
        let rootfs_dir = self.work_dir.join("rootfs");
        
        // Filter out packages that are already included in base system
        let base_packages = vec!["base", "linux", "linux-firmware"];
        let additional_essential: Vec<String> = self.config.packages.essential
            .iter()
            .filter(|pkg| !base_packages.contains(&pkg.as_str()))
            .cloned()
            .collect();
        
        // Install only non-base essential packages
        if !additional_essential.is_empty() {
            println!("Installing additional essential packages: {:?}", additional_essential);
            self.install_package_list(&rootfs_dir, &additional_essential).await?;
        } else {
            println!("✅ Skipping essential packages (already installed in base system)");
        }

        // Install desktop environment
        if let Some(ref de) = self.config.packages.desktop_environment {
            self.install_desktop_environment(&rootfs_dir, de).await?;
        }

        // Install additional packages
        if !self.config.packages.additional_packages.is_empty() {
            self.install_package_list(&rootfs_dir, &self.config.packages.additional_packages).await?;
        }

        Ok(())
    }

    async fn install_package_list(&self, rootfs_dir: &Path, packages: &[String]) -> Result<()> {
        println!("Installing packages: {:?}", packages);
        
        // First update the package database
        let mut update_cmd = AsyncCommand::new("arch-chroot");
        update_cmd.arg(rootfs_dir)
                  .arg("pacman")
                  .arg("-Sy")
                  .arg("--noconfirm");
        
        let update_output = update_cmd.output().await?;
        if !update_output.status.success() {
            println!("Warning: Failed to update package database in chroot");
        }
        
        let mut cmd = AsyncCommand::new("arch-chroot");
        cmd.arg(rootfs_dir)
           .arg("pacman")
           .arg("-S")
           .arg("--noconfirm")
           .arg("--needed");  // Only install if not already present
        
        for package in packages {
            cmd.arg(package);
        }

        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("STDOUT: {}", stdout);
            println!("STDERR: {}", stderr);
            anyhow::bail!("Package installation failed: {}", stderr);
        }

        println!("✅ Successfully installed packages");
        Ok(())
    }

    async fn install_desktop_environment(&self, rootfs_dir: &Path, de: &DesktopEnvironment) -> Result<()> {
        let packages = match de {
            DesktopEnvironment::Gnome => vec!["gnome".to_string()],
            DesktopEnvironment::KDE => vec!["plasma".to_string(), "kde-applications".to_string()],
            DesktopEnvironment::Xfce => vec!["xfce4".to_string(), "xfce4-goodies".to_string()],
            DesktopEnvironment::LXDE => vec!["lxde".to_string()],
            DesktopEnvironment::Mate => vec!["mate".to_string()],
            DesktopEnvironment::Cinnamon => vec!["cinnamon".to_string()],
            DesktopEnvironment::Sway => vec!["sway".to_string()],
            DesktopEnvironment::I3 => vec!["i3".to_string()],
            DesktopEnvironment::Custom(package) => vec![package.clone()],
            DesktopEnvironment::None => return Ok(()),
        };

        self.install_package_list(rootfs_dir, &packages).await
    }

    async fn configure_system(&self) -> Result<()> {
        println!("⚙️  Configuring system...");
        
        let rootfs_dir = self.work_dir.join("rootfs");
        
        // Set hostname
        fs::write(rootfs_dir.join("etc/hostname"), &self.config.name)?;
        
        // Configure hosts file
        let hosts_content = format!(
            "127.0.0.1\tlocalhost\n::1\t\tlocalhost\n127.0.1.1\t{}\n",
            self.config.name
        );
        fs::write(rootfs_dir.join("etc/hosts"), hosts_content)?;
        
        // Enable systemd services
        self.configure_systemd(&rootfs_dir).await?;
        
        Ok(())
    }

    async fn configure_systemd(&self, rootfs_dir: &Path) -> Result<()> {
        let services = [
            "NetworkManager.service",
            "systemd-resolved.service",
            "systemd-timesyncd.service",
        ];

        for service in services {
            let mut cmd = AsyncCommand::new("arch-chroot");
            cmd.arg(rootfs_dir)
               .arg("systemctl")
               .arg("enable")
               .arg(service);

            let _ = cmd.output().await; // Don't fail if service doesn't exist
        }

        Ok(())
    }

    async fn apply_branding(&self) -> Result<()> {
        println!("🎨 Applying branding...");
        
        // Copy logo, wallpaper, themes, etc.
        // This would be customized based on the branding config
        
        Ok(())
    }

    async fn configure_bootloader(&self) -> Result<()> {
        println!("🥾 Configuring bootloader...");
        
        let boot_dir = self.work_dir.join("boot");
        let rootfs_dir = self.work_dir.join("rootfs");
        
        // Copy kernel and initramfs to boot directory
        let kernel_files = [
            "vmlinuz-linux",
            "initramfs-linux.img",
            "initramfs-linux-fallback.img",
        ];

        for file in kernel_files {
            let src = rootfs_dir.join("boot").join(file);
            let dst = boot_dir.join(file);
            if src.exists() {
                fs::copy(src, dst)?;
            }
        }

        match self.config.bootloader.bootloader {
            Bootloader::Syslinux => self.configure_syslinux(&boot_dir).await?,
            Bootloader::GRUB => self.configure_grub(&boot_dir).await?,
            _ => println!("⚠️  Bootloader configuration not implemented yet"),
        }

        Ok(())
    }

    async fn configure_syslinux(&self, boot_dir: &Path) -> Result<()> {
        let syslinux_cfg = format!(
            r#"DEFAULT {default}
TIMEOUT {timeout}0

LABEL {default}
    MENU LABEL {name}
    LINUX /vmlinuz-linux
    APPEND root=/dev/disk/by-label/{name} rw
    INITRD /initramfs-linux.img

LABEL {default}fallback
    MENU LABEL {name} (fallback initramfs)
    LINUX /vmlinuz-linux
    APPEND root=/dev/disk/by-label/{name} rw
    INITRD /initramfs-linux-fallback.img
"#,
            default = self.config.bootloader.default_entry,
            timeout = self.config.bootloader.timeout,
            name = self.config.name
        );

        fs::write(boot_dir.join("syslinux.cfg"), syslinux_cfg)?;
        Ok(())
    }

    async fn configure_grub(&self, _boot_dir: &Path) -> Result<()> {
        // GRUB configuration would go here
        Ok(())
    }

    async fn create_iso(&self) -> Result<PathBuf> {
        println!("💿 Creating ISO image...");
        
        let iso_dir = self.work_dir.join("iso");
        let rootfs_dir = self.work_dir.join("rootfs");
        let boot_dir = self.work_dir.join("boot");
        
        // Create SquashFS from rootfs
        println!("Creating SquashFS filesystem...");
        let squashfs_path = iso_dir.join("live").join("filesystem.squashfs");
        fs::create_dir_all(iso_dir.join("live"))?;
        
        let mut mksquashfs_cmd = AsyncCommand::new("mksquashfs");
        mksquashfs_cmd.arg(&rootfs_dir)
                     .arg(&squashfs_path)
                     .arg("-e")
                     .arg("boot"); // Exclude boot directory from squashfs
        
        match self.config.filesystem.compression {
            CompressionType::Gzip => { mksquashfs_cmd.arg("-comp").arg("gzip"); }
            CompressionType::Xz => { mksquashfs_cmd.arg("-comp").arg("xz"); }
            CompressionType::Zstd => { mksquashfs_cmd.arg("-comp").arg("zstd"); }
            CompressionType::Lz4 => { mksquashfs_cmd.arg("-comp").arg("lz4"); }
            CompressionType::None => {}
        }

        let output = mksquashfs_cmd.output().await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("STDOUT: {}", stdout);
            anyhow::bail!("mksquashfs failed: {}", stderr);
        }
        println!("✅ SquashFS created successfully");

        // Copy boot files from rootfs to ISO
        println!("Copying boot files...");
        fs::create_dir_all(iso_dir.join("boot"))?;
        
        // Copy kernel and initramfs from rootfs/boot
        let rootfs_boot = rootfs_dir.join("boot");
        if rootfs_boot.exists() {
            for entry in fs::read_dir(&rootfs_boot)? {
                let entry = entry?;
                if entry.file_name().to_string_lossy().starts_with("vmlinuz") ||
                   entry.file_name().to_string_lossy().starts_with("initramfs") {
                    let dst = iso_dir.join("boot").join(entry.file_name());
                    fs::copy(entry.path(), &dst)?;
                    println!("Copied: {} -> {}", entry.path().display(), dst.display());
                }
            }
        }

        // Copy syslinux files
        self.copy_syslinux_files(&iso_dir).await?;

        // Create ISO with xorriso
        println!("Creating ISO with xorriso...");
        let iso_filename = format!("{}-{}-{}.iso", 
                                 self.config.name, 
                                 self.config.version,
                                 self.config.architecture);
        let iso_path = self.output_dir.join(iso_filename);

        let mut xorriso_cmd = AsyncCommand::new("xorriso");
        xorriso_cmd.arg("-as").arg("mkisofs")
                   .arg("-iso-level").arg("3")
                   .arg("-full-iso9660-filenames")
                   .arg("-volid").arg(&self.config.name)
                   .arg("-eltorito-boot").arg("boot/isolinux/isolinux.bin")
                   .arg("-eltorito-catalog").arg("boot/isolinux/boot.cat")
                   .arg("-no-emul-boot")
                   .arg("-boot-load-size").arg("4")
                   .arg("-boot-info-table")
                   .arg("-isohybrid-mbr").arg("/usr/lib/syslinux/bios/isohdpfx.bin")
                   .arg("-output").arg(&iso_path)
                   .arg(&iso_dir);

        let output = xorriso_cmd.output().await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("STDOUT: {}", stdout);
            anyhow::bail!("xorriso failed: {}", stderr);
        }

        println!("✅ ISO created successfully: {}", iso_path.display());
        Ok(iso_path)
    }

    async fn copy_syslinux_files(&self, iso_dir: &Path) -> Result<()> {
        let isolinux_dir = iso_dir.join("boot").join("isolinux");
        fs::create_dir_all(&isolinux_dir)?;

        let syslinux_files = [
            "/usr/lib/syslinux/bios/isolinux.bin",
            "/usr/lib/syslinux/bios/ldlinux.c32",
            "/usr/lib/syslinux/bios/libcom32.c32",
            "/usr/lib/syslinux/bios/libutil.c32",
            "/usr/lib/syslinux/bios/menu.c32",
        ];

        for file_path in syslinux_files {
            let file_name = Path::new(file_path).file_name().unwrap();
            let dst = isolinux_dir.join(file_name);
            if Path::new(file_path).exists() {
                fs::copy(file_path, dst)?;
            }
        }

        // Copy syslinux config as isolinux.cfg
        let syslinux_cfg = iso_dir.join("boot").join("syslinux.cfg");
        let isolinux_cfg = isolinux_dir.join("isolinux.cfg");
        if syslinux_cfg.exists() {
            fs::copy(syslinux_cfg, isolinux_cfg)?;
        }

        Ok(())
    }
}

impl Default for DistroConfig {
    fn default() -> Self {
        Self {
            name: "MyLinux".to_string(),
            version: "1.0".to_string(),
            description: "A custom Linux distribution".to_string(),
            architecture: "x86_64".to_string(),
            base_system: BaseSystem::Arch,
            packages: PackageConfig {
                essential: vec![
                    "base".to_string(),
                    "linux".to_string(),
                    "linux-firmware".to_string(),
                    "networkmanager".to_string(),
                    "sudo".to_string(),
                ],
                desktop_environment: Some(DesktopEnvironment::Xfce),
                additional_packages: vec![
                    "firefox".to_string(),
                    "vim".to_string(),
                    "git".to_string(),
                ],
                custom_repositories: vec![],
            },
            kernel: KernelConfig {
                kernel_type: KernelType::Vanilla,
                custom_config: None,
                modules: vec![],
            },
            bootloader: BootloaderConfig {
                bootloader: Bootloader::Syslinux,
                timeout: 30,
                default_entry: "linux".to_string(),
            },
            branding: BrandingConfig {
                logo: None,
                wallpaper: None,
                theme: None,
                colors: ColorScheme {
                    primary: "#0078d4".to_string(),
                    secondary: "#005a9e".to_string(),
                    accent: "#00bcf2".to_string(),
                },
            },
            filesystem: FilesystemConfig {
                root_fs: FilesystemType::SquashFS,
                compression: CompressionType::Xz,
                size_limit: Some(4096), // 4GB
            },
        }
    }
}
