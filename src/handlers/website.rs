use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::SqlitePool;
use serde::Serialize;
use crate::utils::response::{ApiResponse, ApiError};
use crate::models::Website;

pub async fn list_websites(
    State(pool): State<SqlitePool>,
) -> Result<Json<ApiResponse<Vec<Website>>>, ApiError> {
    let websites = sqlx::query_as::<sqlx::Sqlite, Website>("SELECT id, domain FROM website")
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(ApiResponse::success(websites)))
}

#[derive(Serialize)]
pub struct Existence {
    pub existing: bool,
}

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
