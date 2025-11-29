use redis::{AsyncCommands, RedisResult, RedisDataError};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};
use uuid::Uuid;
use std::collections::HashMap;

const USER_CALCS_PREFIX: &str = "user_calc:";
const USER_CALCS_TTL_SECONDS: u64 = 24 * 3600; 

// Дополнительная структура для хранения информации о расчетах пользователя
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersCalcs {
    pub user_id: i64,
    pub calcs: HashSet<Uuid>,
}

impl UsersCalcs {
    
    // получение записей о расчетах пользователя из Redis (внутренняя)
    async fn get(conn: &mut impl AsyncCommands, user_id: i64) -> RedisResult<UsersCalcs, RedisDataError::NotFound> {
        let key = format!("{}{}", REDIS_USER_CALCS_PREFIX, user_id);
        let value: Option<String> = conn.get(&key).await?;  
        match value {
            Some(v) => {
                let record: UsersCalcs = serde_json::from_str(&v)?;
                Ok(Some(record))
            },
            None => Ok(None),
        }
    }
    
    // создание пустой записи о расчетах пользователя в Redis (внутренняя)
    async fn create(conn: &mut impl AsyncCommands, user_id: i64) -> RedisResult<> {
        let record = UsersCalcs {
            user_id,
            calcs: HashSet::new(),
        };
        let key = format!("{}{}", REDIS_USER_CALCS_PREFIX, user_id);
        let json = serde_json::to_string(&record)?;
        conn.set(key, json).await?;
        Ok(Some(record))
    }

    // обновление записей о расчетах пользователя в Redis (внутренняя)
    async fn update(&self, conn: &mut impl AsyncCommands) -> RedisResult<UsersCalcs> {
        let key = format!("{}{}", REDIS_USER_CALCS_PREFIX, self.user_id);
        let json = serde_json::to_string(self)?;
        conn.set(key, json).await?;
        Ok(())
    }
    
    // добавление расчета пользователя в Redis 
    //  - если записи для пользователя нет, создается новая с добавленным расчетом
    //  - если запись есть, расчет добавляется в коллекцию расчетов
    pub async fn add_calc_to_user(conn: &mut impl AsyncCommands, user_id: i64, calc_id: Uuid) -> RedisResult<UsersCalcs> {
        let mut users_calcs = match UsersCalcs::get(&mut conn, user_id).await? {
            Some(r) => r,
            None => UsersCalcs::create(&mut conn, user_id).await?.unwrap(),
        };
        users_calcs.calcs.insert(calc_id);
        users_calcs.update(&mut conn).await?;
        Ok(())
    }

    // удаление расчета пользователя из Redis, если расчетов нет - удаление всей записи
    pub async fn remove_calc_from_user(conn: &mut impl AsyncCommands, user_id: i64, calc_id: Uuid) -> RedisResult<UsersCalcs> {
        if let Some(mut users_calcs) = UsersCalcs::get(&mut conn, user_id).await? {
            users_calcs.calcs.remove(&calc_id);
            if users_calcs.calcs.is_empty() {
                let key = format!("{}{}", REDIS_USER_CALCS_PREFIX, user_id);
                conn.del(key).await?;
            } else {
                users_calcs.update(&mut conn).await?;
            }
        }
        Ok(())
    }

}

// Структура для хранения статистики по расчетам всем пользователей 
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersCalcStats {
    pub total_running: u32,
    pub calcs: HashMap<u64,Vec<UsersCalcs>>,
}

impl UsersCalcStats {
    // Получение статичтики из Redis
    pub async fn get_stats(conn: &mut impl AsyncCommands) -> RedisResult<UsersCalcStats> {
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(format!("{}*", REDIS_USER_CALCS_PREFIX))
            .query_async(conn)
            .await?;
        let ids = keys
            .into_iter()
            .filter_map(|k| k.strip_prefix(REDIS_USER_CALCS_PREFIX))
            .filter_map(|id| id.parse::<i64>().ok())
            .collect();
        Ok(ids)
    }

    pub fn pending_calcs(&self) -> Vec<Uuid> {
        self.calcs.iter().copied().collect()
    }

}