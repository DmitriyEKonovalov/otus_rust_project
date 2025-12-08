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

const NOT_FOUND_MESSAGE: &str = "Расчет не найден.";
const OTHER_ERROR_MESSAGE: &str = "Не удалось получить статус. Попробуйте позже.";

pub async fn get_calc_status(
    bot: Bot,
    msg: Message,
    calc_id: Uuid,
    state: Arc<BotState>,
) -> HandlerResult {
    info!(
        user_id = msg.from().map(|u| u.id.0),
        chat_id = msg.chat.id.0,
        %calc_id,
        "command get_calc_status invoked"
    );
    match calc_runner::get_calc_status(&state.http_client, &state.config, calc_id).await {
        Ok(status) => {
            let text = format!(
                "Расчет {} статус:\nПрогресс: {}%\nДлительность: {}s\nНачат: {}",
                status.calc_id,
                status.progress,
                status.duration,
                status.run_dt.to_rfc3339(),
            );
            bot.send_message(msg.chat.id, text).await?;
            Ok(())
        }
        Err(BotError::Http(err)) => {
            if let Some(StatusCode::NOT_FOUND) = err.status() {
                bot.send_message(msg.chat.id, NOT_FOUND_MESSAGE).await?;
            } else {
                bot.send_message(msg.chat.id, OTHER_ERROR_MESSAGE).await?;
            }
            Ok(())
        }
        Err(_) => {
            bot.send_message(msg.chat.id, OTHER_ERROR_MESSAGE).await?;
            Ok(())
        }
    }
}
