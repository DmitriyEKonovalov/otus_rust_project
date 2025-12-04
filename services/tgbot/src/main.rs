mod commands;
mod init;
mod settings;

use std::sync::Arc;

use dotenvy::dotenv;
use teloxide::{
    dispatching::{dialogue::InMemStorage, UpdateFilterExt},
    dptree,
    prelude::*,
    types::{CallbackQuery, Message},
};

use crate::commands::{
};
use crate::errors::{BotError, HandlerResult};
// use crate::init::create_users;
use crate::settings::{BotConfig, BotState, DEFAULT_MAX_CALCS};

#[tokio::main]
async fn main() -> Result<(), BotError> {
    dotenv().ok();

}

