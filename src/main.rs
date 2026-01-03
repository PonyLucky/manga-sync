use axum::{
    routing::{get, post, delete},
    Router,
    middleware,
};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use manga_sync::auth::key_manager::KeyManager;
use manga_sync::auth::middleware::auth_middleware;
use manga_sync::{db, handlers};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let secret_dir = "secret";
    std::fs::create_dir_all(secret_dir)?;

    let key_manager = Arc::new(KeyManager::new(&format!("{}/key.pub", secret_dir))?);
    let pool = db::init_db(&format!("sqlite:{}/manga.db", secret_dir)).await?;

    let app = Router::new()
        .route("/manga", get(handlers::manga::list_manga).post(handlers::manga::create_manga))
        .route("/manga/:id", get(handlers::manga::get_manga).patch(handlers::manga::update_manga).delete(handlers::manga::delete_manga))
        .route("/manga/:id/source", get(handlers::manga::get_manga_sources))
        .route("/manga/:id/source/:domain", delete(handlers::manga::delete_manga_source))
        .route("/manga/:id/history", get(handlers::manga::get_manga_history))
        .route("/website", get(handlers::website::list_websites))
        .route("/website/:domain", get(handlers::website::check_website).post(handlers::website::create_website))
        .route("/setting", get(handlers::setting::list_settings))
        .route("/setting/:key", post(handlers::setting::update_setting))
        .layer(middleware::from_fn_with_state(key_manager.clone(), auth_middleware))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7783").await?;
    tracing::info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
