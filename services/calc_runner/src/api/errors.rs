use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use crate::storage::StorageErrors;
use tracing::error;

// структура для сообщения об ошибке
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// перечисление специфичных ошибок API
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Storage error: {0}")]
    StorageError(#[from] redis::RedisError),
    #[error("Bad params: {0}")]
    BadParams(String),
    #[error("Invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Not found")]
    NotFound,
    #[error("Calculation not completed: {0}")]
    CalculationNotCompleted(uuid::Uuid),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        error!(error = %self, "API error occurred");
        let status = match self {
            ApiError::BadParams(_) => StatusCode::BAD_REQUEST,
            ApiError::Json(_) => StatusCode::BAD_REQUEST,
            ApiError::StorageError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::CalculationNotCompleted(_) => StatusCode::BAD_REQUEST,
        };
        let body = Json(ErrorResponse {
            error: self.to_string(),
        });
        (status, body).into_response()
    }
}

impl From<StorageErrors> for ApiError {
    fn from(value: StorageErrors) -> Self {
        match value {
            StorageErrors::NotFound(_) => ApiError::NotFound,
            StorageErrors::Json(other) => ApiError::Json(serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::Other, other))),
            StorageErrors::Client(other) => ApiError::StorageError(redis::RedisError::from(std::io::Error::new(std::io::ErrorKind::Other, other))),
            StorageErrors::Pool(other) => ApiError::StorageError(redis::RedisError::from(std::io::Error::new(std::io::ErrorKind::Other, other))),
        }
    }
}
