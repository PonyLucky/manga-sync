use axum::{
    extract::{State, Request},
    middleware::Next,
    response::Response,
    http::header::AUTHORIZATION,
};
use std::sync::Arc;
use crate::auth::key_manager::KeyManager;
use crate::utils::response::ApiError;

pub async fn auth_middleware(
    State(key_manager): State<Arc<KeyManager>>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let auth_header = req.headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(auth_str) if auth_str.starts_with("Bearer ") => {
            let token = &auth_str[7..];
            if key_manager.validate_token(token) {
                Ok(next.run(req).await)
            } else {
                Err(ApiError::Forbidden)
            }
        }
        Some(_) => Err(ApiError::Unauthorized),
        None => Err(ApiError::Unauthorized),
    }
}
