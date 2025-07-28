use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureInfo {
    pub signature_type: SignatureType,
    pub key_id: String,
    pub fingerprint: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub valid: bool,
    pub trust_level: TrustLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureType {
    GPG,
    RSA,
    ECDSA,
    Ed25519,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustLevel {
    Unknown,
    Never,
    Marginal,
    Full,
    Ultimate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedKey {
    pub key_id: String,
    pub fingerprint: String,
    pub owner: String,
    pub email: String,
    pub trust_level: TrustLevel,
    pub expiry: Option<chrono::DateTime<chrono::Utc>>,
    pub added_date: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningPolicy {
    pub require_signature: bool,
    pub allowed_signature_types: Vec<SignatureType>,
    pub minimum_trust_level: TrustLevel,
    pub allow_expired_keys: bool,
    pub verify_chain: bool,
    pub repositories: HashMap<String, RepositorySigningConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySigningConfig {
    pub required_keys: Vec<String>,
    pub keyring_path: Option<PathBuf>,
    pub signature_verification: bool,
    pub trust_level_override: Option<TrustLevel>,
}

pub struct SigningVerificationManager {
    config_path: PathBuf,
    keyring_path: PathBuf,
    trusted_keys: HashMap<String, TrustedKey>,
    signing_policy: SigningPolicy,
}

impl Default for SigningPolicy {
    fn default() -> Self {
        Self {
            require_signature: true,
            allowed_signature_types: vec![
                SignatureType::GPG,
                SignatureType::RSA,
                SignatureType::ECDSA,
                SignatureType::Ed25519,
            ],
            minimum_trust_level: TrustLevel::Marginal,
            allow_expired_keys: false,
            verify_chain: true,
            repositories: HashMap::new(),
        }
    }
}

impl SigningVerificationManager {
    pub fn new(config_dir: &Path) -> Result<Self> {
        let config_path = config_dir.join("signing_config.json");
        let keyring_path = config_dir.join("keyrings");
        
        fs::create_dir_all(&keyring_path)?;
        
        let mut manager = Self {
            config_path,
            keyring_path,
            trusted_keys: HashMap::new(),
            signing_policy: SigningPolicy::default(),
        };
        
        manager.load_config()?;
        Ok(manager)
    }

    pub fn load_config(&mut self) -> Result<()> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)?;
            let config: serde_json::Value = serde_json::from_str(&content)?;
            
            if let Some(keys) = config.get("trusted_keys") {
                self.trusted_keys = serde_json::from_value(keys.clone())?;
            }
            
            if let Some(policy) = config.get("signing_policy") {
                self.signing_policy = serde_json::from_value(policy.clone())?;
            }
        }
        Ok(())
    }

    pub fn save_config(&self) -> Result<()> {
        let config = serde_json::json!({
            "trusted_keys": self.trusted_keys,
            "signing_policy": self.signing_policy
        });
        
        fs::write(&self.config_path, serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }

    pub fn verify_package_signature(&self, package_path: &Path, signature_path: Option<&Path>) -> Result<SignatureInfo> {
        println!("Verifying signature for package: {}", package_path.display());
        
        // Try different verification methods based on available tools
        if let Ok(gpg_result) = self.verify_with_gpg(package_path, signature_path) {
            return Ok(gpg_result);
        }
        
        if let Ok(rpm_result) = self.verify_rpm_signature(package_path) {
            return Ok(rpm_result);
        }
        
        if let Ok(deb_result) = self.verify_deb_signature(package_path) {
            return Ok(deb_result);
        }
        
        // Default to unknown signature
        Ok(SignatureInfo {
            signature_type: SignatureType::GPG,
            key_id: "unknown".to_string(),
            fingerprint: "unknown".to_string(),
            timestamp: chrono::Utc::now(),
            valid: false,
            trust_level: TrustLevel::Unknown,
        })
    }

    fn verify_with_gpg(&self, package_path: &Path, signature_path: Option<&Path>) -> Result<SignatureInfo> {
        let mut cmd = Command::new("gpg");
        cmd.arg("--verify");
        cmd.arg("--status-fd").arg("1");
        
        if let Some(sig_path) = signature_path {
            cmd.arg(sig_path);
            cmd.arg(package_path);
        } else {
            cmd.arg(package_path);
        }
        
        let output = cmd.output()?;
        let status_output = String::from_utf8_lossy(&output.stdout);
        
        self.parse_gpg_output(&status_output)
    }

    fn parse_gpg_output(&self, output: &str) -> Result<SignatureInfo> {
        let mut signature_info = SignatureInfo {
            signature_type: SignatureType::GPG,
            key_id: String::new(),
            fingerprint: String::new(),
            timestamp: chrono::Utc::now(),
            valid: false,
            trust_level: TrustLevel::Unknown,
        };
        
        for line in output.lines() {
            if line.starts_with("[GNUPG:] GOODSIG") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 2 {
                    signature_info.key_id = parts[2].to_string();
                    signature_info.valid = true;
                }
            } else if line.starts_with("[GNUPG:] VALIDSIG") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 2 {
                    signature_info.fingerprint = parts[2].to_string();
                }
            } else if line.starts_with("[GNUPG:] TRUST_") {
                signature_info.trust_level = self.parse_trust_level(line);
            }
        }
        
        Ok(signature_info)
    }

    fn parse_trust_level(&self, line: &str) -> TrustLevel {
        if line.contains("TRUST_ULTIMATE") {
            TrustLevel::Ultimate
        } else if line.contains("TRUST_FULLY") {
            TrustLevel::Full
        } else if line.contains("TRUST_MARGINAL") {
            TrustLevel::Marginal
        } else if line.contains("TRUST_NEVER") {
            TrustLevel::Never
        } else {
            TrustLevel::Unknown
        }
    }

    fn verify_rpm_signature(&self, package_path: &Path) -> Result<SignatureInfo> {
        let output = Command::new("rpm")
            .arg("-K")
            .arg(package_path)
            .output()?;
        
        let result = String::from_utf8_lossy(&output.stdout);
        
        // Parse RPM signature verification output
        let valid = result.contains("OK") && !result.contains("NOT OK");
        
        Ok(SignatureInfo {
            signature_type: SignatureType::RSA,
            key_id: "rpm-signature".to_string(),
            fingerprint: "unknown".to_string(),
            timestamp: chrono::Utc::now(),
            valid,
            trust_level: if valid { TrustLevel::Full } else { TrustLevel::Unknown },
        })
    }

    fn verify_deb_signature(&self, package_path: &Path) -> Result<SignatureInfo> {
        let output = Command::new("dpkg-sig")
            .arg("--verify")
            .arg(package_path)
            .output()?;
        
        let result = String::from_utf8_lossy(&output.stdout);
        let valid = output.status.success() && !result.contains("NOSIG");
        
        Ok(SignatureInfo {
            signature_type: SignatureType::GPG,
            key_id: "deb-signature".to_string(),
            fingerprint: "unknown".to_string(),
            timestamp: chrono::Utc::now(),
            valid,
            trust_level: if valid { TrustLevel::Full } else { TrustLevel::Unknown },
        })
    }

    pub fn add_trusted_key(&mut self, key_file: &Path, owner: &str, email: &str) -> Result<()> {
        println!("Adding trusted key from: {}", key_file.display());
        
        // Import key using GPG
        let output = Command::new("gpg")
            .arg("--import")
            .arg(key_file)
            .output()?;
        
        if !output.status.success() {
return Err(anyhow::anyhow!("Failed to import GPG key: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        // Extract key information
        let list_output = Command::new("gpg")
            .arg("--list-keys")
            .arg("--with-fingerprint")
            .arg("--with-colons")
            .arg(email)
            .output()?;
        
        let key_info = String::from_utf8_lossy(&list_output.stdout);
        let (key_id, fingerprint) = self.parse_key_info(&key_info)?;
        
        let trusted_key = TrustedKey {
            key_id: key_id.clone(),
            fingerprint,
            owner: owner.to_string(),
            email: email.to_string(),
            trust_level: TrustLevel::Full,
            expiry: None, // TODO: Parse expiry from GPG output
            added_date: chrono::Utc::now(),
        };
        
        self.trusted_keys.insert(key_id, trusted_key);
        self.save_config()?;
        
        println!("Successfully added trusted key for {}", email);
        Ok(())
    }

    fn parse_key_info(&self, gpg_output: &str) -> Result<(String, String)> {
        let mut key_id = String::new();
        let mut fingerprint = String::new();
        
        for line in gpg_output.lines() {
            if line.starts_with("pub:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 4 {
                    key_id = parts[4].to_string();
                }
            } else if line.starts_with("fpr:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 9 {
                    fingerprint = parts[9].to_string();
                }
            }
        }
        
        if key_id.is_empty() || fingerprint.is_empty() {
return Err(anyhow::anyhow!("Failed to parse key information"));
        }
        
        Ok((key_id, fingerprint))
    }

    pub fn remove_trusted_key(&mut self, key_id: &str) -> Result<()> {
        if self.trusted_keys.remove(key_id).is_some() {
            self.save_config()?;
            println!("Removed trusted key: {}", key_id);
            Ok(())
        } else {
Err(anyhow::anyhow!("Key not found: {}", key_id))
        }
    }

    pub fn list_trusted_keys(&self) -> Vec<&TrustedKey> {
        self.trusted_keys.values().collect()
    }

    pub fn configure_repository_signing(&mut self, repo_name: &str, config: RepositorySigningConfig) -> Result<()> {
        self.signing_policy.repositories.insert(repo_name.to_string(), config);
        self.save_config()?;
        println!("Updated signing configuration for repository: {}", repo_name);
        Ok(())
    }

    pub fn update_signing_policy(&mut self, policy: SigningPolicy) -> Result<()> {
        self.signing_policy = policy;
        self.save_config()?;
        println!("Updated global signing policy");
        Ok(())
    }

    pub fn verify_repository_metadata(&self, repo_name: &str, metadata_path: &Path) -> Result<bool> {
        println!("Verifying repository metadata for: {}", repo_name);
        
        if let Some(repo_config) = self.signing_policy.repositories.get(repo_name) {
            if !repo_config.signature_verification {
                return Ok(true); // Verification disabled for this repo
            }
            
            // Check for Release.gpg or InRelease files (Debian/Ubuntu style)
            let release_gpg = metadata_path.parent().unwrap().join("Release.gpg");
            let in_release = metadata_path.parent().unwrap().join("InRelease");
            
            if release_gpg.exists() {
                let sig_info = self.verify_with_gpg(metadata_path, Some(&release_gpg))?;
                return Ok(sig_info.valid && self.is_key_trusted(&sig_info.key_id));
            } else if in_release.exists() {
                let sig_info = self.verify_with_gpg(&in_release, None)?;
                return Ok(sig_info.valid && self.is_key_trusted(&sig_info.key_id));
            }
            
            // Check for repomd.xml.asc (Red Hat style)
            let repomd_asc = metadata_path.parent().unwrap().join("repomd.xml.asc");
            if repomd_asc.exists() {
                let sig_info = self.verify_with_gpg(metadata_path, Some(&repomd_asc))?;
                return Ok(sig_info.valid && self.is_key_trusted(&sig_info.key_id));
            }
        }
        
        // Default behavior based on global policy
        Ok(!self.signing_policy.require_signature)
    }

    fn is_key_trusted(&self, key_id: &str) -> bool {
        self.trusted_keys.contains_key(key_id)
    }

    pub fn get_signing_status(&self, package_path: &Path) -> Result<String> {
        let sig_info = self.verify_package_signature(package_path, None)?;
        
        let status = if sig_info.valid {
            if self.is_key_trusted(&sig_info.key_id) {
                "✓ Valid signature from trusted key"
            } else {
                "⚠ Valid signature from untrusted key"
            }
        } else {
            "✗ Invalid or missing signature"
        };
        
        Ok(format!("{} (Key: {}, Trust: {:?})", 
                  status, sig_info.key_id, sig_info.trust_level))
    }

    pub fn batch_verify_packages(&self, package_paths: &[PathBuf]) -> Result<HashMap<PathBuf, SignatureInfo>> {
        let mut results = HashMap::new();
        
        println!("Batch verifying {} packages...", package_paths.len());
        
        for (i, path) in package_paths.iter().enumerate() {
            println!("Verifying package {}/{}: {}", i + 1, package_paths.len(), path.display());
            
            match self.verify_package_signature(path, None) {
                Ok(sig_info) => {
                    results.insert(path.clone(), sig_info);
                }
                Err(e) => {
                    println!("Failed to verify {}: {}", path.display(), e);
                    results.insert(path.clone(), SignatureInfo {
                        signature_type: SignatureType::GPG,
                        key_id: "error".to_string(),
                        fingerprint: "error".to_string(),
                        timestamp: chrono::Utc::now(),
                        valid: false,
                        trust_level: TrustLevel::Unknown,
                    });
                }
            }
        }
        
        Ok(results)
    }

    pub fn export_trusted_keys(&self, export_path: &Path) -> Result<()> {
        let export_data = serde_json::json!({
            "exported_at": chrono::Utc::now(),
            "trusted_keys": self.trusted_keys,
            "signing_policy": self.signing_policy
        });
        
        fs::write(export_path, serde_json::to_string_pretty(&export_data)?)?;
        println!("Exported trusted keys to: {}", export_path.display());
        Ok(())
    }

    pub fn import_trusted_keys(&mut self, import_path: &Path) -> Result<()> {
        let content = fs::read_to_string(import_path)?;
        let import_data: serde_json::Value = serde_json::from_str(&content)?;
        
        if let Some(keys) = import_data.get("trusted_keys") {
            let imported_keys: HashMap<String, TrustedKey> = serde_json::from_value(keys.clone())?;
            
            for (key_id, key) in imported_keys {
                self.trusted_keys.insert(key_id, key);
            }
            
            self.save_config()?;
            println!("Imported trusted keys from: {}", import_path.display());
        }
        
        Ok(())
    }
}

// Command-line interface functions
pub fn handle_signing_verification_command(args: &[String]) -> Result<()> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("linux-distro-agent");
    
    let mut manager = SigningVerificationManager::new(&config_dir)?;
    
    match args.get(0).map(|s| s.as_str()) {
        Some("verify") => {
            if let Some(package_path) = args.get(1) {
                let path = PathBuf::from(package_path);
                let status = manager.get_signing_status(&path)?;
                println!("{}", status);
            } else {
                println!("Usage: lda sign verify <package_path>");
            }
        }
        Some("add-key") => {
            if let (Some(key_file), Some(owner), Some(email)) = (args.get(1), args.get(2), args.get(3)) {
                manager.add_trusted_key(&PathBuf::from(key_file), owner, email)?;
            } else {
                println!("Usage: lda sign add-key <key_file> <owner> <email>");
            }
        }
        Some("list-keys") => {
            let keys = manager.list_trusted_keys();
            if keys.is_empty() {
                println!("No trusted keys configured");
            } else {
                println!("Trusted Keys:");
                for key in keys {
                    println!("  {} - {} <{}> (Trust: {:?})", 
                            key.key_id, key.owner, key.email, key.trust_level);
                }
            }
        }
        Some("remove-key") => {
            if let Some(key_id) = args.get(1) {
                manager.remove_trusted_key(key_id)?;
            } else {
                println!("Usage: lda sign remove-key <key_id>");
            }
        }
        Some("verify-repo") => {
            if let (Some(repo_name), Some(metadata_path)) = (args.get(1), args.get(2)) {
                let valid = manager.verify_repository_metadata(repo_name, &PathBuf::from(metadata_path))?;
                println!("Repository {} metadata verification: {}", 
                        repo_name, if valid { "✓ Valid" } else { "✗ Invalid" });
            } else {
                println!("Usage: lda sign verify-repo <repo_name> <metadata_path>");
            }
        }
        Some("export") => {
            if let Some(export_path) = args.get(1) {
                manager.export_trusted_keys(&PathBuf::from(export_path))?;
            } else {
                println!("Usage: lda sign export <export_path>");
            }
        }
        Some("import") => {
            if let Some(import_path) = args.get(1) {
                manager.import_trusted_keys(&PathBuf::from(import_path))?;
            } else {
                println!("Usage: lda sign import <import_path>");
            }
        }
        _ => {
            println!("Package Signing and Verification Commands:");
            println!("  lda sign verify <package_path>              - Verify package signature");
            println!("  lda sign add-key <key_file> <owner> <email> - Add trusted key");
            println!("  lda sign list-keys                          - List trusted keys");
            println!("  lda sign remove-key <key_id>                - Remove trusted key");
            println!("  lda sign verify-repo <repo> <metadata>      - Verify repository metadata");
            println!("  lda sign export <path>                      - Export trusted keys");
            println!("  lda sign import <path>                      - Import trusted keys");
        }
    }
    
    Ok(())
}
