use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;
use std::collections::HashSet;

pub const CALC_INFO_PREFIX: &str = "CALC_INFO";
pub const CALC_INFO_TTL_SECONDS: u64 = 1 * 3600;

pub const USER_CALC_PREFIX: &str = "USER_CALCS";
pub const USER_CALC_TTL_SECONDS: u64 = 1 * 3600;


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

impl CalcInfo {
    // возвращает ключ объекта в хранилище
    pub fn key(&self) -> String {
        format!("{}:{}", CALC_INFO_PREFIX, self.calc_id)
    }

    pub fn to_key(calc_id: &Uuid) -> String {
        format!("{}:{}", CALC_INFO_PREFIX, calc_id)
    }
}


// Структура для расчетов пользователя
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCalcs {
    pub user_id: i64,
    pub calcs: HashSet<Uuid>,
}

impl UserCalcs {
    // возвращает ключ объекта в хранилище
    pub fn key(&self) -> String {
        format!("{}:{}", USER_CALC_PREFIX, self.user_id)
    }
    
    pub fn to_key(user_id: &i64) -> String {
        format!("{}:{}", USER_CALC_PREFIX, user_id)
    }

}
