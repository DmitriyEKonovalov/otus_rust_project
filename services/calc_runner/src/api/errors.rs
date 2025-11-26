use axum::{http::StatusCode, response::IntoResponse, Json};
use common::redis::RedisDataError;
use serde::Serialize;

// структура для сообщения об ошибке
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// перечисление специфичных ошибок API
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    Redis(RedisDataError),
    #[error("Redis error: {0}")]
    RedisClient(#[from] redis::RedisError),
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
        let status = match self {
            ApiError::BadParams(_) => StatusCode::BAD_REQUEST,
            ApiError::Json(_) => StatusCode::BAD_REQUEST,
            ApiError::Redis(_) | ApiError::RedisClient(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::CalculationNotCompleted(_) => StatusCode::BAD_REQUEST,
        };
        let body = Json(ErrorResponse {
            error: self.to_string(),
        });
        (status, body).into_response()
    }
}

impl From<RedisDataError> for ApiError {
    fn from(value: RedisDataError) -> Self {
        match value {
            RedisDataError::NotFound => ApiError::NotFound,
            other => ApiError::Redis(other),
        }
    }
}
