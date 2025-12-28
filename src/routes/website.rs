use axum::{
    Router,
    routing::{get, post},
    http::StatusCode,
    response::Json,
    extract::Path,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;
use crate::models::Website;
use crate::Response;

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_websites))
        .route("/:domain", get(check_website))
        .route("/:domain", post(create_website))
}

async fn list_websites(
    state: Arc<SqlitePool>,
) -> Result<Json<Response<Vec<Website>>>, StatusCode> {
    // Implémentation à compléter
    Ok(Json(Response {
        status: "success".to_string(),
        message: "Websites list".to_string(),
        data: None,
    }))
}

async fn check_website(
    Path(domain): Path<String>,
    state: Arc<SqlitePool>,
) -> Result<Json<Response<serde_json::Value>>, StatusCode> {
    // Implémentation à compléter
    Ok(Json(Response {
        status: "success".to_string(),
        message: "Website check".to_string(),
        data: None,
    }))
}

async fn create_website(
    Path(domain): Path<String>,
    state: Arc<SqlitePool>,
) -> Result<Json<Response<()>>, StatusCode> {
    // Implémentation à compléter
    Ok(Json(Response {
        status: "success".to_string(),
        message: "Website created".to_string(),
        data: None,
    }))
}
