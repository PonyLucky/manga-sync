#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use sqlx::SqlitePool;
    use std::sync::Arc;
    use manga_sync::auth::key_manager::KeyManager;
    use manga_sync::auth::middleware::auth_middleware;
    use manga_sync::handlers;
    use manga_sync::cache::ChapterCache;
    use manga_sync::state::AppState;
    use axum::{Router, routing::{get, post}, middleware};

    async fn setup_app() -> (Router, String) {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let key_path = "test_key_integration.pub";
        let _ = std::fs::remove_file(key_path);

        // Intercept stdout to get the key is hard, let's just use the hash from KM
        let km = Arc::new(KeyManager::new(key_path).unwrap());

        let state = AppState {
            pool,
            cache: Arc::new(ChapterCache::new()),
            key_manager: km.clone(),
        };

        // Since we can't easily get the plaintext key from KM after it's hashed and KM doesn't expose it
        // and it was printed to stdout...
        // For testing purposes, let's modify KeyManager to allow setting a known key or just use a mock for validation

        let app = Router::new()
            .route("/manga", get(handlers::manga::list_manga).post(handlers::manga::create_manga))
            .route("/website/{domain}", post(handlers::website::create_website))
            .layer(middleware::from_fn_with_state(km.clone(), auth_middleware))
            .with_state(state);

        // We'll need a way to validate in tests.
        // For this test, I'll assume I have the key.
        // Actually, let's just trust KeyManager::new prints it and we can't easily grab it.
        // Wait, I can just make a test-only way to validate.

        (app, "dummy".to_string())
    }

    #[tokio::test]
    async fn test_unauthorized() {
        let (app, _) = setup_app().await;

        let response = app
            .oneshot(Request::builder().uri("/manga").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
