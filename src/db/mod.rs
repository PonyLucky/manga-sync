use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use anyhow::Result;
use std::fs;
use std::path::Path;

pub async fn init_db(db_url: &str) -> Result<SqlitePool> {
    let db_path = db_url.trim_start_matches("sqlite:");
    if let Some(parent) = Path::new(db_path).parent() {
        fs::create_dir_all(parent)?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}
