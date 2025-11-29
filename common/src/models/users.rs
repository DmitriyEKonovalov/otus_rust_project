use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;
use role::Role;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: i64,
    pub user_name: String,
    pub user_groups: Role,
}

//
impl User {

    // загрузка пользователя из Redis по user_id
    pub async fn get(conn: &mut impl AsyncCommands, user_id: i64) -> Result<Option<Self>, BotError> {
        let key: String = format!("{}{}", CALC_INFO_PREFIX, calc_id);
        let value: String = conn.get(&key)?;
        let info: CalcInfo = serde_json::from_str(&value).map_err(|e| RedisDataError::NotFound)?;
        Ok(info)
    }

    // соханение пользователя в Redis
    async fn create(&self, conn: &mut impl AsyncCommands) -> Result<(), BotError> {
        let key: String = format!("{}{}", REDIS_USER_PREFIX, self.user_id);
        let json = serde_json::to_string(self)?;
        conn.set(key, json).await?;
        Ok(())
    }
    
    // получение пользователя или созданеи, если нет, с ролью Guest
    pub async fn get_or_create(&self, conn: &mut impl AsyncCommands) -> Result<(), BotError> {
        if Self::get(conn, self.user_id).await?.is_none() {
            self.create(conn).await?;
        }
        Ok(())
    }

    // удаление пользователя из Redis
    pub async fn delete(conn: &mut impl AsyncCommands, user_id: i64) -> Result<(), BotError> {
        let key = format!("{}{}", REDIS_USER_PREFIX, user_id);
        conn.del(key).await?;
        Ok(())
    }

    // установка роли пользователя
    pub async fn set_role(conn: &mut impl AsyncCommands, user_id: i64, role: Role) -> Result<(), BotError> {
        if let Some(mut user) = User::get(conn, user_id).await? {
            user.user_groups = role;
            user.save(conn).await?;
        }
        Ok(())
    }

}


