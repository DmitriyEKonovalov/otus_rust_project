use std::sync::Arc;

use teloxide::{prelude::*, types::Message};

use crate::{
    exceptions::HandlerResult,
    models::Role,
    permissions::get_user_role,
    settings::BotState,
};

const HELP_MESSAGE_GUEST: &str = "Привет, я бот для запуска \n\
Доступные команды:\n\
/help - помощь.\n\
/base_calc - запуск базового расчета, требуется ввести параметр <кол-во итераций> (int).\n\
/mass_calc - запуск массового расчета, требуется ввести параметры <кол-во итераций> (int) <данные> (набор данных через запятую).\n\
/users_calc - показать список запущенных расчетов.";

const HELP_MESSAGE_ADMIN: &str = "Привет, я бот для запуска расчетов:\n\
Доступные команды:\n\
/help - помощь.\n\
/base_calc - запуск базового расчета, требуется ввести параметр <кол-во итераций> (int).\n\
/mass_calc - запуск массового расчета, требуется ввести параметры <кол-во итераций> (int) <данные> (набор данных через запятую).\n\
/users_calc - показать список запущенных расчетов.\n\
/active_calcs - показать список всех запущенных расчетов (admin only).";


pub async fn help(bot: Bot, msg: Message, state: Arc<BotState>) -> HandlerResult {
    let user = match msg.from() {
        Some(u) => u.clone(),
        None => {
            bot.send_message(msg.chat.id, HELP_MESSAGE_GUEST).await?;
            return Ok(());
        }
    };

    let role = get_user_role(&user, &state.config.admin_user_ids);
    let message = match role {
        Role::Admin => HELP_MESSAGE_ADMIN,
        _ => HELP_MESSAGE_GUEST,
    };

    bot.send_message(msg.chat.id, message).await?;
    Ok(())
}
