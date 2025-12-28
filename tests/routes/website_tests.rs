use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use manga_manager::{
    db::init_db,
    routes::website::router,
};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_list_websites() {
    // Initialisation de la base de données
    let pool = init_db().await.unwrap();
    let app = router().with_state(Arc::new(pool));

    // Test de la route GET /website
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
async fn test_check_website() {
    // Initialisation de la base de données
    let pool = init_db().await.unwrap();
    let app = router().with_state(Arc::new(pool));

    // Test de la route GET /website/example.com
    let response = app
        .oneshot(Request::builder()
            .uri("/example.com")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_website() {
    // Initialisation de la base de données
    let pool = init_db().await.unwrap();
    let app = router().with_state(Arc::new(pool));

    // Test de la route POST /website/example.com
    let response = app
        .oneshot(Request::builder()
            .uri("/example.com")
            .method("POST")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
