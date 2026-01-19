use anyhow::Result;
use sqlx::{Row, SqlitePool};

/// Get a setting value as u64, returning the default if not found or invalid
pub async fn get_setting_u64(pool: &SqlitePool, key: &str, default: u64) -> Result<u64> {
    let result = sqlx::query("SELECT value FROM setting WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;

    match result {
        Some(row) => {
            let value: String = row.get("value");
            Ok(value.parse().unwrap_or(default))
        }
        None => Ok(default),
    }
}

/// Get a setting value as String, returning the default if not found
pub async fn get_setting_string(pool: &SqlitePool, key: &str, default: &str) -> Result<String> {
    let result = sqlx::query("SELECT value FROM setting WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;

    match result {
        Some(row) => {
            let value: String = row.get("value");
            Ok(value)
        }
        None => Ok(default.to_string()),
    }
}
