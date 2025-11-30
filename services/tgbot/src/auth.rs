use std::sync::Arc;

use common::{Role, User, UsersCalcs};
use redis::AsyncCommands;
use teloxide::types::Message;

use crate::{errors::BotError, settings::BotState};

pub use common::{Role, User, UsersCalcs};

fn extract_username(msg: &Message) -> String {
    msg.from
        .as_ref()
        .and_then(|u| u.username.clone())
        .or_else(|| msg.chat.username().map(|u| u.to_string()))
        .unwrap_or_else(|| msg.chat.id.0.to_string())
}

pub async fn ensure_user(state: &Arc<BotState>, msg: &Message) -> Result<User, BotError> {
    let user_id = msg.chat.id.0;
    let user_name = extract_username(msg);
    let mut conn = state.redis_client.get_async_connection().await?;
    let requested = User {
        user_id,
        user_name,
        user_groups: Role::Guest,
    };
    let user = requested.get_or_create(&mut conn).await?;
    Ok(user)
}

pub async fn ensure_user_by_id(
    state: &Arc<BotState>,
    user_id: i64,
    username: Option<String>,
) -> Result<User, BotError> {
    let user_name = username.unwrap_or_else(|| user_id.to_string());
    let mut conn = state.redis_client.get_async_connection().await?;
    let requested = User {
        user_id,
        user_name,
        user_groups: Role::Guest,
    };
    let user = requested.get_or_create(&mut conn).await?;
    Ok(user)
}

pub async fn has_too_many_calcs(state: &Arc<BotState>, user_id: i64) -> Result<bool, BotError> {
    let mut conn = state.redis_client.get_async_connection().await?;
    let count = UsersCalcs::load(&mut conn, user_id)
        .await?
        .map(|c| c.calcs.len())
        .unwrap_or_default();
    Ok(count >= state.config.max_active_calcs)
}

#[macro_export]
macro_rules! require_role {
    ($user:expr, $role:expr, $bot:expr, $chat_id:expr) => {{
        if !$user.user_groups.is_granted($role) {
            $bot
                .send_message($chat_id, format!("Нет прав для выполнения. Нужна роль {:?}.", $role))
                .await?;
            return Ok(());
        }
    }};
}
