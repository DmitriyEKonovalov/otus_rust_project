use std::sync::Arc;

use teloxide::{prelude::*, types::Message};

use crate::settings::BotState;

const START_MESSAGE: &str = {
    "Добро пожаловать!
    Этот бот умеет запускать расчеты. 
    Доступные команды:
    - /help справка по командам.
    - /base_calc - запуск базового расчета.
    - /mass_calc - запуск массового расчета.
    - /users_calc - просмотр статистки по активным расчетам пользователей.
"};


pub async fn start(
    bot: Bot,
    msg: Message,
    user: User,
    dialogue: BotDialogue,
    _state: Arc<BotState>,
) -> HandlerResult {
    let chat_id = msg.chat.id;
    bot.send_message(chat_id, START_MESSAGE).await?;
    Ok(())
}

