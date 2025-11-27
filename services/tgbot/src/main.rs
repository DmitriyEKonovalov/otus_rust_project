use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use common::calc_info::CalcInfo;
use dotenvy::dotenv;
use redis::AsyncCommands;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{
        dialogue::{Dialogue, InMemStorage},
        UpdateFilterExt,
    },
    dptree,
    prelude::*,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, Message},
    utils::command::BotCommands,
};
use thiserror::Error;
use uuid::Uuid;

const REDIS_USER_PREFIX: &str = "USER:";
const REDIS_USER_CALCS_PREFIX: &str = "USERS_CALC::";
const REDIS_CALC_PREFIX: &str = "calc:";
const DEFAULT_MAX_CALCS: usize = 3;
const RESULT_POLL_INTERVAL: Duration = Duration::from_secs(15);
const SEND_THROTTLE: Duration = Duration::from_millis(800);

#[derive(Clone)]
struct BotConfig {
    calc_runner_base: String,
    max_active_calcs: usize,
}

#[derive(Clone)]
struct BotState {
    redis_client: Arc<redis::Client>,
    http_client: reqwest::Client,
    config: BotConfig,
}

#[derive(Debug, Error)]
enum BotError {
    #[error("Telegram error: {0}")]
    Telegram(#[from] teloxide::RequestError),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

type HandlerResult = Result<(), BotError>;
type BotDialogue = Dialogue<DialogueState, InMemStorage<DialogueState>>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
enum DialogueState {
    #[default]
    Idle,
    AwaitingBaseIterations,
    AwaitingFullCalc,
}

#[derive(Debug, Clone, BotCommands)]
#[command(rename_rule = "lowercase", description = "Доступные команды:")]
enum Command {
    /// Запуск и регистрация пользователя
    Start,
    /// Справка
    Help,
    /// Запуск расчета
    Calc,
    /// Список всех активных расчетов (админ)
    UsersCalc,
}

#[tokio::main]
async fn main() -> Result<(), BotError> {
    dotenv().ok();

    let bot_token = std::env::var("BOT_TOKEN").expect("BOT_TOKEN is required in .env");
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let calc_runner_base =
        std::env::var("CALC_RUNNER_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());

    let bot = Bot::new(bot_token);
    let redis_client = Arc::new(redis::Client::open(redis_url)?);
    let mut ping_conn = redis_client.get_async_connection().await?;
    let _: String = redis::cmd("PING").query_async(&mut ping_conn).await?;

    seed_users(redis_client.clone()).await?;

    let state = Arc::new(BotState {
        redis_client,
        http_client: reqwest::Client::new(),
        config: BotConfig {
            calc_runner_base,
            max_active_calcs: DEFAULT_MAX_CALCS,
        },
    });

    let bot_for_worker = bot.clone();
    let state_for_worker = state.clone();
    tokio::spawn(async move {
        run_result_watcher(bot_for_worker, state_for_worker).await;
    });

    let message_handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<DialogueState>, DialogueState>()
        .branch(dptree::entry().filter_command::<Command>().endpoint(handle_command))
        .branch(dptree::endpoint(handle_message));

    let callback_handler = Update::filter_callback_query()
        .enter_dialogue::<CallbackQuery, InMemStorage<DialogueState>, DialogueState>()
        .endpoint(handle_callback);

    let handler = dptree::entry()
        .branch(message_handler)
        .branch(callback_handler);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state, InMemStorage::<DialogueState>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn handle_command(
    bot: Bot,
    state: Arc<BotState>,
    dialogue: BotDialogue,
    msg: Message,
    command: Command,
) -> HandlerResult {
    match command {
        Command::Start => handle_start(bot, state, dialogue, msg).await,
        Command::Help => handle_help(bot, state, dialogue, msg).await,
        Command::Calc => handle_calc(bot, state, dialogue, msg).await,
        Command::UsersCalc => handle_users_calc(bot, state, dialogue, msg).await,
    }
}

async fn handle_message(bot: Bot, state: Arc<BotState>, dialogue: BotDialogue, msg: Message) -> HandlerResult {
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
                    bot.send_message(chat_id, "Не удалось разобрать iterations").await?;
                    return Ok(());
                }
            };

            if iterations == 0 {
                bot.send_message(chat_id, "Количество итераций должно быть > 0").await?;
                return Ok(());
            }

            let user = ensure_user(&state, &msg).await?;
            require_role!(user, Role::Business, bot, chat_id);

            if has_too_many_calcs(&state, user.user_id).await? {
                bot.send_message(chat_id, "Попробуйте позже, у вас уже есть активные расчеты").await?;
                dialogue.update(DialogueState::Idle).await?;
                return Ok(());
            }

            let calc_id = start_base_calc(&state, iterations).await?;
            UsersCalcs::add_calc(&state, user.user_id, calc_id).await?;

            bot.send_message(
                chat_id,
                format!("Расчет запущен. ID: {}\nОжидайте завершения.", calc_id),
            )
            .reply_markup(status_markup(calc_id, false))
            .await?;

            dialogue.update(DialogueState::Idle).await?;
        }
        DialogueState::AwaitingFullCalc => {
            let Some((iterations, data)) = parse_full_calc_payload(text) else {
                bot.send_message(
                    chat_id,
                    "Ожидается формат: <iterations> <num1> <num2> ...",
                )
                .await?;
                return Ok(());
            };

            let user = ensure_user(&state, &msg).await?;
            require_role!(user, Role::Business, bot, chat_id);

            if has_too_many_calcs(&state, user.user_id).await? {
                bot.send_message(chat_id, "Попробуйте позже, у вас уже есть активные расчеты").await?;
                dialogue.update(DialogueState::Idle).await?;
                return Ok(());
            }

            let calc_id = start_full_calc(&state, iterations, data.clone()).await?;
            UsersCalcs::add_calc(&state, user.user_id, calc_id).await?;

            bot.send_message(
                chat_id,
                format!(
                    "Расчет FullCalc запущен. ID: {}\nПараметры: iterations {}, data {:?}",
                    calc_id, iterations, data
                ),
            )
            .reply_markup(status_markup(calc_id, false))
            .await?;

            dialogue.update(DialogueState::Idle).await?;
        }
        DialogueState::Idle => {
            bot.send_message(chat_id, "Команда не распознана. Используйте /help").await?;
        }
    }

