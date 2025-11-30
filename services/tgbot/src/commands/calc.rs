use std::{sync::Arc, time::Instant};

use chrono::{DateTime, Utc};
use common::{CalcInfo, DataError};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use teloxide::{
    prelude::*,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message},
};
use uuid::Uuid;

use crate::{
    auth::{ensure_user, ensure_user_by_id, has_too_many_calcs, Role, User, UsersCalcs},
    commands::{BotDialogue, DialogueState},
    errors::{BotError, HandlerResult},
    require_role,
    settings::{BotState, RESULT_POLL_INTERVAL, SEND_THROTTLE},
};

const INVALID_ITERATIONS_MESSAGE: &str = "Не удалось разобрать iterations";
const ZERO_ITERATIONS_MESSAGE: &str = "Количество итераций должно быть > 0";
const TOO_MANY_CALCS_MESSAGE: &str = "Попробуйте позже, у вас уже есть активные расчеты";
const FULL_CALC_FORMAT_MESSAGE: &str = "Формат запроса: <iterations> <num1> <num2> ...";
const CALC_STARTED_TEMPLATE: &str = "Расчет запущен. ID: {}\nРезультат отправим позже.";
const FULL_CALC_STARTED_TEMPLATE: &str =
    "Расчет FullCalc запущен. ID: {}\nВходные данные: iterations {}, data {:?}";
const UNKNOWN_COMMAND_MESSAGE: &str = "Команда не распознана. Используйте /help";
const BASE_ITERATIONS_PROMPT: &str = "Введите iterations (целое число)";
const FULL_CALC_PROMPT: &str = "Введите iterations и данные через пробел. Пример: 10 5 7 9";
const CANCELLED_MESSAGE: &str = "Отменено";
const INVALID_CALC_ID_MESSAGE: &str = "Неверный ID расчета";
const NO_CALCS_MESSAGE: &str = "Нет запущенных расчетов";
const STATUS_TEMPLATE: &str = "Расчет {}\nПрогресс: {}%\nДлительность: {} сек";
const RESULT_MISSING_MESSAGE: &str = "Результат отсутствует";
const RESULT_TEXT_TEMPLATE: &str = "Расчет {}\nЗапущен: {}\nЗавершен: {}\nПараметры: {}\nРезультат:\n{}";
const WATCHER_RESULT_TEMPLATE: &str =
    "Расчет {} завершен.\nЗапуск: {}\nЗавершен: {}\nПрогресс: {}%\nРезультат:\n{}";
const CALC_MENU_TEXT: &str = "Доступные расчеты:\n\
BaseCalc - базовая проверка заданное количество iterations.\n\
FullCalc - базовая проверка набора чисел с iterations.\n\
InProgress - показать запущенные расчеты.";
const BUTTON_LABEL_BASE: &str = "BaseCalc";
const BUTTON_LABEL_FULL: &str = "FullCalc";
const BUTTON_LABEL_PROGRESS: &str = "InProgress";
const BUTTON_LABEL_CANCEL: &str = "Отмена";
const BUTTON_LABEL_STATUS: &str = "Status";
const BUTTON_LABEL_RESULT: &str = "Result";
const CALLBACK_BASE: &str = "calc:base";
const CALLBACK_FULL: &str = "calc:full";
const CALLBACK_PROGRESS: &str = "calc:progress";
const CALLBACK_CANCEL: &str = "calc:cancel";
const STATUS_PREFIX: &str = "status:";
const RESULT_PREFIX: &str = "result:";

pub async fn calc_handle(
    bot: Bot,
    state: Arc<BotState>,
    dialogue: BotDialogue,
    msg: Message,
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let user = ensure_user(&state, &msg).await?;
    require_role!(user, Role::Business, bot, chat_id);

    bot.send_message(chat_id, CALC_MENU_TEXT)
        .reply_markup(calc_menu_keyboard())
        .await?;

    dialogue.update(DialogueState::Idle).await?;
    Ok(())
}

