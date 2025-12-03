use deadpool_redis::{Pool, Connection};
use deadpool_redis::redis::AsyncCommands;
use uuid::Uuid;
use std::sync::Arc;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashSet;
use serde_json;

use crate::models::{CalcInfo, UserCalcs};
use crate::models::{CALC_INFO_TTL_SECONDS};
use crate::models::{USER_CALC_TTL_SECONDS};

#[derive(Debug)]
pub enum StorageErrors { 
    Pool(String), 
    Client(String), 
    Json(String), 
    NotFound(String),
}

#[derive(Debug)]
pub struct Storage { pool: Pool }

pub type SharedStorage = Arc<Storage>;

impl Storage {
    pub fn new(pool: Pool) -> Self { Self { pool } }

    pub async fn get_conn(&self) -> Result<Connection, StorageErrors> {
        self.pool.get().await.map_err(|e| StorageErrors::Pool(format!("pool error: {e}")))
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T, StorageErrors> {
        let mut conn = self.get_conn().await?;
        let json: Option<String> = conn.get(key).await.map_err(|e| StorageErrors::Client(format!("get {key}: {e}")))?;
        let json = json.ok_or_else(|| StorageErrors::NotFound(format!("Key {key} not found")))?;
        serde_json::from_str(&json).map_err(|e| StorageErrors::Json(format!("deserialize {key}: {e}")))
    }

    pub async fn set<T: serde::Serialize>(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<(), StorageErrors> {
        let mut conn = self.get_conn().await?;
        let json = serde_json::to_string(value).map_err(|e| StorageErrors::Json(format!("serialize {key}: {e}")))?;
        let _: () = conn.set_ex(key, json, ttl_seconds).await.map_err(|e| StorageErrors::Client(format!("set {key}: {e}")))?;
        Ok(())
    }

    pub async fn get_all<T: serde::de::DeserializeOwned>(&self, prefix: &str) -> Result<Option<Vec<T>>, StorageErrors> {
        let mut conn = self.get_conn().await?;
        let pattern = format!("{}*", prefix);
        let keys: Vec<String> = conn.keys(pattern).await.map_err(|e| StorageErrors::Client(format!("keys with prefix {prefix}: {e}")))?;
        if keys.is_empty() {
            return Ok(None);
        }
        let mut values: Vec<T> = Vec::with_capacity(keys.len());
        for key in keys {
            let json: Option<String> = conn.get(&key).await.map_err(|e| StorageErrors::Client(format!("get {key}: {e}")))?;
            if let Some(json) = json {
                let value: T = serde_json::from_str(&json).map_err(|e| StorageErrors::Json(format!("deserialize {key}: {e}")))?;
                values.push(value);
            }
        }
        Ok(Some(values))
    }

    // создает начальные записи для нового расчета
    pub async fn init_calc(&self, user_id: i64, params: serde_json::Value) -> Result<CalcInfo, StorageErrors> {
        // создаем новый расчет
        let calc_id = Uuid::new_v4();
        let calc_info = CalcInfo {
            calc_id: calc_id,
            user_id: user_id,
            run_dt: Utc::now(),
            end_dt: None,
            params: Some(serde_json::to_value(params).unwrap()),
            progress: 0,
            result: None,
        };

        // сохраняем начальную запись о расчете
        self.set(&calc_info.key(), &calc_info, CALC_INFO_TTL_SECONDS).await?;

        // добавляем расчет к пользователю (если пользователя нет - создаем)
        let new_user_calc = UserCalcs {
            user_id: user_id,
            calcs: HashSet::from([calc_id]),
        };
        let mut user_calcs: UserCalcs = match self.get(&new_user_calc.key()).await {
            Ok(uc) => uc,
            Err(StorageErrors::NotFound(_)) => new_user_calc,
            Err(e) => return Err(e),
        };
        user_calcs.calcs.insert(calc_id);
        self.set(&user_calcs.key(), &user_calcs, USER_CALC_TTL_SECONDS).await?;

        Ok(calc_info)
    }

    // обновляет прогресс расчета
    pub async fn update_progress(&self, calc_info: &CalcInfo, progress: u32) -> Result<(), StorageErrors> {
        let mut new_calc_info = calc_info.clone();
        new_calc_info.progress = progress;
        self.set(&calc_info.key(), &new_calc_info, CALC_INFO_TTL_SECONDS).await
    }

    // сохранение результата расчета
    pub async fn set_result(&self, calc_info: &CalcInfo, result: Value) -> Result<(), StorageErrors> {
        // обновляем запись расчета с результатом
        let mut new_calc_info = calc_info.clone();
        new_calc_info.progress = 100;
        new_calc_info.end_dt = Some(Utc::now());
        new_calc_info.result = Some(result);
        self.set(&calc_info.key(), &new_calc_info, CALC_INFO_TTL_SECONDS).await?;

        // удаляем расчет из активных у пользователя
        let mut user_calcs: UserCalcs = self.get(&UserCalcs::to_key(&calc_info.user_id)).await?;
        user_calcs.calcs.remove(&calc_info.calc_id);
        self.set(&user_calcs.key(), &user_calcs, USER_CALC_TTL_SECONDS).await?;

        Ok(())
    }

}