    Ok(())
}

async fn handle_callback(
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
        .unwrap_or(ChatId(q.from.id.0));

    match data.as_str() {
        "calc:base" => {
            bot.answer_callback_query(q.id).await?;
            let user = ensure_user_by_id(&state, q.from.id.0, q.from.username.clone()).await?;
            require_role!(user, Role::Business, bot, chat_id);
            if has_too_many_calcs(&state, user.user_id).await? {
                bot.send_message(chat_id, "Попробуйте позже, у вас уже есть активные расчеты").await?;
                dialogue.update(DialogueState::Idle).await?;
                return Ok(());
            }
            bot.send_message(chat_id, "Введите iterations (целое число)").await?;
            dialogue.update(DialogueState::AwaitingBaseIterations).await?;
        }
        "calc:full" => {
            bot.answer_callback_query(q.id).await?;
            let user = ensure_user_by_id(&state, q.from.id.0, q.from.username.clone()).await?;
            require_role!(user, Role::Business, bot, chat_id);
            if has_too_many_calcs(&state, user.user_id).await? {
                bot.send_message(chat_id, "Попробуйте позже, у вас уже есть активные расчеты").await?;
                dialogue.update(DialogueState::Idle).await?;
                return Ok(());
            }
            bot.send_message(
                chat_id,
                "Введите iterations и список чисел через пробел. Пример: 10 5 7 9",
            )
            .await?;
            dialogue.update(DialogueState::AwaitingFullCalc).await?;
        }
        "calc:progress" => {
            bot.answer_callback_query(q.id).await?;
            if let Some(message) = q.message {
                let user = ensure_user(&state, &message).await?;
                show_in_progress(&bot, &state, &user, message.chat.id).await?;
            }
        }
        "calc:cancel" => {
            bot.answer_callback_query(q.id).await?;
            bot.send_message(chat_id, "Отменено").await?;
            dialogue.update(DialogueState::Idle).await?;
        }
        other if other.starts_with("status:") => {
            bot.answer_callback_query(q.id).await?;
            let id_str = other.trim_start_matches("status:");
            match Uuid::parse_str(id_str) {
                Ok(calc_id) => send_status(&bot, &state, chat_id, calc_id).await?,
                Err(_) => bot.send_message(chat_id, "Неверный ID расчета").await?,
            }
        }
        other if other.starts_with("result:") => {
            bot.answer_callback_query(q.id).await?;
            let id_str = other.trim_start_matches("result:");
            match Uuid::parse_str(id_str) {
                Ok(calc_id) => send_result(&bot, &state, chat_id, calc_id).await?,
                Err(_) => bot.send_message(chat_id, "Неверный ID расчета").await?,
            }
        }
        _ => {
            bot.answer_callback_query(q.id).await?;
        }
    }

    Ok(())
}

