use crate::calc_info::CalcInfo;
use chrono::Utc;
use redis::{Commands, Connection, ErrorKind};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum RedisDataError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Task not found")]
    NotFound,
}

pub type RedisResult<T> = Result<T, RedisDataError>;

#[derive(Clone)]
pub struct AppState {
    pub redis_client: Arc<redis::Client>,
}

fn task_not_found() -> redis::RedisError {
    redis::RedisError::from((ErrorKind::ResponseError, "Task not found"))
}

pub fn get_calc_info(conn: &mut Connection, calc_id: Uuid) -> RedisResult<CalcInfo> {
    let key = format!("calc:{}", calc_id);
    let json: Option<String> = conn.get(&key)?;
    let Some(s) = json else {
        return Err(RedisDataError::NotFound);
    };

    let info = serde_json::from_str(&s)?;
    Ok(info)
}

pub fn set_calc_info(conn: &mut Connection, calc_id: Uuid, info: &CalcInfo) -> RedisResult<()> {
    let key = format!("calc:{}", calc_id);
    let json = serde_json::to_string(info)?;
    // TTL = 1 hour
    conn.set_ex::<_, _, ()>(&key, &json, 3600)?;
    Ok(())
}

pub fn update_progress(conn: &mut Connection, calc_id: Uuid, progress: u32) -> RedisResult<()> {
    let mut info = get_calc_info(conn, calc_id)?;

    info.progress = progress;
    set_calc_info(conn, calc_id, &info)
}

pub fn set_result(
    conn: &mut Connection,
    calc_id: Uuid,
    result: serde_json::Value,
) -> RedisResult<()> {
    let mut info = get_calc_info(conn, calc_id)?;

    info.end_dt = Some(Utc::now());
    info.result = Some(result);
    info.progress = 100;
    set_calc_info(conn, calc_id, &info)
}
