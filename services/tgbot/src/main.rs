// mod commands;
// mod init;
// mod settings;

// use std::sync::Arc;

// use dotenvy::dotenv;
// use teloxide::{
//     dispatching::{dialogue::InMemStorage, UpdateFilterExt},
//     dptree,
//     prelude::*,
//     types::{CallbackQuery, Message},
// };

// use crate::commands::{
//     start, help, 
//     calc_handle, callback_handle, help_handle, message_handle, run_result_watcher, start_handle,
//     users_calc_handle, BotDialogue, Command, DialogueState,
// };
// use crate::errors::{BotError, HandlerResult};
// // use crate::init::create_users;
// use crate::settings::{BotConfig, BotState, DEFAULT_MAX_CALCS};

// #[tokio::main]
// async fn main() -> Result<(), BotError> {
//     dotenv().ok();

//     // let redis_url: String = std::env::var("REDIS_URL").unwrap_or_else(|_| {
//     //     let host = std::env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
//     //     let port = std::env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
//     //     let username = std::env::var("REDIS_USERNAME")
//     //         .or_else(|_| std::env::var("REDIS_USER"))
//     //         .ok()
//     //         .filter(|v| !v.is_empty());
//     //     let password = std::env::var("REDIS_PASSWORD").ok().filter(|v| !v.is_empty());

//     //     match (username, password) {
//     //         (Some(user), Some(pass)) => format!("redis://{}:{}@{}:{}/", user, pass, host, port),
//     //         (_, Some(pass)) => format!("redis://:{}@{}:{}/", pass, host, port),
//     //         _ => format!("redis://{}:{}/", host, port),
//     //     }
//     // });
//     // let redis_client: Arc<redis::Client> = Arc::new(redis::Client::open(redis_url)?);

//     // create_users(redis_client.clone()).await?;

//     // let state = Arc::new(BotState {
//     //     redis_client,
//     //     http_client: reqwest::Client::new(),
//     //     config: BotConfig {
//     //         calc_runner_base,
//     //         max_active_calcs: DEFAULT_MAX_CALCS,
//     //     },
//     // });
//     // let state_for_worker = Arc::clone(&state);

//     // поклюячение к api сервиса calc_runner
//     let calc_runner_base =
//         std::env::var("CALC_RUNNER_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());

//     let bot_token = std::env::var("BOT_TOKEN").expect("BOT_TOKEN is required in .env");
//     let bot: Bot = Bot::new(bot_token);
//     let bot_for_worker = bot.clone();
    
//     // фоновый процесс ожидания результатов расчетов 
//     tokio::spawn(async move {
//         run_result_watcher(bot_for_worker, state_for_worker).await;
//     });


//     Ok(())
// }

// async fn handle_command(
//     bot: Bot,
//     state: Arc<BotState>,
//     dialogue: BotDialogue,
//     msg: Message,
//     command: Command,
// ) -> HandlerResult {
//     match command {
//         Command::Start => start_handle(bot, state, dialogue, msg).await,
//         Command::Help => help_handle(bot, state, dialogue, msg).await,
//         Command::Calc => calc_handle(bot, state, dialogue, msg).await,
//         Command::UsersCalc => users_calc_handle(bot, state, dialogue, msg).await,
//     }
// }

// async fn handle_message(
//     bot: Bot,
//     state: Arc<BotState>,
//     dialogue: BotDialogue,
//     msg: Message,
// ) -> HandlerResult {
//     message_handle(bot, state, dialogue, msg).await
// }

// async fn handle_callback(
//     bot: Bot,
//     state: Arc<BotState>,
//     dialogue: BotDialogue,
//     q: CallbackQuery,
// ) -> HandlerResult {
//     callback_handle(bot, state, dialogue, q).await
// }

fn main () {

}