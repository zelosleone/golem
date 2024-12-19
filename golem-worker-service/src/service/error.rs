use std::fmt;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum ServiceError {
    ValidationError(String),
    ConversionError(String),
    NotFound(String),
    InternalError(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ServiceError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            ServiceError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ServiceError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ServiceError {}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ServiceError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ServiceError::ConversionError(msg) => (StatusCode::BAD_REQUEST, msg),
            ServiceError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ServiceError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, message).into_response()
    }
}
