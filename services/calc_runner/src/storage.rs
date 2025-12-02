use deadpool_redis::{Pool, Connection};
use deadpool_redis::redis::AsyncCommands;
use std::sync::Arc;
use chrono::Utc;
use serde_json::Value;

use crate::models::{CalcInfo, UserCalc};
use crate::models::{CALC_INFO_PREFIX, CALC_INFO_TTL_SECONDS};
use crate::models::{USER_CALC_PREFIX, USER_CALC_TTL_SECONDS};

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

    pub async fn del(&self, key: &str) -> Result<(), StorageErrors> {
        let mut conn = self.get_conn().await?;
        let _: () = conn.del(key).await.map_err(|e| StorageErrors::Client(format!("del {key}: {e}")))?;
        Ok(())
    }

    pub async fn keys(&self, pattern: &str) -> Result<Vec<String>, StorageErrors> {
        let mut conn = self.get_conn().await?;
        let pattern = if pattern.ends_with('*') { pattern.to_owned() } else { format!("{pattern}*") };
        let keys: Vec<String> = conn.keys(&pattern).await.map_err(|e| StorageErrors::Client(format!("keys {pattern}: {e}")))?;
        Ok(keys)
    }

    pub async fn start_calc(&self, calc_info: &CalcInfo) -> Result<(), StorageErrors> {
        let calc_key = format!("{}{}", CALC_INFO_PREFIX, calc_info.calc_id);
        let user_calc_key = format!("{}{}", USER_CALC_PREFIX, calc_info.user_id);
        let user_calc = UserCalc { user_id: calc_info.user_id, calc_id: calc_info.calc_id };

        self.set(&calc_key, calc_info, CALC_INFO_TTL_SECONDS).await?;
        self.set(&user_calc_key, &user_calc, USER_CALC_TTL_SECONDS).await?;
        Ok(())
    }

    pub async fn update_progress(&self, calc_info: &CalcInfo, progress: u32) -> Result<(), StorageErrors> {
        let mut updated = calc_info.clone();
        updated.progress = progress;
        let calc_key = format!("{}{}", CALC_INFO_PREFIX, updated.calc_id);
        self.set(&calc_key, &updated, CALC_INFO_TTL_SECONDS).await
    }

    pub async fn set_result(&self, calc_info: &CalcInfo, result: Value) -> Result<(), StorageErrors> {
        let mut updated = calc_info.clone();
        updated.progress = 100;
        updated.end_dt = Some(Utc::now());
        updated.result = Some(result);

        let calc_key = format!("{}{}", CALC_INFO_PREFIX, updated.calc_id);
        self.set(&calc_key, &updated, CALC_INFO_TTL_SECONDS).await?;

        let user_calc_key = format!("{}{}", USER_CALC_PREFIX, updated.user_id);
        self.del(&user_calc_key).await?;
        Ok(())
    }

}
