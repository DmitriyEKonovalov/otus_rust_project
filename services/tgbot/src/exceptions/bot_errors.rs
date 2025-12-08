use thiserror::Error;

#[derive(Debug, Error)]
pub enum BotError {
    #[error("Telegram error: {0}")]
    Telegram(#[from] teloxide::RequestError),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type HandlerResult = Result<(), BotError>;
