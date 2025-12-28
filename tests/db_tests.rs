use manga_manager::db::init_db;
use sqlx::SqlitePool;

#[tokio::test]
async fn test_db_initialization() {
    // Suppression de la base de données s'elle existe
    let db_path = std::path::Path::new("secret/manga_manager.db");
    if db_path.exists() {
        std::fs::remove_file(db_path).unwrap();
    }

    // Test de l'initialisation de la base de données
    let pool = init_db().await.unwrap();

    // Vérification que la base de données a été créée
    assert!(db_path.exists());

    // Vérification que les tables existent
    let manga_table = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='manga'"
    )
    .fetch_optional(&pool)
    .await
    .unwrap()
    .is_some();

    let website_table = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='website'"
    )
    .fetch_optional(&pool)
    .await
    .unwrap()
    .is_some();

    let source_table = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='source'"
    )
    .fetch_optional(&pool)
    .await
    .unwrap()
    .is_some();

    let chapter_table = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='chapter'"
    )
    .fetch_optional(&pool)
    .await
    .unwrap()
    .is_some();

    let setting_table = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='setting'"
    )
    .fetch_optional(&pool)
    .await
    .unwrap()
    .is_some();

    assert!(manga_table);
    assert!(website_table);
    assert!(source_table);
    assert!(chapter_table);
    assert!(setting_table);
}

#[tokio::test]
async fn test_migrations() {
    // Suppression de la base de données s'elle existe
    let db_path = std::path::Path::new("secret/manga_manager.db");
    if db_path.exists() {
        std::fs::remove_file(db_path).unwrap();
    }

    // Initialisation de la base de données
    let pool = init_db().await.unwrap();

    // Vérification que la table des migrations existe
    let migrations_table = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='migrations'"
    )
    .fetch_optional(&pool)
    .await
    .unwrap()
    .is_some();

    assert!(migrations_table);

    // Vérification que la migration initiale a été appliquée
    let migration_applied = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM migrations WHERE name='001_initial'"
    )
    .fetch_optional(&pool)
    .await
    .unwrap()
    .is_some();

    assert!(migration_applied);
}
