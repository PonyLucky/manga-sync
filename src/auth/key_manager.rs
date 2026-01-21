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
    hash: std::sync::RwLock<String>,
    ttl_warning_days: u64,
    ttl_limit_days: u64,
}

impl KeyManager {
    pub fn new(key_path: &str, ttl_warning_days: u64, ttl_limit_days: u64) -> Result<Self> {
        let path = Path::new(key_path);
        let km = KeyManager {
            key_path: key_path.to_string(),
            hash: std::sync::RwLock::new(String::new()),
            ttl_warning_days,
            ttl_limit_days,
        };

        info!("Key TTL settings: warning after {} days, auto-rotate after {} days", ttl_warning_days, ttl_limit_days);

        if !path.exists() {
            km.generate_new_key()?;
        } else {
            km.load_and_check_key()?;
        }

        Ok(km)
    }

    fn generate_new_key(&self) -> Result<String> {
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

        let hash = Self::hash_key(&key);

        // Remove read-only if exists, then write
        if Path::new(&self.key_path).exists() {
            let mut perms = fs::metadata(&self.key_path)?.permissions();
            perms.set_readonly(false);
            fs::set_permissions(&self.key_path, perms)?;
        }

        fs::write(&self.key_path, &hash)?;

        // Set read-only
        let mut perms = fs::metadata(&self.key_path)?.permissions();
        perms.set_readonly(true);
        fs::set_permissions(&self.key_path, perms)?;

        *self.hash.write().unwrap() = hash;
        info!("Key generated and stored.");
        Ok(key)
    }

    fn get_file_age(metadata: &fs::Metadata) -> Result<std::time::Duration> {
        let file_time = metadata.created().or_else(|_| {
            warn!("Creation time not available, falling back to modification time");
            metadata.modified()
        })?;
        Ok(SystemTime::now().duration_since(file_time)?)
    }

    fn load_and_check_key(&self) -> Result<()> {
        let metadata = fs::metadata(&self.key_path)?;
        let duration = Self::get_file_age(&metadata)?;
        let days = duration.as_secs() / 86400;

        info!("API key age: {} days", days);

        if days > self.ttl_limit_days {
            warn!("Key is older than {} days, auto-rotating...", self.ttl_limit_days);
            self.generate_new_key()?;
        } else {
            if days > self.ttl_warning_days {
                warn!("Key is older than {} days.", self.ttl_warning_days);
            }
            *self.hash.write().unwrap() = fs::read_to_string(&self.key_path)?.trim().to_string();
        }

        Ok(())
    }

    fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn validate_token(&self, token: &str) -> bool {
        Self::hash_key(token) == *self.hash.read().unwrap()
    }

    /// Returns the age of the key.pub file in days
    pub fn get_age_in_days(&self) -> Result<u64> {
        let metadata = fs::metadata(&self.key_path)?;
        let duration = Self::get_file_age(&metadata)?;
        Ok(duration.as_secs() / 86400)
    }

    /// Refreshes the key and returns the new plaintext key
    pub fn refresh_key(&self) -> Result<String> {
        info!("Refreshing API key...");
        self.generate_new_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn cleanup(path: &str) {
        if Path::new(path).exists() {
            let mut perms = fs::metadata(path).unwrap().permissions();
            perms.set_readonly(false);
            fs::set_permissions(path, perms).ok();
            fs::remove_file(path).ok();
        }
    }

    #[test]
    fn test_key_generation() {
        let temp_file = "test_key_gen.pub";
        cleanup(temp_file);

        {
            let km = KeyManager::new(temp_file, 90, 365).unwrap();
            assert!(Path::new(temp_file).exists());

            let metadata = fs::metadata(temp_file).unwrap();
            assert!(metadata.permissions().readonly());
            assert!(!km.hash.read().unwrap().is_empty());
        }

        cleanup(temp_file);
    }

    #[test]
    fn test_get_age_in_days() {
        let temp_file = "test_key_age.pub";
        cleanup(temp_file);

        {
            let km = KeyManager::new(temp_file, 90, 365).unwrap();
            let age = km.get_age_in_days().unwrap();
            // Newly created key should be 0 days old
            assert_eq!(age, 0);
        }

        cleanup(temp_file);
    }

    #[test]
    fn test_refresh_key() {
        let temp_file = "test_key_refresh.pub";
        cleanup(temp_file);

        {
            let km = KeyManager::new(temp_file, 90, 365).unwrap();
            let old_hash = km.hash.read().unwrap().clone();

            // Refresh the key
            let new_key = km.refresh_key().unwrap();

            // New key should not be empty
            assert!(!new_key.is_empty());
            // Key should be between 24 and 64 characters
            assert!(new_key.len() >= 24 && new_key.len() <= 64);

            // Hash should have changed
            let new_hash = km.hash.read().unwrap().clone();
            assert_ne!(old_hash, new_hash);

            // New key should validate
            assert!(km.validate_token(&new_key));
        }

        cleanup(temp_file);
    }

    #[test]
    fn test_refresh_key_invalidates_old_token() {
        let temp_file = "test_key_invalidate.pub";
        cleanup(temp_file);

        {
            let km = KeyManager::new(temp_file, 90, 365).unwrap();

            // Get a valid token by refreshing (since we don't have access to the original)
            let first_key = km.refresh_key().unwrap();
            assert!(km.validate_token(&first_key));

            // Refresh again
            let second_key = km.refresh_key().unwrap();

            // Old token should no longer be valid
            assert!(!km.validate_token(&first_key));
            // New token should be valid
            assert!(km.validate_token(&second_key));
        }

        cleanup(temp_file);
    }

    #[test]
    fn test_validate_token() {
        let temp_file = "test_key_validate.pub";
        cleanup(temp_file);

        {
            let km = KeyManager::new(temp_file, 90, 365).unwrap();

            // Invalid token should fail
            assert!(!km.validate_token("invalid_token"));
            assert!(!km.validate_token(""));

            // Get a valid key and test it
            let valid_key = km.refresh_key().unwrap();
            assert!(km.validate_token(&valid_key));
        }

        cleanup(temp_file);
    }

    #[test]
    fn test_key_contains_special_characters() {
        let temp_file = "test_key_special.pub";
        cleanup(temp_file);

        {
            let km = KeyManager::new(temp_file, 90, 365).unwrap();
            let key = km.refresh_key().unwrap();

            // Key should contain at least one special character
            let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
            let has_special = key.chars().any(|c| special_chars.contains(c));
            assert!(has_special, "Key should contain special characters");
        }

        cleanup(temp_file);
    }
}
