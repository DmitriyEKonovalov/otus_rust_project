use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

pub const CALC_INFO_PREFIX: &str = "calc_info:";
pub const CALC_INFO_TTL_SECONDS: u64 = 24 * 3600;

pub const USER_CALCS_PREFIX: &str = "user_calc:";
pub const USER_CALCS_TTL_SECONDS: u64 = 24 * 3600;


// Структура для хранения информации о расчете
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcInfo {
    pub calc_id: Uuid,
    pub run_dt: DateTime<Utc>,
    pub end_dt: Option<DateTime<Utc>>,
    pub params: Option<serde_json::Value>,
    pub progress: u32, // 0..100%
    pub result: Option<serde_json::Value>,
}

// Структура для хранения расчетов пользователя
#[derive(Debug, Clone)]
pub struct UsersCalcs {
    pub user_id: i64,
    pub calcs: HashSet<Uuid>,
}


// Структура для хранения статистики по расчетам всем пользователей 
#[derive(Debug, Clone)]
pub struct UsersCalcStats {
    pub total_running: u32,
    pub calcs: HashMap<u64,Vec<UsersCalcs>>,
}
