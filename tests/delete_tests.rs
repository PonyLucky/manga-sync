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
    use std::sync::Arc;
    use manga_sync::handlers;
    use manga_sync::cache::ChapterCache;
    use manga_sync::state::AppState;

    // Test without auth middleware to verify logic
    async fn setup_app_no_auth() -> (Router, SqlitePool) {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let state = AppState {
            pool: pool.clone(),
            cache: Arc::new(ChapterCache::new()),
        };

        let app = Router::new()
            .route("/manga", get(handlers::manga::list_manga).post(handlers::manga::create_manga))
            .route("/manga/{id}", get(handlers::manga::get_manga).patch(handlers::manga::update_manga).delete(handlers::manga::delete_manga))
            .route("/manga/{id}/source", get(handlers::manga::get_manga_sources))
            .route("/manga/{id}/source/{domain}", axum::routing::delete(handlers::manga::delete_manga_source))
            .route("/website/{domain}", get(handlers::website::check_website).post(handlers::website::create_website).delete(handlers::website::delete_website))
            .with_state(state);

        (app, pool)
    }

    #[tokio::test]
    async fn test_delete_manga() {
        let (app, pool) = setup_app_no_auth().await;

        // Create a manga
        sqlx::query("INSERT INTO manga (name, cover, cover_small) VALUES (?, ?, ?)")
            .bind("Test Manga")
            .bind("cover.jpg")
            .bind("cover_small.jpg")
            .execute(&pool)
            .await
            .unwrap();

        // Delete it
        let response = app.clone()
            .oneshot(
                Request::builder()
                    .uri("/manga/1")
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify it's gone
        let check = sqlx::query("SELECT id FROM manga WHERE id = 1")
            .fetch_optional(&pool)
            .await
            .unwrap();
        assert!(check.is_none());
    }

    #[tokio::test]
    async fn test_delete_manga_source() {
        let (app, pool) = setup_app_no_auth().await;

        // Create a manga
        sqlx::query("INSERT INTO manga (name, cover, cover_small) VALUES (?, ?, ?)")
            .bind("Test Manga")
            .bind("cover.jpg")
            .bind("cover_small.jpg")
            .execute(&pool)
            .await
            .unwrap();

        // Create a website
        sqlx::query("INSERT INTO website (domain) VALUES (?)")
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

        // Delete the source
        let response = app.clone()
            .oneshot(
                Request::builder()
                    .uri("/manga/1/source/example.com")
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify it's gone
        let check = sqlx::query("SELECT * FROM source WHERE manga_id = 1 AND website_id = 1")
            .fetch_optional(&pool)
            .await
            .unwrap();
        assert!(check.is_none());
    }

    #[tokio::test]
    async fn test_delete_manga_not_found() {
        let (app, _) = setup_app_no_auth().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/manga/999")
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_website() {
        let (app, pool) = setup_app_no_auth().await;

        // Create a website
        let result = sqlx::query("INSERT INTO website (domain) VALUES (?)")
            .bind("example.com")
            .execute(&pool)
            .await
            .unwrap();
        let website_id = result.last_insert_rowid();

        // Delete it
        let response = app.clone()
            .oneshot(
                Request::builder()
                    .uri("/website/example.com")
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify it's gone
        let check = sqlx::query("SELECT id FROM website WHERE id = ?")
            .bind(website_id)
            .fetch_optional(&pool)
            .await
            .unwrap();
        assert!(check.is_none());
    }

    #[tokio::test]
    async fn test_delete_website_not_found() {
        let (app, _pool) = setup_app_no_auth().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/website/nonexistent.com")
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_website_cascade() {
        let (app, pool) = setup_app_no_auth().await;

        // Create a manga
        sqlx::query("INSERT INTO manga (name, cover, cover_small) VALUES (?, ?, ?)")
            .bind("Test Manga")
            .bind("cover.jpg")
            .bind("cover_small.jpg")
            .execute(&pool)
            .await
            .unwrap();

        // Create a website
        let result = sqlx::query("INSERT INTO website (domain) VALUES (?)")
            .bind("example.com")
            .execute(&pool)
            .await
            .unwrap();
        let website_id = result.last_insert_rowid();

        // Create a source
        sqlx::query("INSERT INTO source (manga_id, website_id, path) VALUES (?, ?, ?)")
            .bind(1)
            .bind(website_id)
            .bind("/manga/test")
            .execute(&pool)
            .await
            .unwrap();

        // Delete the website
        let response = app.clone()
            .oneshot(
                Request::builder()
                    .uri("/website/example.com")
                    .method("DELETE")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify website is gone
        let check_website = sqlx::query("SELECT id FROM website WHERE id = ?")
            .bind(website_id)
            .fetch_optional(&pool)
            .await
            .unwrap();
        assert!(check_website.is_none());

        // Verify source is gone (cascade)
        let check_source = sqlx::query("SELECT * FROM source WHERE website_id = ?")
            .bind(website_id)
            .fetch_optional(&pool)
            .await
            .unwrap();
        assert!(check_source.is_none());
    }
}
