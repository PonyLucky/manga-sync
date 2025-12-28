use serde::Serialize;
use axum::{
    response::{IntoResponse, Response},
    Json,
    http::StatusCode,
};

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub message: String,
    pub data: Option<T>,
}

impl ApiResponse<()> {
    pub fn success_null() -> Self {
        ApiResponse {
            status: "success".to_string(),
            message: "Operation successful".to_string(),
            data: None,
        }
    }
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            status: "success".to_string(),
            message: "Operation successful".to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: &str) -> Self {
        ApiResponse {
            status: "error".to_string(),
            message: message.to_string(),
            data: None,
        }
    }
}

pub enum ApiError {
    Unauthorized,
    Forbidden,
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            ApiError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            ApiError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone()),
        };

        let body = Json(ApiResponse::<()>::error(&message));
        (status, body).into_response()
    }
}