async fn handle_start(bot: Bot, state: Arc<BotState>, dialogue: BotDialogue, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;
    let user = ensure_user(&state, &msg).await?;

    bot.send_message(chat_id, commands_help(&user)).await?;
    dialogue.update(DialogueState::Idle).await?;
    Ok(())
}

async fn handle_help(bot: Bot, state: Arc<BotState>, dialogue: BotDialogue, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;
    let user = ensure_user(&state, &msg).await?;
    bot.send_message(chat_id, commands_help(&user)).await?;
    dialogue.update(DialogueState::Idle).await?;
    Ok(())
}

async fn handle_calc(bot: Bot, state: Arc<BotState>, dialogue: BotDialogue, msg: Message) -> HandlerResult {
    let chat_id = msg.chat.id;
    let user = ensure_user(&state, &msg).await?;
    require_role!(user, Role::Business, bot, chat_id);

    let text = "Доступные расчеты:\n\
BaseCalc — симуляции случайных значений с параметром iterations.\n\
FullCalc — симуляции по переданному массиву чисел и iterations.\n\
InProgress — показать запущенные расчеты.";

    bot.send_message(chat_id, text)
        .reply_markup(calc_menu_keyboard())
        .await?;

    dialogue.update(DialogueState::Idle).await?;
    Ok(())
}

async fn handle_users_calc(
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
        bot.send_message(chat_id, "Нет запущенных расчетов у пользователей").await?;
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
        bot.send_message(chat_id, "Нет запущенных расчетов у пользователей").await?;
    } else {
        bot.send_message(chat_id, format!("Активные расчеты:\n{}", rows.join("\n"))).await?;
    }

    dialogue.update(DialogueState::Idle).await?;
    Ok(())
}

