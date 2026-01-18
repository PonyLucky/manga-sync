use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::SqlitePool;
use serde::Serialize;
use crate::utils::response::{ApiResponse, ApiError};
use crate::models::Website;

use utoipa::ToSchema;

#[utoipa::path(
    get,
    path = "/website",
    responses(
        (status = 200, description = "List all websites", body = ApiResponse<Vec<Website>>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_websites(
    State(pool): State<SqlitePool>,
) -> Result<Json<ApiResponse<Vec<Website>>>, ApiError> {
    let websites = sqlx::query_as::<sqlx::Sqlite, Website>("SELECT id, domain FROM website")
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(websites)))
}

#[derive(Serialize, ToSchema)]
pub struct Existence {
    pub existing: bool,
}

#[utoipa::path(
    get,
    path = "/website/{domain}",
    responses(
        (status = 200, description = "Check if website exists", body = Object)
    ),
    params(
        ("domain" = String, Path, description = "Website domain")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn check_website(
    State(pool): State<SqlitePool>,
    Path(domain): Path<String>,
) -> Result<Json<ApiResponse<Existence>>, ApiError> {
    let website = sqlx::query("SELECT id FROM website WHERE domain = ?")
        .bind(&domain)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(Existence {
        existing: website.is_some(),
    })))
}

#[utoipa::path(
    post,
    path = "/website/{domain}",
    responses(
        (status = 200, description = "Website registered successfully", body = Object)
    ),
    params(
        ("domain" = String, Path, description = "Website domain")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_website(
    State(pool): State<SqlitePool>,
    Path(domain): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let existing = sqlx::query("SELECT id FROM website WHERE domain = ?")
        .bind(&domain)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if existing.is_some() {
        return Err(ApiError::BadRequest("Website already exists".into()));
    }

    sqlx::query("INSERT INTO website (domain) VALUES (?)")
        .bind(&domain)
        .execute(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success_null()))
}

#[utoipa::path(
    delete,
    path = "/website/{domain}",
    responses(
        (status = 200, description = "Website deleted successfully", body = Object),
        (status = 404, description = "Website not found")
    ),
    params(
        ("domain" = String, Path, description = "Website domain")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_website(
    State(pool): State<SqlitePool>,
    Path(domain): Path<String>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    let result = sqlx::query("DELETE FROM website WHERE domain = ?")
        .bind(domain)
        .execute(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Website not found".into()));
    }

    Ok(Json(ApiResponse::success_null()))
}
