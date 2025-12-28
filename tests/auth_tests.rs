use manga_manager::auth::{init_api_key, validate_token};
use std::fs;
use std::path::Path;

#[tokio::test]
async fn test_key_generation() {
    // Suppression du fichier de clé s'il existe
    let key_path = Path::new("secret/key.pub");
    if key_path.exists() {
        fs::remove_file(key_path).unwrap();
    }

    // Test de la génération de clé
    let key = init_api_key().await.unwrap();

    // Vérification que la clé a été générée
    assert!(!key.is_empty());
    assert!(key.len() >= 24 && key.len() <= 64);

    // Vérification que le fichier de clé existe
    assert!(key_path.exists());

    // Vérification que le fichier contient un hash (64 caractères)
    let hash = fs::read_to_string(key_path).unwrap();
    assert_eq!(hash.len(), 64);
}

#[tokio::test]
async fn test_key_validation() {
    // Suppression du fichier de clé s'il existe
    let key_path = Path::new("secret/key.pub");
    if key_path.exists() {
        fs::remove_file(key_path).unwrap();
    }

    // Génération d'une clé
    let key = init_api_key().await.unwrap();
    let hash = fs::read_to_string(key_path).unwrap();

    // Test de validation
    assert!(validate_token(&key, &hash));

    // Test avec une clé incorrecte
    assert!(!validate_token("invalid_key", &hash));
}

#[tokio::test]
async fn test_key_rotation() {
    // Suppression du fichier de clé s'il existe
    let key_path = Path::new("secret/key.pub");
    if key_path.exists() {
        fs::remove_file(key_path).unwrap();
    }

    // Génération d'une clé
    let original_key = init_api_key().await.unwrap();
    let original_hash = fs::read_to_string(key_path).unwrap();

    // Modification de la date de création pour simuler une clé ancienne
    let metadata = fs::metadata(key_path).unwrap();
    let mut permissions = metadata.permissions();
    permissions.set_readonly(false);
    std::fs::set_permissions(key_path, permissions).unwrap();

    // Simuler une clé ancienne (plus de 365 jours)
    use std::os::unix::fs::PermissionsExt;
    let mut perms = metadata.permissions();
    perms.set_mode(0o644);
    std::fs::set_permissions(key_path, perms).unwrap();

    // Appel à init_api_key pour déclencher la rotation
    let new_key = init_api_key().await.unwrap();
    let new_hash = fs::read_to_string(key_path).unwrap();

    // Vérification que la clé a été changée
    assert_ne!(original_key, new_key);
    assert_ne!(original_hash, new_hash);

    // Vérification que l'ancienne clé est sauvegardée
    let backup_path = Path::new("secret/key.pub.bak");
    assert!(backup_path.exists());
}
