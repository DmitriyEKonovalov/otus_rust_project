use std::sync::Arc;

use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Message},
};
use uuid::Uuid;

use crate::exceptions::HandlerResult;
use crate::models::calc_runner::{self, MassCalcParams};
use crate::settings::{BotState, LIMIT_EXCEEDED_MESSAGE, MAX_CALC_FOR_USER};

const INVALID_DATA_MESSAGE: &str =
    "Provide numbers separated by commas after iterations, e.g. /mass_calc 3 1,2,3.";
const STARTED_MESSAGE: &str = "Started mass calculation.";

pub async fn run_mass_calc_command(
    bot: Bot,
    msg: Message,
    iterations: u32,
    raw_data: String,
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

    let data = match parse_numbers(&raw_data) {
        Some(parsed) if !parsed.is_empty() => parsed,
        _ => {
            bot.send_message(msg.chat.id, INVALID_DATA_MESSAGE).await?;
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

    let params = MassCalcParams {
        user_id: user.id.0 as i64,
        data,
        iterations,
    };
    let response =
        calc_runner::run_mass_calc(&state.http_client, &state.config, &params).await?;
    let calc_id = response.calc_id;

    bot.send_message(
        msg.chat.id,
        format!(
            "{} ID: {} (iterations: {}, data size: {})",
            STARTED_MESSAGE,
            calc_id,
            iterations,
            params.data.len()
        ),
    )
    .reply_markup(calc_actions_keyboard(calc_id))
    .await?;

    Ok(())
}

fn parse_numbers(raw: &str) -> Option<Vec<u32>> {
    let mut numbers = Vec::new();
    for token in raw.split(|c| c == ',' || c == ';' || c == ' ') {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        if let Ok(value) = token.parse::<u32>() {
            numbers.push(value);
        } else {
            return None;
        }
    }
    Some(numbers)
}

fn calc_actions_keyboard(calc_id: Uuid) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("Status", format!("calc_status:{calc_id}")),
        InlineKeyboardButton::callback("Result", format!("calc_result:{calc_id}")),
    ]])
}
