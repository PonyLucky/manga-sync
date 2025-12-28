use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, patch},
    http::StatusCode,
    response::Json,
    extract::Path,
    extract::Query,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use sha2::{Sha256, Digest};
use rand::{distributions::Alphanumeric, Rng};
use std::io::Write;

mod auth;
mod db;
mod models;
mod routes;

#[derive(Debug, Serialize, Deserialize)]
struct Response<T> {
    status: String,
    message: String,
    data: Option<T>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialisation de la base de données
    let db_pool = db::init_db().await?;

    // Gestion de la clé d'authentification
    let api_key = auth::init_api_key().await?;

    // Création du routeur
    let app = Router::new()
        .nest("/manga", routes::manga::router())
        .nest("/website", routes::website::router())
        .nest("/setting", routes::setting::router())
        .with_state(Arc::new(db_pool));

    // Démarrage du serveur
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 7783));
    println!("Server running on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