async fn ensure_user(state: &Arc<BotState>, msg: &Message) -> Result<User, BotError> {
    let user_id = msg.from.as_ref().map(|u| u.id.0).unwrap_or(msg.chat.id.0);
    let user_name = msg
        .from
        .as_ref()
        .and_then(|u| u.username.clone())
        .or_else(|| msg.chat.username().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    let mut conn = state.redis_client.get_async_connection().await?;

    if let Some(existing) = User::load(&mut conn, user_id).await? {
        return Ok(existing);
    }

    let user = User {
        user_id,
        user_name,
        user_groups: Role::Guest,
    };
    user.save(&mut conn).await?;
    Ok(user)
}

async fn ensure_user_by_id(
    state: &Arc<BotState>,
    user_id: i64,
    username: Option<String>,
) -> Result<User, BotError> {
    let mut conn = state.redis_client.get_async_connection().await?;
    if let Some(existing) = User::load(&mut conn, user_id).await? {
        return Ok(existing);
    }
    let user = User {
        user_id,
        user_name: username.unwrap_or_else(|| "unknown".to_string()),
        user_groups: Role::Guest,
    };
    user.save(&mut conn).await?;
    Ok(user)
}

async fn has_too_many_calcs(state: &Arc<BotState>, user_id: i64) -> Result<bool, BotError> {
    let mut conn = state.redis_client.get_async_connection().await?;
    let count = UsersCalcs::load(&mut conn, user_id)
        .await?
        .map(|c| c.calcs.len())
        .unwrap_or_default();
    Ok(count >= state.config.max_active_calcs)
}

async fn show_in_progress(bot: &Bot, state: &Arc<BotState>, user: &User, chat_id: ChatId) -> HandlerResult {
    let mut conn = state.redis_client.get_async_connection().await?;
    let Some(records) = UsersCalcs::load(&mut conn, user.user_id).await? else {
        bot.send_message(chat_id, "Нет запущенных расчетов").await?;
        return Ok(());
    };

    if records.calcs.is_empty() {
        bot.send_message(chat_id, "Нет запущенных расчетов").await?;
        return Ok(());
    }

    let mut lines = Vec::new();
    let mut buttons = Vec::new();

    for calc_id in records.calcs.iter() {
        match get_status(&state, *calc_id).await {
            Ok(status) => {
                lines.push(format!(
                    "ID: {} | Прогресс: {}% | Длительность: {} сек",
                    calc_id, status.progress, status.duration
                ));
                buttons.push(vec![InlineKeyboardButton::callback(
                    format!("Статус {}", calc_id),
                    format!("status:{}", calc_id),
                )]);
            }
            Err(_) => {
                lines.push(format!("ID: {} | статус недоступен", calc_id));
            }
        }
    }

    bot.send_message(chat_id, lines.join("\n"))
        .reply_markup(InlineKeyboardMarkup::new(buttons))
        .await?;
    Ok(())
}

async fn send_status(bot: &Bot, state: &Arc<BotState>, chat_id: ChatId, calc_id: Uuid) -> HandlerResult {
    match get_status(state, calc_id).await {
        Ok(status) => {
            let mut markup = status_markup(calc_id, status.progress == 100);
            let text = format!(
                "Статус {}\nПрогресс: {}%\nДлительность: {} сек",
                calc_id, status.progress, status.duration
            );

            bot.send_message(chat_id, text).reply_markup(markup).await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("Не удалось получить статус: {}", e)).await?;
        }
    }
    Ok(())
}

async fn send_result(bot: &Bot, state: &Arc<BotState>, chat_id: ChatId, calc_id: Uuid) -> HandlerResult {
    match get_result(state, calc_id).await {
        Ok(result) => {
            let result_json = result
                .result
                .map(|v| serde_json::to_string_pretty(&v))
                .transpose()?
                .unwrap_or_else(|| "Результат отсутствует".to_string());
            let text = format!(
                "Расчет {}\nСтарт: {}\nЗавершен: {}\nПараметры: {}\nРезультат:\n{}",
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
            UsersCalcs::remove_calc(state, chat_id.0, calc_id).await?;
        }
        Err(e) => {
            bot.send_message(chat_id, format!("Не удалось получить результат: {}", e)).await?;
        }
    }
    Ok(())
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

async fn get_status(state: &Arc<BotState>, calc_id: Uuid) -> Result<CalcStatusResponse, BotError> {
    let url = format!("{}/api/calc/{}", state.config.calc_runner_base, calc_id);
    let resp = state.http_client.post(url).send().await?;
    if resp.status() == StatusCode::NOT_FOUND {
        return Err(BotError::Parse("Расчет не найден".into()));
    }
    let body: CalcStatusResponse = resp.json().await?;
    Ok(body)
}

async fn get_result(state: &Arc<BotState>, calc_id: Uuid) -> Result<CalcResultResponse, BotError> {
    let url = format!("{}/api/calc/result/{}", state.config.calc_runner_base, calc_id);
    let resp = state.http_client.post(url).send().await?;
    if resp.status() == StatusCode::NOT_FOUND {
        return Err(BotError::Parse("Расчет не найден".into()));
    }
    let body: CalcResultResponse = resp.json().await?;
    Ok(body)
}

fn calc_menu_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![InlineKeyboardButton::callback("BaseCalc", "calc:base")],
        vec![InlineKeyboardButton::callback("FullCalc", "calc:full")],
        vec![InlineKeyboardButton::callback("InProgress", "calc:progress")],
        vec![InlineKeyboardButton::callback("Отмена", "calc:cancel")],
    ])
}

