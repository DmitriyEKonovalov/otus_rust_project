use std::sync::Arc;
use thiserror::Error;


#[derive(Debug, Error)]
pub enum RedisDataError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Calc not found")]
    NotFound,
}

pub type RedisResult<T> = Result<T, RedisDataError>;

#[derive(Clone)]
pub struct AppState {
    pub redis_client: Arc<redis::Client>,
}

