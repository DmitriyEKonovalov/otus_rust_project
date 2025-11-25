use crate::calc_info::CalcInfo;
use chrono::Utc;
use redis::{Commands, Connection};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

const TTL_SECONDS: u64 = 20000;
const CALC_PREFIX: &str = "calc:";

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

//  сохранить параметры расчета в Redis
pub fn set_calc_info(conn: &mut Connection, calc_id: Uuid, info: &CalcInfo) -> RedisResult<()> {
    let key = format!("{}{}", CALC_PREFIX, calc_id);
    let json = serde_json::to_string(info)?;
    conn.set_ex::<_, _, ()>(&key, &json, TTL_SECONDS)?;
    Ok(())
}

//  получить информацию о расчете из Redis
pub fn get_calc_info(conn: &mut Connection, calc_id: Uuid) -> RedisResult<CalcInfo> {
    let key = format!("{}{}", CALC_PREFIX, calc_id);
    let json: Option<String> = conn.get(&key)?;
    let Some(s) = json else {
        return Err(RedisDataError::NotFound);
    };

    let calc_info = serde_json::from_str(&s)?;
    Ok(calc_info)
}


//  обновить прогресс расчета в Redis
pub fn update_progress(conn: &mut Connection, calc_id: Uuid, progress: u32) -> RedisResult<()> {
    let mut calc_info = get_calc_info(conn, calc_id)?;

    calc_info.progress = progress;
    set_calc_info(conn, calc_id, &calc_info)
}

//  сохранить результат расчета в Redis
pub fn set_result(
    conn: &mut Connection,
    calc_id: Uuid,
    result: serde_json::Value,
) -> RedisResult<()> {
    let mut calc_info = get_calc_info(conn, calc_id)?;

    calc_info.end_dt = Some(Utc::now());
    calc_info.result = Some(result);
    calc_info.progress = 100;
    set_calc_info(conn, calc_id, &calc_info)
}
