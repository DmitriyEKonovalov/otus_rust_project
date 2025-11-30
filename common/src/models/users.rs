use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::models::errors::DataError;
use crate::models::roles::Role;

const USER_PREFIX: &str = "user:";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: i64,
    pub user_name: String,
    pub user_groups: Role,
}

impl User {
    pub async fn save(&self, conn: &mut impl AsyncCommands) -> Result<(), DataError> {
        let key: String = format!("{}{}", USER_PREFIX, self.user_id);
        let json = serde_json::to_string(self)?;
        let _: () = conn.set(key, json).await?;
        Ok(())
    }

    pub async fn get(conn: &mut impl AsyncCommands, user_id: i64) -> Result<User, DataError> {
        let key: String = format!("{}{}", USER_PREFIX, user_id);
        let value: String = conn.get(&key).await.map_err(|_| DataError::NotFound)?;
        let info: User = serde_json::from_str(&value).map_err(|_| DataError::NotFound)?;
        Ok(info)
    }

    pub async fn load(conn: &mut impl AsyncCommands, user_id: i64) -> Result<Option<User>, DataError> {
        match User::get(conn, user_id).await {
            Ok(user) => Ok(Some(user)),
            Err(DataError::NotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub async fn get_or_create(&self, conn: &mut impl AsyncCommands) -> Result<User, DataError> {
        match User::get(conn, self.user_id).await {
            Ok(user) => Ok(user),
            Err(_) => {
                let new_user = User {
                    user_id: self.user_id,
                    user_name: self.user_name.clone(),
                    user_groups: Role::Guest,
                };
                new_user.save(conn).await?;
                Ok(new_user)
            }
        }
    }

    pub async fn delete(conn: &mut impl AsyncCommands, user_id: i64) -> Result<(), DataError> {
        let key: String = format!("{}{}", USER_PREFIX, user_id);
        let _: () = conn.del(key).await?;
        Ok(())
    }

    pub async fn set_role(conn: &mut impl AsyncCommands, user_id: i64, role: Role) -> Result<(), DataError> {
        let mut user = User::get(conn, user_id).await?;
        user.user_groups = role;
        user.save(conn).await
    }
}
