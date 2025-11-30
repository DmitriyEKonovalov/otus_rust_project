use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashSet;
use uuid::Uuid;

use crate::models::errors;
use errors::DataError;

pub const USER_CALCS_PREFIX: &str = "user_calc:";
const USER_CALCS_TTL_SECONDS: u64 = 24 * 3600;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersCalcs {
    pub user_id: i64,
    pub calcs: HashSet<Uuid>,
}

impl UsersCalcs {
    pub fn new(user_id: i64) -> Self {
        UsersCalcs {
            user_id,
            calcs: HashSet::new(),
        }
    }
    // сохранить обьект в redis (key - user_id, value - сериализованный UsersCalcs)
    async fn set(self, conn: &mut impl AsyncCommands) -> Result<UsersCalcs, DataError> {
        let key = format!("{}{}", USER_CALCS_PREFIX, self.user_id);
        let value = serde_json::to_string(&self).map_err(|e| {
            DataError::SerializationError(format!("Failed to serialize UsersCalcs: {}", e))
        })?;
        let _: () = conn
            .set_ex(key, value, USER_CALCS_TTL_SECONDS as usize)
            .await
            .map_err(|e| {
                DataError::RedisError(format!("Failed to set UsersCalcs in Redis: {}", e))
            })?;
        Ok(self)
    }

    // получить обьект из redis по user_id 
    async fn get(conn: &mut impl AsyncCommands, user_id: i64,) -> Result<UsersCalcs, DataError> {
        let key = format!("{}{}", USER_CALCS_PREFIX, user_id);
        let value: String = conn.get(key).await.map_err(|e| {
            DataError::RedisError(format!("Failed to get UsersCalcs from Redis: {}", e))
        })?;
        let users_calcs: UsersCalcs = serde_json::from_str(&value).map_err(|e| {
            DataError::SerializationError(format!("Failed to deserialize UsersCalcs: {}", e))
        })?;
        Ok(users_calcs)
    }
 
    // добавить calc_id в множество calcs для user_id
    // если UsersCalcs не существует, создать новый
    // если есть - добавить к существующему в calcs calc_id
    pub async fn add_calc_to_user(self, conn: &mut impl AsyncCommands,) -> Result<(), DataError> {
        let mut users_calcs = match UsersCalcs::get(conn, self.user_id).await {
            Ok(uc) => uc,
            Err(_) => UsersCalcs::new(self.user_id, self.calcs.clone()),
        };
        users_calcs.calcs.insert(calc_id);
        users_calcs.set(conn).await?;
        Ok(())
    }

    // удаляет расчет calc_id из UserCalcs по user_id
    // если есть другие расчеты удаляе только calc_id из множества calcs
    // если это был последний расчет - удаляет весь обьект UsersCalcs из redis
    pub async fn remove_calc_from_user(self, onn: &mut impl AsyncCommands,) -> Result<(), DataError> {
        let mut users_calcs = UsersCalcs::get(conn, self.user_id).await?;
        users_calcs.calcs.remove(&calc_id);
        if users_calcs.calcs.is_empty() {
            let key = format!("{}{}", USER_CALCS_PREFIX, self.user_id);
            let _: () = conn.del(key).await.map_err(|e| {
                DataError::RedisError(format!("Failed to delete UsersCalcs from Redis: {}", e))
            })?;
        } else {
            users_calcs.set(conn).await?;
        }
        Ok(())
    }
}
