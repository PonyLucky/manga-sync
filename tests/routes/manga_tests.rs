use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use manga_manager::{
    db::init_db,
    routes::manga::router,
};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_list_manga() {
    // Initialisation de la base de données
    let pool = init_db().await.unwrap();
    let app = router().with_state(Arc::new(pool));

    // Test de la route GET /manga
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
async fn test_get_manga() {
    // Initialisation de la base de données
    let pool = init_db().await.unwrap();
    let app = router().with_state(Arc::new(pool));

    // Test de la route GET /manga/1
    let response = app
        .oneshot(Request::builder()
            .uri("/1")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_sources() {
    // Initialisation de la base de données
    let pool = init_db().await.unwrap();
    let app = router().with_state(Arc::new(pool));

    // Test de la route GET /manga/1/source
    let response = app
        .oneshot(Request::builder()
            .uri("/1/source")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_history() {
    // Initialisation de la base de données
    let pool = init_db().await.unwrap();
    let app = router().with_state(Arc::new(pool));

    // Test de la route GET /manga/1/history
    let response = app
        .oneshot(Request::builder()
            .uri("/1/history")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_manga() {
    // Initialisation de la base de données
    let pool = init_db().await.unwrap();
    let app = router().with_state(Arc::new(pool));

    // Test de la route POST /manga
    let response = app
        .oneshot(Request::builder()
            .uri("/")
            .method("POST")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_update_manga() {
    // Initialisation de la base de données
    let pool = init_db().await.unwrap();
    let app = router().with_state(Arc::new(pool));

    // Test de la route PATCH /manga/1
    let response = app
        .oneshot(Request::builder()
            .uri("/1")
            .method("PATCH")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
