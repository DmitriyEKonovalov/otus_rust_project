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
    async fn load_raw(conn: &mut impl AsyncCommands, user_id: i64) -> Result<UsersCalcs, DataError> {
        let key = format!("{}{}", USER_CALCS_PREFIX, user_id);
        let value: String = conn.get(&key).await.map_err(|_| DataError::NotFound)?;
        let record: UsersCalcs = serde_json::from_str(&value)?;
        Ok(record)
    }

    async fn create(conn: &mut impl AsyncCommands, user_id: i64) -> Result<UsersCalcs, DataError> {
        let users_calcs = UsersCalcs {
            user_id,
            calcs: HashSet::new(),
        };
        users_calcs.save(conn).await?;
        Ok(users_calcs)
    }

    async fn save(&self, conn: &mut impl AsyncCommands) -> Result<(), DataError> {
        let key = format!("{}{}", USER_CALCS_PREFIX, self.user_id);
        let json = serde_json::to_string(self)?;
        let _: () = conn.set_ex(key, json, USER_CALCS_TTL_SECONDS).await?;
        Ok(())
    }

    pub async fn load(
        conn: &mut impl AsyncCommands,
        user_id: i64,
    ) -> Result<Option<UsersCalcs>, DataError> {
        match UsersCalcs::load_raw(conn, user_id).await {
            Ok(record) => Ok(Some(record)),
            Err(DataError::NotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub async fn add_calc(
        conn: &mut impl AsyncCommands,
        user_id: i64,
        calc_id: Uuid,
    ) -> Result<(), DataError> {
        let mut users_calcs = match UsersCalcs::load_raw(conn, user_id).await {
            Ok(record) => record,
            Err(_) => UsersCalcs::create(conn, user_id).await?,
        };
        users_calcs.calcs.insert(calc_id);
        users_calcs.save(conn).await
    }

    pub async fn remove_calc(
        conn: &mut impl AsyncCommands,
        user_id: i64,
        calc_id: Uuid,
    ) -> Result<(), DataError> {
        let Some(mut users_calcs) = UsersCalcs::load(conn, user_id).await? else {
            return Ok(());
        };

        users_calcs.calcs.remove(&calc_id);
        if users_calcs.calcs.is_empty() {
            let key = format!("{}{}", USER_CALCS_PREFIX, user_id);
            let _: () = conn.del(key).await?;
        } else {
            users_calcs.save(conn).await?;
        }
        Ok(())
    }

    pub async fn list_tracked_users(conn: &mut impl AsyncCommands) -> Result<Vec<i64>, DataError> {
        let keys: Vec<String> = conn.keys(format!("{}*", USER_CALCS_PREFIX)).await?;
        let ids = keys
            .into_iter()
            .filter_map(|key| key.trim_start_matches(USER_CALCS_PREFIX).parse::<i64>().ok())
            .collect();
        Ok(ids)
    }

    pub fn pending_calcs(&self) -> Vec<Uuid> {
        self.calcs.iter().copied().collect()
    }

    pub async fn add_calc_to_user(
        conn: &mut impl AsyncCommands,
        user_id: i64,
        calc_id: Uuid,
    ) -> Result<(), DataError> {
        UsersCalcs::add_calc(conn, user_id, calc_id).await
    }

    pub async fn remove_calc_from_user(
        conn: &mut impl AsyncCommands,
        user_id: i64,
        calc_id: Uuid,
    ) -> Result<(), DataError> {
        UsersCalcs::remove_calc(conn, user_id, calc_id).await
    }
}
