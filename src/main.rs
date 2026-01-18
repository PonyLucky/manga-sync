use axum::{
    routing::{get, post, delete},
    Router,
    middleware,
};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use manga_sync::auth::key_manager::KeyManager;
use manga_sync::auth::middleware::auth_middleware;
use manga_sync::cache::ChapterCache;
use manga_sync::state::AppState;
use manga_sync::{db, handlers, sync};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let secret_dir = "secret";
    std::fs::create_dir_all(secret_dir)?;

    let key_manager = Arc::new(KeyManager::new(&format!("{}/key.pub", secret_dir))?);
    let pool = db::init_db(&format!("sqlite:{}/manga.db", secret_dir)).await?;
    let cache = Arc::new(ChapterCache::new());

    let _scheduler = sync::scheduler::start_scheduler(pool.clone(), cache.clone()).await?;

    let state = AppState {
        pool,
        cache,
    };

    let app = Router::new()
        .route("/manga", get(handlers::manga::list_manga).post(handlers::manga::create_manga))
        .route("/manga/{id}", get(handlers::manga::get_manga).patch(handlers::manga::update_manga).delete(handlers::manga::delete_manga))
        .route("/manga/{id}/source", get(handlers::manga::get_manga_sources))
        .route("/manga/{id}/source/{domain}", delete(handlers::manga::delete_manga_source))
        .route("/manga/{id}/history", get(handlers::manga::get_manga_history))
        .route("/manga/refresh-unread", post(handlers::manga::refresh_all_unread))
        .route("/website", get(handlers::website::list_websites))
        .route("/website/{domain}", get(handlers::website::check_website).post(handlers::website::create_website).delete(handlers::website::delete_website))
        .route("/source", get(handlers::source::list_sources))
        .route("/setting", get(handlers::setting::list_settings))
        .route("/setting/{key}", post(handlers::setting::update_setting))
        .layer(middleware::from_fn_with_state(key_manager.clone(), auth_middleware))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7783").await?;
    tracing::info!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
