use std::sync::Arc;

use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
};
use uuid::Uuid;

use crate::exceptions::HandlerResult;
use crate::models::calc_runner::{self, BaseCalcParams};
use crate::settings::{BotState, LIMIT_EXCEEDED_MESSAGE, MAX_CALC_FOR_USER};

const STARTED_MESSAGE: &str = "Started base calculation.";

pub async fn run_base_calc_command(
    bot: Bot,
    msg: Message,
    iterations: u32,
    state: Arc<BotState>,
) -> HandlerResult {
    if iterations == 0 {
        bot.send_message(msg.chat.id, "Iterations must be greater than zero.")
            .await?;
        return Ok(());
    }

    let user = match msg.from() {
        Some(u) => u.clone(),
        None => {
            bot.send_message(msg.chat.id, "Cannot identify user.").await?;
            return Ok(());
        }
    };

    let active_calcs = calc_runner::get_user_calcs(
        &state.http_client,
        &state.config,
        user.id.0 as i64,
    )
    .await?;
    if active_calcs.calcs.len() >= MAX_CALC_FOR_USER {
        bot.send_message(msg.chat.id, LIMIT_EXCEEDED_MESSAGE).await?;
        return Ok(());
    }

    let params = BaseCalcParams {
        user_id: user.id.0 as i64,
        iterations,
    };
    let response = calc_runner::run_base_calc(&state.http_client, &state.config, &params).await?;
    let calc_id = response.calc_id;

    bot.send_message(
        msg.chat.id,
        format!(
            "{} ID: {} (iterations: {})",
            STARTED_MESSAGE, calc_id, iterations
        ),
    )
    .reply_markup(calc_actions_keyboard(calc_id))
    .await?;

    Ok(())
}

fn calc_actions_keyboard(calc_id: Uuid) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("Status", format!("calc_status:{calc_id}")),
        InlineKeyboardButton::callback("Result", format!("calc_result:{calc_id}")),
    ]])
}
