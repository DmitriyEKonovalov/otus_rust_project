use std::sync::Arc;

use teloxide::{prelude::*, types::Message};

use crate::{
    settings::BotState,
};

const NO_USERS_CALCS_MESSAGE: &str = "Нет запущенных расчетов у пользователей";
const USERS_CALCS_HEADER: &str = "Активные расчеты:\n";

//admin only
pub async fn get_active_calcs(
    bot: Bot,
    state: Arc<BotState>,
    dialogue: BotDialogue,
    msg: Message,
) -> HandlerResult {

    // получает команду
    // проверztn is_admin
    // отправляет http запрос в сервис calc_runner на на url get_active_calcs
    // получает ответ
    // отправляет сообщение пользователю с этой инфой

}
