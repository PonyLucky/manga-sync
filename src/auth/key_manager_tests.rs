#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_key_generation() {
        let dir = tempdir().unwrap();
        let key_path = dir.path().join("key.pub");
        let key_path_str = key_path.to_str().unwrap();

        let km = KeyManager::new(key_path_str).unwrap();
        assert!(key_path.exists());
        
        let metadata = fs::metadata(key_path_str).unwrap();
        assert!(metadata.permissions().readonly());

        // Validate some dummy token - should fail
        assert!(!km.validate_token("wrong-token"));
    }
}
