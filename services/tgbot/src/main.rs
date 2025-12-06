mod commands;
mod exceptions;
mod init;
mod models;
mod permissions;
mod settings;

use std::{env, sync::Arc};

use dotenvy::dotenv;
use teloxide::{dispatching::UpdateFilterExt, dptree, prelude::*, utils::command::BotCommands};
use tracing_subscriber::EnvFilter;

use crate::{
    commands::{dispatch_command, Command},
    exceptions::BotError,
    settings::{BotConfig, BotState},
};

#[tokio::main]
async fn main() -> Result<(), BotError> {
    dotenv().ok();
    // allow running from workspace root or crate dir
    let _ = dotenvy::from_filename("services/tgbot/.env");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let bot_token = env::var("BOT_ID")
        .or_else(|_| env::var("TELEGRAM_BOT_TOKEN"))
        .map_err(|_| BotError::Config("BOT_ID env var is required".into()))?;
    let bot = Bot::new(bot_token);

    let config = BotConfig::from_env();
    let http_client = reqwest::Client::builder()
        .build()
        .map_err(|e| BotError::Config(format!("Failed to build HTTP client: {e}")))?;

    let state = Arc::new(BotState {
        http_client,
        config,
    });

    bot.set_my_commands(Command::bot_commands()).await?;
    
    let message_handler = Update::filter_message()
        .filter_command::<Command>()
        .endpoint(dispatch_command);

    let callback_handler = Update::filter_callback_query().endpoint(commands::dispatch_callback);

    let handler = dptree::entry()
        .branch(message_handler)
        .branch(callback_handler);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
