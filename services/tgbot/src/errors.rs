use common::DataError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BotError {
    #[error("Telegram error: {0}")]
    Telegram(#[from] teloxide::RequestError),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type HandlerResult = Result<(), BotError>;

impl From<DataError> for BotError {
    fn from(err: DataError) -> Self {
        match err {
            DataError::Redis(e) => BotError::Redis(e),
            DataError::Json(e) => BotError::Json(e),
            DataError::NotFound => BotError::Parse("Not found".into()),
        }
    }
}
