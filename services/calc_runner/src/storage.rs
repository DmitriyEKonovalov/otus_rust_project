use std::sync::Arc;
use redis::{AsyncCommands};

#[derive(Clone, Debug)]
pub enum StorageErrors{
    Client(String),
    Json(String),
    NotFound(String),
}


#[derive(Clone, Debug)]
pub struct Storage{
    pub client: Arc<redis::Client>,
}

impl Storage {
    // 
    pub async fn get_conn(&self) -> Result<redis::aio::MultiplexedConnection, StorageErrors> {
        let conn = self.client.get_multiplexed_async_connection().await.map_err(|e| {
            StorageErrors::Client(format!("Failed to get Redis connection: {}", e))
        })?;
        Ok(conn)
    }

    // get object from storage by key, serialize to Struct T, return instance of T,
    pub async fn get<T>(&self, key:  &str) -> Result<T, StorageErrors>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut conn = self.get_conn().await?;
        let json: String = conn.get(key).await.map_err(|e| {
            StorageErrors::Client(format!("Failed to get key {} from Redis: {}", key, e))
        })?;
        if json.is_empty() {
            return Err(StorageErrors::NotFound(format!("Key {} not found in storage", key)));
        };
        let value: T = serde_json::from_str(&json).map_err(|e| {
            StorageErrors::Json(format!("Failed to deserialize value for key {}: {}", key, e))
        })?;
        Ok(value)
    }

    pub async fn set<T>(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<(), StorageErrors>
    where
        T: serde::Serialize,
    {
        let mut conn = self.get_conn().await?;
        let json = serde_json::to_string(value).map_err(|e| {
            StorageErrors::Json(format!("Failed to serialize value for key {}: {}", key, e))
        })?;
        let _: () = conn.set_ex(key, json, ttl_seconds).await.map_err(|e| {
            StorageErrors::Client(format!("Failed to set key {} in Redis: {}", key, e))
        })?;
        Ok(())
    }
    
    pub async fn delete<T>(&self, key:  &str) -> Result<(), StorageErrors> {
        let mut conn = self.get_conn().await?;
        let _: () = conn.del(key).await.map_err(|e| {
            StorageErrors::Client(format!("Failed to delete key {} from Redis: {}", key, e))
        })?;
        Ok(())
    }

}