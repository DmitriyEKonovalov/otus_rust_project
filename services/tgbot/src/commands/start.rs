use std::sync::Arc;

use teloxide::{prelude::*, types::Message};

use crate::{exceptions::HandlerResult, settings::BotState};

const START_MESSAGE: &str = "Привет, я бот для запуска \n\
Доступные команды:\n\
/help - помощь.\n\
/base_calc - запуск базового расчета, требуется ввести параметр <кол-во итераций> (int).\n\
/mass_calc - запуск массового расчета, требуется ввести параметры <кол-во итераций> (int) <данные> (набор данных через запятую).\n\
/users_calc - показать список запущенных расчетов.\n\
";

pub async fn start(bot: Bot, msg: Message, _state: Arc<BotState>) -> HandlerResult {
    bot.send_message(msg.chat.id, START_MESSAGE).await?;
    Ok(())
}
