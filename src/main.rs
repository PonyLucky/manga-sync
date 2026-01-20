use axum::{
    routing::{get, post, patch, delete},
    Router,
    middleware,
};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::trace::TraceLayer;
use std::sync::Arc;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use manga_sync::auth::key_manager::KeyManager;
use manga_sync::auth::middleware::auth_middleware;
use manga_sync::cache::ChapterCache;
use manga_sync::state::AppState;
use manga_sync::{db, handlers, sync, settings};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let secret_dir = "secret";
    std::fs::create_dir_all(secret_dir)?;

    // Initialize the database first so we can read settings
    let pool = db::init_db(&format!("sqlite:{}/manga.db", secret_dir)).await?;

    // Load settings from the database
    let ttl_warning = settings::get_setting_u64(&pool, "TTL_KEY_WARNING", 90).await?;
    let ttl_limit = settings::get_setting_u64(&pool, "TTL_KEY_LIMIT", 365).await?;
    let cron_sync = settings::get_setting_string(&pool, "CRON_SYNC", "0 0 0 * * *").await?;

    let key_manager = Arc::new(KeyManager::new(
        &format!("{}/key.pub", secret_dir),
        ttl_warning,
        ttl_limit,
    )?);
    let cache = Arc::new(ChapterCache::new());

    let mut scheduler = sync::scheduler::start_scheduler(pool.clone(), cache.clone(), &cron_sync).await?;

    let state = AppState {
        pool: pool.clone(),
        cache,
        key_manager: key_manager.clone(),
    };

    let app = Router::new()
        .route("/manga", get(handlers::manga::list_manga).post(handlers::manga::create_manga))
        .route("/manga/{id}", get(handlers::manga::get_manga).patch(handlers::manga::update_manga).delete(handlers::manga::delete_manga))
        .route("/manga/{id}/source", get(handlers::manga::get_manga_sources).post(handlers::manga::create_manga_source))
        .route("/manga/{id}/source/{domain}", delete(handlers::manga::delete_manga_source))
        .route("/manga/{id}/history", get(handlers::manga::get_manga_history))
        .route("/manga/refresh-unread", post(handlers::manga::refresh_all_unread))
        .route("/website", get(handlers::website::list_websites))
        .route("/website/{domain}", get(handlers::website::check_website).post(handlers::website::create_website).delete(handlers::website::delete_website))
        .route("/source", get(handlers::source::list_sources))
        .route("/setting", get(handlers::setting::list_settings))
        .route("/setting/{key}", patch(handlers::setting::update_setting))
        .route("/key", get(handlers::key::get_key_age).post(handlers::key::refresh_key))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
                .on_response(tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO))
        )
        .layer(middleware::from_fn_with_state(key_manager.clone(), auth_middleware))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 7783));
    let cert_path = format!("{}/ssl/cert.pem", secret_dir);
    let key_path = format!("{}/ssl/key.pem", secret_dir);

    if std::path::Path::new(&cert_path).exists() && std::path::Path::new(&key_path).exists() {
        let tls_config = RustlsConfig::from_pem_file(&cert_path, &key_path).await?;
        tracing::info!("Listening on https://{}", addr);

        let handle = axum_server::Handle::new();
        let shutdown_handle = handle.clone();

        tokio::spawn(async move {
            shutdown_signal().await;
            shutdown_handle.graceful_shutdown(Some(std::time::Duration::from_secs(10)));
        });

        axum_server::bind_rustls(addr, tls_config)
            .handle(handle)
            .serve(app.into_make_service())
            .await?;
    } else {
        tracing::info!("SSL certificates not found, falling back to HTTP");
        tracing::info!("Listening on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
    }

    tracing::info!("Shutting down scheduler...");
    scheduler.shutdown().await?;

    tracing::info!("Closing database connections...");
    pool.close().await;

    tracing::info!("Shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received SIGINT, initiating graceful shutdown...");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM, initiating graceful shutdown...");
        }
    }
}
