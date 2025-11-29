use std::sync::Arc;

use teloxide::{prelude::*, types::Message};

use super::calc::get_status;
use crate::{
    auth::{ensure_user, Role, User, UsersCalcs},
    commands::{BotDialogue, DialogueState},
    errors::HandlerResult,
    require_role,
    settings::BotState,
};

const NO_USERS_CALCS_MESSAGE: &str = "Нет запущенных расчетов у пользователей";
const USERS_CALCS_HEADER: &str = "Активные расчеты:\n";

pub async fn stats_handle(
    bot: Bot,
    state: Arc<BotState>,
    dialogue: BotDialogue,
    msg: Message,
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let user = ensure_user(&state, &msg).await?;
    require_role!(user, Role::Admin, bot, chat_id);

    let mut conn = state.redis_client.get_async_connection().await?;
    let user_ids = UsersCalcs::list_tracked_users(&mut conn).await?;
    drop(conn);

    if user_ids.is_empty() {
        bot.send_message(chat_id, NO_USERS_CALCS_MESSAGE).await?;
        dialogue.update(DialogueState::Idle).await?;
        return Ok(());
    }

    let mut rows = Vec::new();
    for uid in user_ids {
        let mut conn = state.redis_client.get_async_connection().await?;
        let calcs = UsersCalcs::load(&mut conn, uid).await?;
        let Some(records) = calcs else { continue };
        if records.calcs.is_empty() {
            continue;
        }
        let user_info = User::load(&mut conn, uid)
            .await?
            .map(|u| u.user_name)
            .unwrap_or_else(|| "unknown".to_string());
        drop(conn);

        let mut calc_lines = Vec::new();
        for calc_id in records.pending_calcs() {
            let status = get_status(&state, calc_id).await;
            match status {
                Ok(st) => calc_lines.push(format!("{} ({}%)", calc_id, st.progress)),
                Err(_) => calc_lines.push(format!("{} (н/д)", calc_id)),
            }
        }

        if !calc_lines.is_empty() {
            rows.push(format!("{} ({}): {}", user_info, uid, calc_lines.join(", ")));
        }
    }

    if rows.is_empty() {
        bot.send_message(chat_id, NO_USERS_CALCS_MESSAGE).await?;
    } else {
        bot.send_message(chat_id, format!("{USERS_CALCS_HEADER}{}", rows.join("\n"))).await?;
    }

    dialogue.update(DialogueState::Idle).await?;
    Ok(())
}