pub async fn message_handle(
    bot: Bot,
    state: Arc<BotState>,
    dialogue: BotDialogue,
    msg: Message,
) -> HandlerResult {
    let Some(text) = msg.text().map(str::trim) else {
        return Ok(());
    };

    let chat_id = msg.chat.id;
    let state_value = dialogue.get().await?.unwrap_or_default();

    match state_value {
        DialogueState::AwaitingBaseIterations => {
            let iterations: u32 = match text.parse() {
                Ok(v) => v,
                Err(_) => {
                    bot.send_message(chat_id, INVALID_ITERATIONS_MESSAGE).await?;
                    return Ok(());
                }
            };

            if iterations == 0 {
                bot.send_message(chat_id, ZERO_ITERATIONS_MESSAGE).await?;
                return Ok(());
            }

            let user = ensure_user(&state, &msg).await?;
            require_role!(user, Role::Business, bot, chat_id);

            if has_too_many_calcs(&state, user.user_id).await? {
                bot.send_message(chat_id, TOO_MANY_CALCS_MESSAGE).await?;
                dialogue.update(DialogueState::Idle).await?;
                return Ok(());
            }

            let calc_id = start_base_calc(&state, iterations).await?;
            let mut conn = state.redis_client.get_async_connection().await?;
            UsersCalcs::add_calc(&mut conn, user.user_id, calc_id).await?;

            bot.send_message(chat_id, format!(CALC_STARTED_TEMPLATE, calc_id))
                .reply_markup(status_markup(calc_id, false))
                .await?;

            dialogue.update(DialogueState::Idle).await?;
        }
        DialogueState::AwaitingFullCalc => {
            let Some((iterations, data)) = parse_full_calc_payload(text) else {
                bot.send_message(chat_id, FULL_CALC_FORMAT_MESSAGE).await?;
                return Ok(());
            };

            let user = ensure_user(&state, &msg).await?;
            require_role!(user, Role::Business, bot, chat_id);

            if has_too_many_calcs(&state, user.user_id).await? {
                bot.send_message(chat_id, TOO_MANY_CALCS_MESSAGE).await?;
                dialogue.update(DialogueState::Idle).await?;
                return Ok(());
            }

            let calc_id = start_full_calc(&state, iterations, data.clone()).await?;
            let mut conn = state.redis_client.get_async_connection().await?;
            UsersCalcs::add_calc(&mut conn, user.user_id, calc_id).await?;

            bot.send_message(
                chat_id,
                format!(FULL_CALC_STARTED_TEMPLATE, calc_id, iterations, data),
            )
            .reply_markup(status_markup(calc_id, false))
            .await?;

            dialogue.update(DialogueState::Idle).await?;
        }
        DialogueState::Idle => {
            bot.send_message(chat_id, UNKNOWN_COMMAND_MESSAGE).await?;
        }
    }

    Ok(())
}

