use std::sync::Arc;

use teloxide::{prelude::*, types::Message};


const NO_USERS_CALCS_MESSAGE: &str = "Нет запущенных расчетов у пользователей";
const USERS_CALCS_HEADER: &str = "Активные расчеты:\n";


pub async fn get_user_calcs(
    bot: Bot,
    state: Arc<BotState>,
    dialogue: BotDialogue,
    msg: Message,
) -> HandlerResult {
    
    // получает команду
    // извлекает пользователя 
    // отправляет http запрос в сервис calc_runner на на url get_users_calcs
    // получает ответ
    // отправляет сообщение пользователю с этой инфой и прикрепляет кнопки для статуса расчета
    // если прогресс 100, дать markup  кнопку для получения результата 
}

