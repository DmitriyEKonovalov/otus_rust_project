use std::sync::Arc;

use teloxide::{
    prelude::*,
    types::CallbackQuery,
    utils::command::BotCommands,
};
use uuid::Uuid;

use crate::{exceptions::HandlerResult, settings::BotState};

pub mod get_active_calcs;
pub mod get_calc_result;
pub mod get_calc_status;
pub mod get_user_calcs;
pub mod help;
pub mod run_base_calc;
pub mod run_mass_calc;
pub mod start;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "snake_case", description = "Available commands")]
pub enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Show help")]
    Help,
    #[command(description = "Run base calculation: /base_calc <iterations>")]
    BaseCalc { iterations: u32 },
    #[command(parse_with = "split")]
    #[command(description = "Run mass calculation: /mass_calc <iterations> <comma separated numbers>")]
    MassCalc { iterations: u32, data: String },
    #[command(description = "Get calculation status by id")]
    CalcStatus { calc_id: Uuid },
    #[command(description = "Get calculation result by id")]
    CalcResult { calc_id: Uuid },
    #[command(description = "List your calculations")]
    UsersCalc,
    #[command(description = "Show active calculations (admin only)")]
    ActiveCalcs,
    // #[command(parse_with = "split")]
    #[command(description = "тестовая команда отправить сообщение")]
    Send { text: String,},

}

pub async fn dispatch_command(
    bot: Bot,
    msg: Message,
    state: Arc<BotState>,
    command: Command,
) -> HandlerResult {
    match command {
        Command::Start => start::start(bot, msg, state).await,
        Command::Help => help::help(bot, msg, state).await,
        Command::BaseCalc { iterations } => {
            run_base_calc::run_base_calc_command(bot, msg, iterations, state).await
        }
        Command::MassCalc { iterations, data } => {
            run_mass_calc::run_mass_calc_command(bot, msg, iterations, data, state).await
        }
        Command::CalcStatus { calc_id } => {
            get_calc_status::get_calc_status(bot, msg, calc_id, state).await
        }
        Command::CalcResult { calc_id } => {
            get_calc_result::get_calc_result(bot, msg, calc_id, state).await
        }
        Command::UsersCalc => get_user_calcs::get_user_calcs(bot, state, msg).await,
        Command::ActiveCalcs => get_active_calcs::get_active_calcs(bot, state, msg).await,
        Command::Send { text } => {
            let response: String = format!("📨 Сообщение:\n{}", text);
            bot.send_message(msg.chat.id, response).await?;
            Ok(())
        },

    }
}

pub async fn dispatch_callback(
    bot: Bot,
    query: CallbackQuery,
    state: Arc<BotState>,
) -> HandlerResult {
    let action = query
        .data
        .as_deref()
        .and_then(|data| data.split_once(':'))
        .and_then(|(kind, raw_id)| Uuid::parse_str(raw_id).ok().map(|id| (kind, id)))
        .and_then(|(kind, calc_id)| query.message.clone().map(|msg| (kind, calc_id, msg)));

    let (action_result, is_valid_action) = if let Some((kind, calc_id, message)) = action {
        let result = match kind {
            "calc_status" => {
                get_calc_status::get_calc_status(bot.clone(), message, calc_id, state.clone())
                    .await
            }
            "calc_result" => {
                get_calc_result::get_calc_result(bot.clone(), message, calc_id, state.clone())
                    .await
            }
            _ => Ok(()),
        };
        (result, true)
    } else {
        (Ok(()), false)
    };

    if is_valid_action {
        bot.answer_callback_query(query.id).await?;
    } else {
        bot.answer_callback_query(query.id)
            .text("Unknown action")
            .await?;
    }

    action_result
}
