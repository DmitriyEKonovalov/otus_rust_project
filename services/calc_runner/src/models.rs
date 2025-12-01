use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

pub const CALC_INFO_PREFIX: &str = "calc_info:";
pub const CALC_INFO_TTL_SECONDS: u64 = 24 * 3600;

pub const USER_CALC_PREFIX: &str = "user_calc:";
pub const USER_CALC_TTL_SECONDS: u64 = 24 * 3600;


// Структура для информации о расчете
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcInfo {
    pub calc_id: Uuid,
    pub user_id:i64,
    pub run_dt: DateTime<Utc>,
    pub end_dt: Option<DateTime<Utc>>,
    pub params: Option<serde_json::Value>,
    pub progress: u32, // 0..100%
    pub result: Option<serde_json::Value>,
}

// Структура для расчетов пользователя
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCalc {
    pub user_id: i64,
    pub calc_id: Uuid,
}