pub async fn callback_handle(
    bot: Bot,
    state: Arc<BotState>,
    dialogue: BotDialogue,
    q: CallbackQuery,
) -> HandlerResult {
    let Some(data) = q.data.clone() else {
        return Ok(());
    };
    let chat_id = q
        .message
        .as_ref()
        .map(|m| m.chat.id)
        .unwrap_or(ChatId(q.from.id.0 as i64));

    match data.as_str() {
        CALLBACK_BASE => {
            bot.answer_callback_query(q.id).await?;
            let user = ensure_user_by_id(&state, q.from.id.0 as i64, q.from.username.clone()).await?;
            require_role!(user, Role::Business, bot, chat_id);
            if has_too_many_calcs(&state, user.user_id).await? {
                bot.send_message(chat_id, TOO_MANY_CALCS_MESSAGE).await?;
                dialogue.update(DialogueState::Idle).await?;
                return Ok(());
            }
            bot.send_message(chat_id, BASE_ITERATIONS_PROMPT).await?;
            dialogue.update(DialogueState::AwaitingBaseIterations).await?;
        }
        CALLBACK_FULL => {
            bot.answer_callback_query(q.id).await?;
            let user = ensure_user_by_id(&state, q.from.id.0, q.from.username.clone()).await?;
            require_role!(user, Role::Business, bot, chat_id);
            if has_too_many_calcs(&state, user.user_id).await? {
                bot.send_message(chat_id, TOO_MANY_CALCS_MESSAGE).await?;
                dialogue.update(DialogueState::Idle).await?;
                return Ok(());
            }
            bot.send_message(chat_id, FULL_CALC_PROMPT).await?;
            dialogue.update(DialogueState::AwaitingFullCalc).await?;
        }
        CALLBACK_PROGRESS => {
            bot.answer_callback_query(q.id).await?;
            if let Some(message) = q.message {
                let user = ensure_user(&state, &message).await?;
                show_in_progress(&bot, &state, &user, message.chat.id).await?;
            }
        }
        CALLBACK_CANCEL => {
            bot.answer_callback_query(q.id).await?;
            bot.send_message(chat_id, CANCELLED_MESSAGE).await?;
            dialogue.update(DialogueState::Idle).await?;
        }
        other if other.starts_with(STATUS_PREFIX) => {
            bot.answer_callback_query(q.id).await?;
            let id_str = other.trim_start_matches(STATUS_PREFIX);
            match Uuid::parse_str(id_str) {
                Ok(calc_id) => send_status(&bot, &state, chat_id, calc_id).await?,
                Err(_) => bot.send_message(chat_id, INVALID_CALC_ID_MESSAGE).await?,
            }
        }
        other if other.starts_with(RESULT_PREFIX) => {
            bot.answer_callback_query(q.id).await?;
            let id_str = other.trim_start_matches(RESULT_PREFIX);
            match Uuid::parse_str(id_str) {
                Ok(calc_id) => send_result(&bot, &state, chat_id, calc_id).await?,
                Err(_) => bot.send_message(chat_id, INVALID_CALC_ID_MESSAGE).await?,
            }
        }
        _ => {
            bot.answer_callback_query(q.id).await?;
        }
    }

    Ok(())
}

pub async fn send_status(bot: &Bot, state: &Arc<BotState>, chat_id: ChatId, calc_id: Uuid) -> HandlerResult {
    match get_status(state, calc_id).await {
        Ok(status) => {
            let markup = status_markup(calc_id, status.progress == 100);
            let text = format!(STATUS_TEMPLATE, calc_id, status.progress, status.duration);

            bot.send_message(chat_id, text).reply_markup(markup).await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("Не удалось получить статус: {}", e)).await?;
        }
    }
    Ok(())
}

pub async fn send_result(bot: &Bot, state: &Arc<BotState>, chat_id: ChatId, calc_id: Uuid) -> HandlerResult {
    match get_result(state, calc_id).await {
        Ok(result) => {
            let result_json = result
                .result
                .map(|v| serde_json::to_string_pretty(&v))
                .transpose()?
                .unwrap_or_else(|| RESULT_MISSING_MESSAGE.to_string());
            let text = format!(
                RESULT_TEXT_TEMPLATE,
                result.calc_id,
                result.run_dt,
                result
                    .end_dt
                    .map(|dt| dt.to_string())
                    .unwrap_or_else(|| "в процессе".to_string()),
                result
                    .params
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                result_json
            );
            bot.send_message(chat_id, text).await?;
            let mut conn = state.redis_client.get_async_connection().await?;
            UsersCalcs::remove_calc(&mut conn, chat_id.0, calc_id).await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("Не удалось получить результат: {}", e)).await?;
        }
    }
    Ok(())
}

