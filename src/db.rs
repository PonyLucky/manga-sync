use sqlx::SqlitePool;
use std::path::Path;

const DB_PATH: &str = "secret/manga_manager.db";

pub async fn init_db() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let db_path = Path::new(DB_PATH);

    // Création du répertoire secret s'il n'existe pas
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Création de la pool de connexions
    let pool = SqlitePool::connect(&format!("sqlite:{}", DB_PATH)).await?;

    // Application des migrations
    apply_migrations(&pool).await?;

    Ok(pool)
}

async fn apply_migrations(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    // Vérification si la table des migrations existe
    let migrations_table_exists = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='migrations'"
    )
    .fetch_optional(pool)
    .await?
    .is_some();

    if !migrations_table_exists {
        // Création de la table des migrations
        sqlx::query(
            "CREATE TABLE migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )"
        )
        .execute(pool)
        .await?;
    }

    // Liste des migrations disponibles
    let migrations = vec![
        "001_initial.sql",
        // Ajoutez d'autres fichiers de migration ici
    ];

    for migration in migrations {
        let migration_name = migration.replace(".sql", "");

        // Vérification si la migration a déjà été appliquée
        let already_applied = sqlx::query_scalar::<_, i64>(
            "SELECT 1 FROM migrations WHERE name = ?"
        )
        .bind(&migration_name)
        .fetch_optional(pool)
        .await?
        .is_some();

        if !already_applied {
            // Lecture du fichier de migration
            let migration_sql = std::fs::read_to_string(migration)?;

            // Exécution de la migration
            sqlx::query(&migration_sql)
                .execute(pool)
                .await?;

            // Enregistrement de la migration appliquée
            sqlx::query(
                "INSERT INTO migrations (name) VALUES (?)"
            )
            .bind(&migration_name)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}
