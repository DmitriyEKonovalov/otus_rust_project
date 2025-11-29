use crate::errors::BotError;
use crate::settings::{BotState, REDIS_USER_CALCS_PREFIX, REDIS_USER_PREFIX};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};
use teloxide::{prelude::*, types::Message};
use uuid::Uuid;

const INITIAL_USERS: &[(i64, &str, Role)] = &[
    (1, "admin", Role::Admin),
    (2, "business", Role::Business),
    (3, "guest", Role::Guest),
];


pub async fn create_users(redis_client: Arc<redis::Client>) -> Result<(), BotError> {
    let mut conn = redis_client.get_async_connection().await?;
    for (id, name, role) in INITIAL_USERS {
        if User::load(&mut conn, *id).await?.is_none() {
            let user = User {
                user_id: *id,
                user_name: name.to_string(),
                user_groups: *role,
            };
            user.save(&mut conn).await?;
        }
    }
    Ok(())
}
