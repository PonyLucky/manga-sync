use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::{SqlitePool, Row};
use std::collections::HashMap;
use crate::utils::response::{ApiResponse, ApiError};

pub async fn list_settings(
    State(pool): State<SqlitePool>,
) -> Result<Json<ApiResponse<HashMap<String, String>>>, ApiError> {
    let settings = sqlx::query("SELECT key, value FROM setting")
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let map = settings.into_iter().map(|s| (s.get("key"), s.get("value"))).collect();
    Ok(Json(ApiResponse::success(map)))
}

pub async fn update_setting(
    State(pool): State<SqlitePool>,
    Path(key): Path<String>,
    body: String,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    sqlx::query(
        "INSERT INTO setting (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value"
    )
    .bind(&key)
    .bind(&body)
    .execute(&pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success_null()))
}
