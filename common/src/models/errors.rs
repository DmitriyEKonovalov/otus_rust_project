use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Calc not found")]
    NotFound,
}
