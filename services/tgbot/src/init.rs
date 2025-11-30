use std::sync::Arc;

use redis::AsyncCommands;

use crate::errors::BotError;

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
