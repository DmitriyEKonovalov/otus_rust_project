pub mod redis;
pub mod calc_info;

pub use calc_info::CalcInfo;
pub use redis::{AppState, RedisDataError, RedisResult, RedisDataError};
