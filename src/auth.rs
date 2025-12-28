use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};
use rand::{distributions::Alphanumeric, Rng};
use std::io::Write;

const KEY_FILE: &str = "secret/key.pub";

pub async fn init_api_key() -> Result<String, Box<dyn std::error::Error>> {
    let key_path = Path::new(KEY_FILE);

    if !key_path.exists() {
        // Génération d'une nouvelle clé
        let mut rng = rand::thread_rng();
        let length = rng.gen_range(24..=64);
        let key: String = rng.sample_iter(Alphanumeric)
            .collect();

        // Création du répertoire secret s'il n'existe pas
        if let Some(parent) = key_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Stockage de la clé (hashée)
        let hash = format!("{:x}", Sha256::digest(key.as_bytes()));
        fs::write(key_path, hash)?;

        // Affichage de la clé en clair
        println!("New API key generated: {}", key);
        Ok(key)
    } else {
        // Lecture de la clé existante
        let hash = fs::read_to_string(key_path)?;
        let creation_time = key_path.metadata()?.modified()?;
        let now = SystemTime::now();
        let age = now.duration_since(creation_time)?.as_secs() / 86400;

        if age > 365 {
            // Rotation automatique de la clé
            let mut rng = rand::thread_rng();
            let length = rng.gen_range(24..=64);
            let new_key: String = rng.sample_iter(Alphanumeric)
                .collect();
            let new_hash = format!("{:x}", Sha256::digest(new_key.as_bytes()));

            // Sauvegarde de l'ancienne clé avant modification
            let backup_path = Path::new("secret/key.pub.bak");
            fs::write(backup_path, &hash)?;

            // Écriture de la nouvelle clé
            fs::write(key_path, new_hash)?;

            println!("API key rotated (age: {} days)", age);
            println!("New API key: {}", new_key);
        } else if age > 90 {
            println!("Warning: API key is {} days old", age);
        }

        Ok(hash)
    }
}

pub fn validate_token(token: &str, stored_hash: &str) -> bool {
    let computed_hash = format!("{:x}", Sha256::digest(token));
    computed_hash == stored_hash
}
