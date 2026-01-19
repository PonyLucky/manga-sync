use axum::{
    extract::State,
    Json,
};
use serde::Serialize;
use tracing::info;
use crate::state::AppState;
use crate::utils::response::{ApiResponse, ApiError};

#[derive(Serialize)]
pub struct KeyAgeResponse {
    pub age_in_days: u64,
}

#[derive(Serialize)]
pub struct KeyRefreshResponse {
    pub key: String,
}

#[utoipa::path(
    get,
    path = "/key",
    responses(
        (status = 200, description = "Get key age in days", body = Object)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_key_age(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<KeyAgeResponse>>, ApiError> {
    let age_in_days = state
        .key_manager
        .get_age_in_days()
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(KeyAgeResponse { age_in_days })))
}

#[utoipa::path(
    post,
    path = "/key",
    responses(
        (status = 200, description = "Refresh key and return new key", body = Object)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn refresh_key(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<KeyRefreshResponse>>, ApiError> {
    let new_key = state
        .key_manager
        .refresh_key()
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    info!("New API key generated: {}", new_key);

    Ok(Json(ApiResponse::success(KeyRefreshResponse { key: new_key })))
}
