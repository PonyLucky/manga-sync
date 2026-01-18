use sha2::{Sha256, Digest};
use rand::distr::Alphanumeric;
use rand::Rng;
use std::fs;
use std::path::Path;
use std::time::SystemTime;
use anyhow::Result;
use tracing::{info, warn};

pub struct KeyManager {
    key_path: String,
    hash: String,
}

impl KeyManager {
    pub fn new(key_path: &str) -> Result<Self> {
        let path = Path::new(key_path);
        let mut km = KeyManager {
            key_path: key_path.to_string(),
            hash: String::new(),
        };

        if !path.exists() {
            km.generate_new_key()?;
        } else {
            km.load_and_check_key()?;
        }

        Ok(km)
    }

    fn generate_new_key(&mut self) -> Result<()> {
        if let Some(parent) = Path::new(&self.key_path).parent() {
            fs::create_dir_all(parent)?;
        }
        let rng = rand::rng();
        let length = rand::rng().random_range(24..=64);
        let key: String = rng
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect();
        
        // Ensure it includes special characters as per spec (Alphanumeric doesn't)
        // Let's add some special characters
        let mut key = key;
        let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
        let mut rng = rand::rng();
        for _ in 0..5 {
            let idx = rng.random_range(0..key.len());
            let spec_idx = rng.random_range(0..special_chars.len());
            key.replace_range(idx..idx+1, &special_chars[spec_idx..spec_idx+1]);
        }

        println!("Generated new API key: {}", key);
        
        let hash = self.hash_key(&key);
        fs::write(&self.key_path, &hash)?;
        
        // Set read-only
        let mut perms = fs::metadata(&self.key_path)?.permissions();
        perms.set_readonly(true);
        fs::set_permissions(&self.key_path, perms)?;

        self.hash = hash;
        info!("Key generated and stored.");
        Ok(())
    }

    fn load_and_check_key(&mut self) -> Result<()> {
        let metadata = fs::metadata(&self.key_path)?;
        let created = metadata.created()?;
        let duration = SystemTime::now().duration_since(created)?;
        let days = duration.as_secs() / 86400;

        info!("API key age: {} days", days);

        if days > 365 {
            warn!("Key is older than 365 days, auto-rotating...");
            self.generate_new_key()?;
        } else {
            if days > 90 {
                warn!("Key is older than 90 days.");
            }
            self.hash = fs::read_to_string(&self.key_path)?.trim().to_string();
        }

        Ok(())
    }

    fn hash_key(&self, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn validate_token(&self, token: &str) -> bool {
        self.hash_key(token) == self.hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_key_generation() {
        let temp_file = "test_key.pub";
        let _ = fs::remove_file(temp_file);
        
        {
            let km = KeyManager::new(temp_file).unwrap();
            assert!(std::path::Path::new(temp_file).exists());
            
            let metadata = fs::metadata(temp_file).unwrap();
            assert!(metadata.permissions().readonly());
            assert!(!km.hash.is_empty());
        }

        fs::remove_file(temp_file).unwrap();
    }
}
