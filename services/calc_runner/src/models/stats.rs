use errors::DataError;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::user_calcs::{USER_CALCS_PREFIX, UsersCalcs};
use crate::models::calc_info::{CALC_INFO_PREFIX, CalcInfo};
use serde_json;

// Структура для хранения статистики по расчетам всем пользователей 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersCalcStats {
    pub total_running: u32,
    pub calcs: HashMap<u64,Vec<UsersCalcs>>,
}

impl UsersCalcStats {
    // Сбор статистики из Redis по всем пользотвателям
    // получает все ключи по префиксу USER_CALCS_PREFIX
    // и добавляет в UsersCalcStats.calcs где ключ - user_id, значение - вектор UsersCalcs
    pub async fn get(conn: &mut impl AsyncCommands) -> Result<UsersCalcStats, DataError> {
        let mut stats = UsersCalcStats {
            total_running: 0,
            calcs: HashMap::new(),
        };

        let keys: Vec<String> = conn.keys(format!("{}*", USER_CALCS_PREFIX)).await?;
        for key in keys {
            let value: String = conn.get(&key).await.map_err(|_| DataError::NotFound)?;
            let user_calcs: UsersCalcs = serde_json::from_str(&value)?;
            let user_id = user_calcs.user_id as u64;
            stats.calcs.entry(user_id).or_insert_with(Vec::new).push(user_calcs);
            stats.total_running += 1;
        }

        Ok(stats)
    }
}


// Структура для хранения списка запущенных расчетов (без указангия пользователя)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningCalcs {
    pub total_running: u32,
    pub calcs: Vec<UsersCalcs>,
}

impl RunningCalcs {
    // получает список активных расчетов по всем ключам по префиксу СALC_INFO_PREFIX
    // и фильтрует по незавершенным расчетам (end_dt == None)
    // возвращает вектор CalcInfo
    pub async fn get(conn: &mut impl AsyncCommands) -> Result<Vec<CalcInfo>, DataError> {
        let mut running_calcs = Vec::new();
        let keys: Vec<String> = conn.keys(format!("{}*", CALC_INFO_PREFIX)).await?;
        for key in keys {
            let value: String = conn.get(&key).await.map_err(|_| DataError::NotFound)?;
            let calc_info: CalcInfo = serde_json::from_str(&value)?;
            if calc_info.end_dt.is_none() {
                running_calcs.push(calc_info);
            }
        }
        Ok(running_calcs)
    }
}

