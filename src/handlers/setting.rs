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
    let settings = sqlx::query("SELECT key, value FROM setting ORDER BY key")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let map = settings.into_iter().map(|s| (s.get("key"), s.get("value"))).collect();
    Ok(Json(ApiResponse::success(map)))
}

#[utoipa::path(
    patch,
    path = "/setting/{key}",
    request_body = Object,
    responses(
        (status = 200, description = "Setting updated successfully", body = Object),
        (status = 404, description = "Setting not found", body = Object)
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
    let result = sqlx::query("UPDATE setting SET value = ? WHERE key = ?")
        .bind(&body)
        .bind(&key)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound(format!("Setting '{}' not found", key)));
    }

    Ok(Json(ApiResponse::success_null()))
}
