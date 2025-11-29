use std::sync::Arc;

use teloxide::{prelude::*, types::Message};

use crate::{
    auth::ensure_user,
    commands::{commands_help, BotDialogue, DialogueState},
    errors::HandlerResult,
    settings::BotState,
};

pub async fn help_handle(
    bot: Bot,
    state: Arc<BotState>,
    dialogue: BotDialogue,
    msg: Message,
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let user = ensure_user(&state, &msg).await?;
    bot.send_message(chat_id, commands_help(&user)).await?;
    dialogue.update(DialogueState::Idle).await?;
    Ok(())
}