pub async fn get_status(state: &Arc<BotState>, calc_id: Uuid) -> Result<CalcStatusResponse, BotError> {
    let url = format!("{}/api/calc/{}", state.config.calc_runner_base, calc_id);
    let resp = state.http_client.post(url).send().await?;
    if resp.status() == StatusCode::NOT_FOUND {
        return Err(BotError::Parse("Расчет не найден".into()));
    }
    let body: CalcStatusResponse = resp.json().await?;
    Ok(body)
}

pub async fn get_result(state: &Arc<BotState>, calc_id: Uuid) -> Result<CalcResultResponse, BotError> {
    let url = format!("{}/api/calc/result/{}", state.config.calc_runner_base, calc_id);
    let resp = state.http_client.post(url).send().await?;
    if resp.status() == StatusCode::NOT_FOUND {
        return Err(BotError::Parse("Расчет не найден".into()));
    }
    let body: CalcResultResponse = resp.json().await?;
    Ok(body)
}

pub async fn run_result_watcher(bot: Bot, state: Arc<BotState>) {
    let mut last_send: Option<Instant> = None;
    loop {
        if let Err(err) = check_and_send_results(&bot, &state, &mut last_send).await {
            eprintln!("result watcher error: {err}");
        }
        tokio::time::sleep(RESULT_POLL_INTERVAL).await;
    }
}

async fn check_and_send_results(
    bot: &Bot,
    state: &Arc<BotState>,
    last_send: &mut Option<Instant>,
) -> HandlerResult {
    let mut conn = state.redis_client.get_async_connection().await?;
    let user_ids = UsersCalcs::list_tracked_users(&mut conn).await?;
    drop(conn);

    for user_id in user_ids {
        let mut conn = state.redis_client.get_async_connection().await?;
        if let Some(user_calcs) = UsersCalcs::load(&mut conn, user_id).await? {
            for calc_id in user_calcs.calcs.iter().copied().collect::<Vec<_>>() {
                if let Some(calc_info) = fetch_calc_info(&mut conn, calc_id).await? {
                    if calc_info.end_dt.is_some() {
                        let text = format_result_message(&calc_info);
                        let chat_id = ChatId(user_id);
                        if let Some(ts) = last_send {
                            if ts.elapsed() < SEND_THROTTLE {
                                tokio::time::sleep(SEND_THROTTLE - ts.elapsed()).await;
                            }
                        }
                        bot.send_message(chat_id, text).await?;
                        UsersCalcs::remove_calc(&mut conn, user_id, calc_id).await?;
                        *last_send = Some(Instant::now());
                    }
                }
            }
        }
    }

    Ok(())
}

async fn fetch_calc_info(
    conn: &mut impl redis::AsyncCommands,
    calc_id: Uuid,
) -> Result<Option<CalcInfo>, BotError> {
    match CalcInfo::get(conn, calc_id).await {
        Ok(info) => Ok(Some(info)),
        Err(DataError::NotFound) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

fn format_result_message(info: &CalcInfo) -> String {
    let result_json = info
        .result
        .as_ref()
        .map(|v| serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string()))
        .unwrap_or_else(|| RESULT_MISSING_MESSAGE.to_string());

    format!(
        WATCHER_RESULT_TEMPLATE,
        info.calc_id,
        info.run_dt,
        info.end_dt
            .map(|dt| dt.to_string())
            .unwrap_or_else(|| "в процессе".to_string()),
        info.progress,
        result_json
    )
}

fn calc_menu_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback(BUTTON_LABEL_BASE, CALLBACK_BASE)],
        vec![InlineKeyboardButton::callback(BUTTON_LABEL_FULL, CALLBACK_FULL)],
        vec![InlineKeyboardButton::callback(
            BUTTON_LABEL_PROGRESS,
            CALLBACK_PROGRESS,
        )],
        vec![InlineKeyboardButton::callback(BUTTON_LABEL_CANCEL, CALLBACK_CANCEL)],
    ])
}

