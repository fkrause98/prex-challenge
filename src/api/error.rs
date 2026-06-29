use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde_json::json;
use thiserror::Error;

/// Type for API Errors, should serialize as
/// ```json
/// { "error": { "message": "<description>" } }
/// ```
#[derive(Debug, Error)]
#[error("{message}")]
pub struct ApiError {
    pub status_code: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn new(status_code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status_code,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code).json(json!({
            "error": {
                "message": self.message
            }
        }))
    }

    fn status_code(&self) -> StatusCode {
        self.status_code
    }
}

impl From<crate::store::StoreError> for actix_web::Error {
    fn from(e: crate::store::StoreError) -> Self {
        use crate::store::StoreError;
        match e {
            StoreError::AlreadyExists => actix_web::error::ErrorConflict(ApiError::conflict(e.to_string())),
            StoreError::Overflow => actix_web::error::ErrorBadRequest(ApiError::bad_request(e.to_string())),
            StoreError::LockError(msg) => actix_web::error::ErrorInternalServerError(ApiError::internal(msg)),
            StoreError::Io(io_e) => actix_web::error::ErrorInternalServerError(ApiError::internal(io_e.to_string())),
        }
    }
}
