use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use redis::AsyncCommands;
use serde_json;

use crate::models::errors;
use errors::DataError;

pub const CALC_INFO_PREFIX: &str = "calc_info:";
const CALC_INFO_TTL_SECONDS: u64 = 24 * 3600; // 24 часа

// Структура для работы с расчетами пользователя
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcInfo {
    pub calc_id: Uuid,
    pub run_dt: DateTime<Utc>,
    pub end_dt: Option<DateTime<Utc>>,
    pub params: Option<serde_json::Value>,
    pub progress: u32, // 0..100%
    pub result: Option<serde_json::Value>,
}

impl CalcInfo {
    pub fn new(calc_id: Uuid, params: Option<serde_json::Value>) -> Self {
        CalcInfo {
            calc_id,
            run_dt: Utc::now(),
            end_dt: None,
            params,
            progress: 0,
            result: None,
        }
    }

    //  сохранить параметры расчета в Redis
    pub async fn set(self, conn: &mut impl AsyncCommands) -> Result<(), DataError> {
        let key: String = format!("{}{}", CALC_INFO_PREFIX, self.calc_id);
        let json = serde_json::to_string(&self)?;
        let _: () = conn.set_ex(key, json, CALC_INFO_TTL_SECONDS).await?;
        Ok(())
    }

    //  получить информацию о расчете из Redis
    pub async fn get(conn: &mut impl AsyncCommands, calc_id: Uuid) -> Result<CalcInfo, DataError> {
        let key: String = format!("{}{}", CALC_INFO_PREFIX, calc_id);
        let value: String = conn.get(&key).await.map_err(|_| DataError::NotFound)?;
        let info: CalcInfo = serde_json::from_str(&value)?;
        Ok(info)
    }


    //  обновить прогресс расчета в Redis
    pub async fn update_progress(self, conn: &mut impl AsyncCommands, calc_id: Uuid, progress: u32) -> Result<(), DataError> {
        let mut calc_info = CalcInfo::get(conn, calc_id).await?;
        calc_info.progress = progress;
        CalcInfo::set(conn, calc_id, &calc_info).await
    }

    //  сохранить результат расчета в Redis
    pub async fn set_result(conn: &mut impl AsyncCommands, calc_id: Uuid,result: serde_json::Value,) -> Result<(), DataError> {
        let mut calc_info = CalcInfo::get(conn, calc_id).await?;

        calc_info.end_dt = Some(Utc::now());
        calc_info.result = Some(result);
        calc_info.progress = 100;
        CalcInfo::set(conn, calc_id, &calc_info).await
    }


}