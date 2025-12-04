use std::sync::Arc;
use teloxide::{prelude::*, types::Message};
use crate::settings::BotState;

const HELP_MESSAGE_GUEST: &str = {
    "Добро пожаловать!
    Этот бот умеет запускать расчеты. 
    Доступные команды:
    - /help справка по командам.
    - /base_calc - запуск базового расчета.
    - /mass_calc - запуск массового расчета.
    - /users_calc - просмотр статистки по активным расчетам пользователей.
"};

const HELP_MESSAGE_ADMIN: &str = {
    "Добро пожаловать!
    Этот бот умеет запускать расчеты. 
    Доступные команды:
    - /help справка по командам.
    - /run_base_calc - запуск базового расчета.
    - /run_mass_calc - запуск массового расчета.
    - /get_users_calc - просмотр статистки по активным расчетам пользователей.
    - /get_active_calcs - (АДМИН) получение всех акьтивных расчетов и по всем пользователям 
"};


pub async fn help(
    bot: Bot,
    msg: Message,
    user: User,
    dialogue: BotDialogue,
    _state: Arc<BotState>,
) -> HandlerResult {
    // если guest - показать  HELP_MESSAGE_GUEST
    // если админ - показать HELP_MESSAGE_ADMIN
}

