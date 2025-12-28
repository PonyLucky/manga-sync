use axum::{
    Router,
    routing::{get, post, patch},
    http::StatusCode,
    response::Json,
    extract::{Path, Query},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use crate::models::{Manga, MangaDetail, MangaFilter, Pagination, Chapter};
use crate::Response;

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_manga))
        .route("/:id", get(get_manga))
        .route("/:id/source", get(get_sources))
        .route("/:id/history", get(get_history))
        .route("/", post(create_manga))
        .route("/:id", patch(update_manga))
}

async fn list_manga(
    Query(pagination): Query<Pagination>,
    Query(filters): Query<Vec<MangaFilter>>,
    state: Arc<SqlitePool>,
) -> Result<Json<Response<Vec<Manga>>>, StatusCode> {
    // Implémentation à compléter
    Ok(Json(Response {
        status: "success".to_string(),
        message: "Manga list".to_string(),
        data: None,
    }))
}

async fn get_manga(
    Path(id): Path<i64>,
    state: Arc<SqlitePool>,
) -> Result<Json<Response<MangaDetail>>, StatusCode> {
    // Implémentation à compléter
    Ok(Json(Response {
        status: "success".to_string(),
        message: "Manga details".to_string(),
        data: None,
    }))
}

async fn get_sources(
    Path(id): Path<i64>,
    state: Arc<SqlitePool>,
) -> Result<Json<Response<Vec<String>>>, StatusCode> {
    // Implémentation à compléter
    Ok(Json(Response {
        status: "success".to_string(),
        message: "Sources list".to_string(),
        data: None,
    }))
}

async fn get_history(
    Path(id): Path<i64>,
    state: Arc<SqlitePool>,
) -> Result<Json<Response<Vec<Chapter>>>, StatusCode> {
    // Implémentation à compléter
    Ok(Json(Response {
        status: "success".to_string(),
        message: "Reading history".to_string(),
        data: None,
    }))
}

async fn create_manga(
    state: Arc<SqlitePool>,
) -> Result<Json<Response<()>>, StatusCode> {
    // Implémentation à compléter
    Ok(Json(Response {
        status: "success".to_string(),
        message: "Manga created".to_string(),
        data: None,
    }))
}

async fn update_manga(
    Path(id): Path<i64>,
    state: Arc<SqlitePool>,
) -> Result<Json<Response<()>>, StatusCode> {
    // Implémentation à compléter
    Ok(Json(Response {
        status: "success".to_string(),
        message: "Manga updated".to_string(),
        data: None,
    }))
}
