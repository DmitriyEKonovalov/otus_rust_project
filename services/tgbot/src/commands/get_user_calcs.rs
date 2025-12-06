use std::sync::Arc;

use teloxide::{prelude::*, types::Message};

use crate::{
    exceptions::HandlerResult,
    models::calc_runner,
    settings::BotState,
};

const NO_USERS_CALCS_MESSAGE: &str = "Нет активный расчетов.";
const USERS_CALCS_HEADER: &str = "Список активных расчетов:\n";

pub async fn get_user_calcs(bot: Bot, state: Arc<BotState>, msg: Message) -> HandlerResult {
    let user = match msg.from() {
        Some(u) => u.clone(),
        None => {
            bot.send_message(msg.chat.id, NO_USERS_CALCS_MESSAGE).await?;
            return Ok(());
        }
    };

    let response =
        calc_runner::get_user_calcs(&state.http_client, &state.config, user.id.0 as i64).await?;

    if response.calcs.is_empty() {
        bot.send_message(msg.chat.id, NO_USERS_CALCS_MESSAGE).await?;
    } else {
        let mut lines = Vec::new();
        for calc_id in response.calcs.iter() {
            lines.push(format!("- {}", calc_id));
        }
        bot.send_message(
            msg.chat.id,
            format!("{}{}", USERS_CALCS_HEADER, lines.join("\n")),
        )
        .await?;
    }

    Ok(())
}