fn status_markup(calc_id: Uuid, finished: bool) -> InlineKeyboardMarkup {
    let mut rows = vec![vec![InlineKeyboardButton::callback(
        "Status",
        format!("status:{}", calc_id),
    )]];
    if finished {
        rows.push(vec![InlineKeyboardButton::callback(
            "Result",
            format!("result:{}", calc_id),
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

fn commands_help(user: &User) -> String {
    let mut commands = vec!["/start - регистрация/проверка".to_string(), "/help - помощь".to_string()];
    if user.user_groups.is_granted(Role::Business) {
        commands.push("/calc - запустить расчет".to_string());
    }
    if user.user_groups.is_granted(Role::Admin) {
        commands.push("/users_calc - активные расчеты всех пользователей".to_string());
    }

    format!(
        "Привет, {}!\nВаша роль: {:?}\nКоманды:\n{}",
        user.user_name,
        user.user_groups,
        commands.join("\n")
    )
}

async fn run_result_watcher(bot: Bot, state: Arc<BotState>) {
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
                        UsersCalcs::remove_calc(&state, user_id, calc_id).await?;
                        *last_send = Some(Instant::now());
                    }
                }
            }
        }
    }

    Ok(())
}

async fn fetch_calc_info(
    conn: &mut impl AsyncCommands,
    calc_id: Uuid,
) -> Result<Option<CalcInfo>, BotError> {
    let key = format!("{}{}", REDIS_CALC_PREFIX, calc_id);
    let value: Option<String> = conn.get(&key).await?;
    match value {
        Some(v) => Ok(Some(serde_json::from_str(&v)?)),
        None => Ok(None),
    }
}

