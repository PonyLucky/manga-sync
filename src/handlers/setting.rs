use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::Row;
use std::collections::HashMap;
use crate::state::AppState;
use crate::utils::response::{ApiResponse, ApiError};

#[utoipa::path(
    get,
    path = "/setting",
    responses(
        (status = 200, description = "List all settings", body = Object)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_settings(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<HashMap<String, String>>>, ApiError> {
    let settings = sqlx::query("SELECT key, value FROM setting")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let map = settings.into_iter().map(|s| (s.get("key"), s.get("value"))).collect();
    Ok(Json(ApiResponse::success(map)))
}

#[utoipa::path(
    post,
    path = "/setting/{key}",
    request_body = Object,
    responses(
        (status = 200, description = "Setting updated successfully", body = Object)
    ),
    params(
        ("key" = String, Path, description = "Setting key")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_setting(
    State(state): State<AppState>,
    Path(key): Path<String>,
    body: String,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    sqlx::query(
        "INSERT INTO setting (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value"
    )
    .bind(&key)
    .bind(&body)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success_null()))
}
