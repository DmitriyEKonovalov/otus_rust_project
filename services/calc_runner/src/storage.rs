use deadpool_redis::{Pool, Connection};
use deadpool_redis::redis::AsyncCommands;
use std::sync::Arc;

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

    pub async fn mget(&self, pattern: &str) -> Result<Vec<String>, StorageErrors> {
        let mut conn = self.get_conn().await?;
        let keys: Vec<String> = conn.keys(pattern).await.map_err(|e| StorageErrors::Client(format!("keys {pattern}: {e}")))?;
        Ok(keys)
    }

}
