use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                // For development: show detailed error
                let detail = format!("Database error: {}", e);
                eprintln!("🔴 DATABASE ERROR: {}", detail);
                (StatusCode::INTERNAL_SERVER_ERROR, detail)
            }
            AppError::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.to_string()),
            AppError::Authentication(ref msg) => (StatusCode::UNAUTHORIZED, msg.to_string()),
            AppError::Authorization(ref msg) => (StatusCode::FORBIDDEN, msg.to_string()),
            AppError::Forbidden(ref msg) => (StatusCode::FORBIDDEN, msg.to_string()),
            AppError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.to_string()),
            AppError::Conflict(ref msg) => (StatusCode::CONFLICT, msg.to_string()),
            AppError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string())
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
