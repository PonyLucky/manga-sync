#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
        routing::{get},
    };
    use tower::ServiceExt;
    use sqlx::SqlitePool;
    use manga_sync::handlers;
    
    async fn setup_app_no_auth() -> (Router, SqlitePool) {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let app = Router::new()
            .route("/source", get(handlers::source::list_sources))
            .with_state(pool.clone());

        (app, pool)
    }

    #[tokio::test]
    async fn test_list_sources() {
        let (app, pool) = setup_app_no_auth().await;

        // Create a manga
        sqlx::query("INSERT INTO manga (id, name, cover, cover_small) VALUES (?, ?, ?, ?)")
            .bind(1)
            .bind("Test Manga")
            .bind("cover.jpg")
            .bind("cover_small.jpg")
            .execute(&pool)
            .await
            .unwrap();

        // Create a website
        sqlx::query("INSERT INTO website (id, domain) VALUES (?, ?)")
            .bind(1)
            .bind("example.com")
            .execute(&pool)
            .await
            .unwrap();

        // Create a source
        sqlx::query("INSERT INTO source (manga_id, website_id, path) VALUES (?, ?, ?)")
            .bind(1)
            .bind(1)
            .bind("/manga/test")
            .execute(&pool)
            .await
            .unwrap();

        let response = app.clone()
            .oneshot(
                Request::builder()
                    .uri("/source")
                    .method("GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        assert!(body_str.contains("/manga/test"));
        assert!(body_str.contains("\"manga_id\":1"));
        assert!(body_str.contains("\"website_id\":1"));
    }
}
