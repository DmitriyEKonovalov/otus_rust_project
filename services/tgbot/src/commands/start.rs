use std::sync::Arc;

use teloxide::{prelude::*, types::Message};

use crate::{
    auth::{ensure_user, Role, User},
    commands::{BotDialogue, DialogueState},
    errors::HandlerResult,
    settings::BotState,
};

const START_DESC: &str = "/start - регистрация/проверка";
const HELP_DESC: &str = "/help - помощь";
const CALC_DESC: &str = "/calc - запуск расчетов";
const USERS_CALC_DESC: &str = "/users_calc - активные расчеты пользователей";
const GREETING_TEMPLATE: &str = "Привет, {}!\nТвои роли: {:?}\nДоступные команды:\n{}";

pub async fn start_handle(
    bot: Bot,
    state: Arc<BotState>,
    dialogue: BotDialogue,
    msg: Message,
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let user = ensure_user(&state, &msg).await?;

    bot.send_message(chat_id, commands_help(&user)).await?;
    dialogue.update(DialogueState::Idle).await?;
    Ok(())
}

pub fn commands_help(user: &User) -> String {
    let mut commands = vec![START_DESC.to_string(), HELP_DESC.to_string()];
    if user.user_groups.is_granted(Role::Business) {
        commands.push(CALC_DESC.to_string());
    }
    if user.user_groups.is_granted(Role::Admin) {
        commands.push(USERS_CALC_DESC.to_string());
    }

    format!(
        GREETING_TEMPLATE,
        user.user_name,
        user.user_groups,
        commands.join("\n")
    )
}
