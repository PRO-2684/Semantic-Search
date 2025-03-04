//! Module for handling messages.

use super::ThrottledBot;
use teloxide::{
    dispatching::{HandlerExt, UpdateFilterExt, UpdateHandler},
    dptree,
    prelude::{Requester, ResponseResult},
    types::{Me, Message, Update},
    utils::command::BotCommands,
};

const FALLBACK_MESSAGES: [&str; 5] = [
    "😹 Maow?",
    "😼 Meowww!",
    "🙀 Nyaaa!",
    "😿 Mew...",
    "😾 Prrrrr...!",
];

/// Handles incoming messages.
pub async fn message_handler(bot: ThrottledBot, msg: Message, me: Me)  -> ResponseResult<()> {
    let Some(username) = &me.username else {
        log::error!("Bot username not found.");
        return Ok(());
    };
    let Some(text) = msg.text() else {
        // Ignore non-text messages.
        answer_fallback(bot, msg).await?;
        return Ok(());
    };
    let Ok(cmd) = Command::parse(text, &username) else {
        // Cannot parse the command
        answer_fallback(bot, msg).await?;
        return Ok(());
    };
    answer_command(bot, msg, cmd).await?;
    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "😼 Purr-fectly supported commands, just for your whiskers 🐾:"
)]
pub enum Command {
    #[command(description = "paw-some help text, just for curious kitties.")]
    Help,
    #[command(description = "sniff out the purr-fect meme.")]
    Search(String),
}

async fn answer_command(bot: ThrottledBot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Search(query) => {
            // TODO: Implement the search command.
            bot.send_message(msg.chat.id, format!("😸 Searching for: {query}"))
                .await?
        }
    };

    Ok(())
}

async fn answer_fallback(bot: ThrottledBot, msg: Message) -> ResponseResult<()> {
    // Choose a pseudo-random message from the fallback messages.
    let idx = msg.id.0.abs() as usize % FALLBACK_MESSAGES.len();
    bot.send_message(msg.chat.id, FALLBACK_MESSAGES[idx]).await?;

    Ok(())
}