fn format_result_message(info: &CalcInfo) -> String {
    let result_json = info
        .result
        .as_ref()
        .map(|v| serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string()))
        .unwrap_or_else(|| "результат отсутствует".to_string());

    format!(
        "Расчет {} завершен.\nСтарт: {}\nЗавершен: {}\nПрогресс: {}%\nРезультат:\n{}",
        info.calc_id,
        info.run_dt,
        info.end_dt
            .map(|dt| dt.to_string())
            .unwrap_or_else(|| "в процессе".to_string()),
        info.progress,
        result_json
    )
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
struct CalcStatusResponse {
    run_dt: DateTime<Utc>,
    progress: u32,
    duration: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CalcResultResponse {
    calc_id: Uuid,
    run_dt: DateTime<Utc>,
    end_dt: Option<DateTime<Utc>>,
    params: Option<serde_json::Value>,
    progress: u32,
    result: Option<serde_json::Value>,
    duration: Option<i64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum Role {
    Admin = 1,
    Business = 2,
    Guest = 3,
}

impl Role {
    fn priority(&self) -> u8 {
        match self {
            Role::Admin => 1,
            Role::Business => 2,
            Role::Guest => 3,
        }
    }

    pub fn is_granted(&self, required: Role) -> bool {
        self.priority() <= required.priority()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: i64,
    pub user_name: String,
    pub user_groups: Role,
}

impl User {
    async fn load(conn: &mut impl AsyncCommands, user_id: i64) -> Result<Option<Self>, BotError> {
        let key = format!("{}{}", REDIS_USER_PREFIX, user_id);
        let value: Option<String> = conn.get(&key).await?;
        match value {
            Some(v) => Ok(Some(serde_json::from_str(&v)?)),
            None => Ok(None),
        }
    }

    async fn save(&self, conn: &mut impl AsyncCommands) -> Result<(), BotError> {
        let key = format!("{}{}", REDIS_USER_PREFIX, self.user_id);
        let json = serde_json::to_string(self)?;
        conn.set(key, json).await?;
        Ok(())
    }

    async fn delete(conn: &mut impl AsyncCommands, user_id: i64) -> Result<(), BotError> {
        let key = format!("{}{}", REDIS_USER_PREFIX, user_id);
        conn.del(key).await?;
        Ok(())
    }

    async fn set_role(
        conn: &mut impl AsyncCommands,
        user_id: i64,
        role: Role,
    ) -> Result<(), BotError> {
        if let Some(mut user) = User::load(conn, user_id).await? {
            user.user_groups = role;
            user.save(conn).await?;
        }
        Ok(())
    }

    pub fn is_granted(&self, required: Role) -> bool {
        self.user_groups.is_granted(required)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersCalcs {
    pub user_id: i64,
    pub calcs: HashSet<Uuid>,
}

impl UsersCalcs {
    async fn load(
        conn: &mut impl AsyncCommands,
        user_id: i64,
    ) -> Result<Option<Self>, BotError> {
        let key = format!("{}{}", REDIS_USER_CALCS_PREFIX, user_id);
        let value: Option<String> = conn.get(&key).await?;
        match value {
            Some(v) => Ok(Some(serde_json::from_str(&v)?)),
            None => Ok(None),
        }
    }

    async fn save(&self, conn: &mut impl AsyncCommands) -> Result<(), BotError> {
        let key = format!("{}{}", REDIS_USER_CALCS_PREFIX, self.user_id);
        let json = serde_json::to_string(self)?;
        conn.set(key, json).await?;
        Ok(())
    }

    async fn add_calc(state: &Arc<BotState>, user_id: i64, calc_id: Uuid) -> Result<(), BotError> {
        let mut conn = state.redis_client.get_async_connection().await?;
        let mut record = UsersCalcs::load(&mut conn, user_id)
            .await?
            .unwrap_or_else(|| UsersCalcs {
                user_id,
                calcs: HashSet::new(),
            });
        record.calcs.insert(calc_id);
        record.save(&mut conn).await?;
        Ok(())
    }

    async fn remove_calc(state: &Arc<BotState>, user_id: i64, calc_id: Uuid) -> Result<(), BotError> {
        let mut conn = state.redis_client.get_async_connection().await?;
        if let Some(mut record) = UsersCalcs::load(&mut conn, user_id).await? {
            record.calcs.remove(&calc_id);
            if record.calcs.is_empty() {
                let key = format!("{}{}", REDIS_USER_CALCS_PREFIX, user_id);
                conn.del(key).await?;
            } else {
                record.save(&mut conn).await?;
            }
        }
        Ok(())
    }

    async fn list_tracked_users(conn: &mut impl AsyncCommands) -> Result<Vec<i64>, BotError> {
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(format!("{}*", REDIS_USER_CALCS_PREFIX))
            .query_async(conn)
            .await?;
        let ids = keys
            .into_iter()
            .filter_map(|k| k.strip_prefix(REDIS_USER_CALCS_PREFIX))
            .filter_map(|id| id.parse::<i64>().ok())
            .collect();
        Ok(ids)
    }

    fn pending_calcs(&self) -> Vec<Uuid> {
        self.calcs.iter().copied().collect()
    }
}

#[macro_export]
macro_rules! require_role {
    ($user:expr, $role:expr, $bot:expr, $chat_id:expr) => {{
        if !$user.user_groups.is_granted($role) {
            $bot
                .send_message($chat_id, format!("Недостаточно прав. Требуется {:?}.", $role))
                .await?;
            return Ok(());
        }
    }};
}

const INITIAL_USERS: &[(i64, &str, Role)] = &[
    (1, "admin", Role::Admin),
    (2, "business", Role::Business),
    (3, "guest", Role::Guest),
];

async fn seed_users(redis_client: Arc<redis::Client>) -> Result<(), BotError> {
    let mut conn = redis_client.get_async_connection().await?;
    for (id, name, role) in INITIAL_USERS {
        if User::load(&mut conn, *id).await?.is_none() {
            let user = User {
                user_id: *id,
                user_name: name.to_string(),
                user_groups: *role,
            };
            user.save(&mut conn).await?;
        }
    }
    Ok(())
}
