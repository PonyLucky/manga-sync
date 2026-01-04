use axum::{
    extract::State,
    Json,
};
use sqlx::SqlitePool;
use crate::utils::response::{ApiResponse, ApiError};
use crate::models::Source;

#[utoipa::path(
    get,
    path = "/source",
    responses(
        (status = 200, description = "List all sources", body = ApiResponse<Vec<Source>>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_sources(
    State(pool): State<SqlitePool>,
) -> Result<Json<ApiResponse<Vec<Source>>>, ApiError> {
    let sources = sqlx::query_as::<sqlx::Sqlite, Source>("SELECT id, manga_id, website_id, path FROM source")
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(sources)))
}
