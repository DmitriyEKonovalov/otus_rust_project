use std::sync::Arc;

use reqwest::StatusCode;
use teloxide::{prelude::*, types::Message};
use uuid::Uuid;

use crate::{
    exceptions::{BotError, HandlerResult},
    models::calc_runner,
    settings::BotState,
};
use tracing::info;

const NOT_FOUND_MESSAGE: &str = "Calculation not found.";
const NOT_READY_MESSAGE: &str = "Calculation has not completed yet. Please try again later.";
const GENERIC_ERROR_MESSAGE: &str = "Failed to fetch result. Please try again later.";

pub async fn get_calc_result(
    bot: Bot,
    msg: Message,
    calc_id: Uuid,
    state: Arc<BotState>,
) -> HandlerResult {
    info!(
        user_id = msg.from().map(|u| u.id.0),
        chat_id = msg.chat.id.0,
        %calc_id,
        "command get_calc_result invoked"
    );
    match calc_runner::get_calc_result(&state.http_client, &state.config, calc_id).await {
        Ok(response) => {
            let params = response
                .params
                .as_ref()
                .map(to_pretty_json)
                .unwrap_or_else(|| "n/a".into());
            let result = response
                .result
                .as_ref()
                .map(to_pretty_json)
                .unwrap_or_else(|| "n/a".into());
            let duration = response
                .duration
                .map(|secs| format!("{secs}s"))
                .unwrap_or_else(|| "n/a".into());
            let end_dt = response
                .end_dt
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "in progress".into());

            let message = format!(
                "Calc {} result:\nProgress: {}%\nDuration: {}\nStarted: {}\nFinished: {}\nParams:\n{}\nResult:\n{}",
                response.calc_id,
                response.progress,
                duration,
                response.run_dt.to_rfc3339(),
                end_dt,
                params,
                result,
            );

            bot.send_message(msg.chat.id, message).await?;
            Ok(())
        }
        Err(BotError::Http(err)) => {
            if let Some(status) = err.status() {
                match status {
                    StatusCode::NOT_FOUND => {
                        bot.send_message(msg.chat.id, NOT_FOUND_MESSAGE).await?;
                    }
                    StatusCode::BAD_REQUEST => {
                        bot.send_message(msg.chat.id, NOT_READY_MESSAGE).await?;
                    }
                    _ => {
                        bot.send_message(msg.chat.id, GENERIC_ERROR_MESSAGE).await?;
                    }
                }
            } else {
                bot.send_message(msg.chat.id, GENERIC_ERROR_MESSAGE).await?;
            }
            Ok(())
        }
        Err(_) => {
            bot.send_message(msg.chat.id, GENERIC_ERROR_MESSAGE).await?;
            Ok(())
        }
    }
}

fn to_pretty_json(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}
