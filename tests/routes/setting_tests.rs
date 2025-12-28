use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use manga_manager::{
    db::init_db,
    routes::setting::router,
};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_get_settings() {
    // Initialisation de la base de données
    let pool = init_db().await.unwrap();
    let app = router().with_state(Arc::new(pool));

    // Test de la route GET /setting
    let response = app
        .oneshot(Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_update_setting() {
    // Initialisation de la base de données
    let pool = init_db().await.unwrap();
    let app = router().with_state(Arc::new(pool));

    // Test de la route POST /setting/theme
    let response = app
        .oneshot(Request::builder()
            .uri("/theme")
            .method("POST")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