fn status_markup(calc_id: Uuid, finished: bool) -> InlineKeyboardMarkup {
    let mut rows = vec![vec![InlineKeyboardButton::callback(
        BUTTON_LABEL_STATUS,
        format!("{STATUS_PREFIX}{calc_id}"),
    )]];
    if finished {
        rows.push(vec![InlineKeyboardButton::callback(
            BUTTON_LABEL_RESULT,
            format!("{RESULT_PREFIX}{calc_id}"),
        )]);
    }
    InlineKeyboardMarkup::new(rows)
}

fn parse_full_calc_payload(text: &str) -> Option<(u32, Vec<u32>)> {
    let mut parts = text.split_whitespace();
    let iterations: u32 = parts.next()?.parse().ok()?;
    let data: Vec<u32> = parts.map(|p| p.parse().ok()).collect::<Option<_>>()?;
    if data.is_empty() || iterations == 0 {
        return None;
    }
    Some((iterations, data))
}

async fn show_in_progress(bot: &Bot, state: &Arc<BotState>, user: &User, chat_id: ChatId) -> HandlerResult {
    let mut conn = state.redis_client.get_async_connection().await?;
    let Some(records) = UsersCalcs::load(&mut conn, user.user_id).await? else {
        bot.send_message(chat_id, NO_CALCS_MESSAGE).await?;
        return Ok(());
    };

    if records.calcs.is_empty() {
        bot.send_message(chat_id, NO_CALCS_MESSAGE).await?;
        return Ok(());
    }

    let mut lines = Vec::new();
    let mut buttons = Vec::new();

    for calc_id in records.calcs.iter() {
        match get_status(state, *calc_id).await {
            Ok(status) => {
                lines.push(format!(
                    "ID: {} | Прогресс: {}% | Длительность: {} сек",
                    calc_id, status.progress, status.duration
                ));
                buttons.push(vec![InlineKeyboardButton::callback(
                    format!("Статус {}", calc_id),
                    format!("{STATUS_PREFIX}{calc_id}"),
                )]);
            }
            Err(_) => {
                lines.push(format!("ID: {} | Статус недоступен", calc_id));
            }
        }
    }

    bot.send_message(chat_id, lines.join("\n"))
        .reply_markup(InlineKeyboardMarkup::new(buttons))
        .await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BaseCalcRequest {
    iterations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FullCalcRequest {
    iterations: u32,
    data: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunCalcResponse {
    calc_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcStatusResponse {
    run_dt: DateTime<Utc>,
    progress: u32,
    duration: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcResultResponse {
    calc_id: Uuid,
    run_dt: DateTime<Utc>,
    end_dt: Option<DateTime<Utc>>,
    params: Option<serde_json::Value>,
    progress: u32,
    result: Option<serde_json::Value>,
    duration: Option<i64>,
}

async fn start_base_calc(state: &Arc<BotState>, iterations: u32) -> Result<Uuid, BotError> {
    let url = format!("{}/api/calc/base_calc", state.config.calc_runner_base);
    let resp = state
        .http_client
        .post(url)
        .json(&BaseCalcRequest { iterations })
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(BotError::Parse(format!(
            "calc_runner вернул ошибку: {}",
            resp.status()
        )));
    }
    let body: RunCalcResponse = resp.json().await?;
    Ok(body.calc_id)
}

async fn start_full_calc(state: &Arc<BotState>, iterations: u32, data: Vec<u32>) -> Result<Uuid, BotError> {
    let url = format!("{}/api/calc/mass_calc", state.config.calc_runner_base);
    let resp = state
        .http_client
        .post(url)
        .json(&FullCalcRequest { iterations, data })
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(BotError::Parse(format!(
            "calc_runner вернул ошибку: {}",
            resp.status()
        )));
    }
    let body: RunCalcResponse = resp.json().await?;
    Ok(body.calc_id)
}
