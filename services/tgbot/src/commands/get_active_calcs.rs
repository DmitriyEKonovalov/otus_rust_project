use std::sync::Arc;

use teloxide::{prelude::*, types::Message};


use crate::{
    exceptions::HandlerResult,
    models::{calc_runner, Role},
    permissions::get_user_role,
    settings::BotState,
};
use tracing::info;

const NO_ACTIVE_CALCS_MESSAGE: &str = "Нет активный расчетов.";
const ACTIVE_CALCS_HEADER: &str = "Список активных расчетов:\n";
const ADMIN_ONLY_MESSAGE: &str = "Не достаточно прав на команду";

pub async fn get_active_calcs(bot: Bot, state: Arc<BotState>, msg: Message) -> HandlerResult {
    let user = match msg.from() {
        Some(u) => u.clone(),
        None => {
            bot.send_message(msg.chat.id, ADMIN_ONLY_MESSAGE).await?;
            return Ok(());
        }
    };
    info!(
        user_id = user.id.0,
        chat_id = msg.chat.id.0,
        "command get_active_calcs invoked"
    );

    let role = get_user_role(&user, &state.config.admin_user_ids);
    if !role.is_granted(Role::Admin) {
        bot.send_message(msg.chat.id, ADMIN_ONLY_MESSAGE).await?;
        return Ok(());
    }

    let response = calc_runner::get_active_calcs(&state.http_client, &state.config).await?;

    if response.calcs.is_empty() {
        bot.send_message(msg.chat.id, NO_ACTIVE_CALCS_MESSAGE).await?;
        return Ok(());
    }

    let mut lines = Vec::new();
    for calc in response.calcs.iter() {
        let progress = calc.progress;
        let run_dt = calc.run_dt.to_rfc3339();
        lines.push(format!(
            "- {} | user {} | started {} | progress {}%",
            calc.calc_id, calc.user_id, run_dt, progress
        ));
    }

    bot.send_message(
        msg.chat.id,
        format!("{}{}", ACTIVE_CALCS_HEADER, lines.join("\n")),
    )
    .await?;

    Ok(())
}
