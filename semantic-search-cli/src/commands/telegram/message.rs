//! Module for handling messages.

use super::{ApiClient, Database, ThrottledBot};
use semantic_search::Embedding;
use std::sync::Arc;
use teloxide::{
    dispatching::{HandlerExt, UpdateFilterExt, UpdateHandler},
    dptree,
    payloads::SendMessageSetters,
    prelude::{Requester, ResponseResult},
    types::{ChatId, Me, Message, ReplyParameters, Update},
    utils::command::BotCommands,
};
use tokio::sync::Mutex;

const FALLBACK_MESSAGES: [&str; 5] = [
    "😹 Maow?",
    "😼 Meowww!",
    "🙀 Nyaaa!",
    "😿 Mew...",
    "😾 Prrrrr...!",
];

/// Handles incoming messages.
pub async fn message_handler(
    bot: ThrottledBot,
    msg: Message,
    me: Me,
    db: Arc<Mutex<Database>>,
    api: ApiClient,
) -> ResponseResult<()> {
    let Some(username) = &me.username else {
        log::error!("Bot username not found.");
        return Ok(());
    };
    let Some(text) = msg.text() else {
        // Ignore non-text messages.
        answer_fallback(&bot, msg).await?;
        return Ok(());
    };
    let Ok(cmd) = Command::parse(text, &username) else {
        // Cannot parse the command
        answer_fallback(&bot, msg).await?;
        return Ok(());
    };
    answer_command(bot, msg, cmd, db, api).await?;
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
    #[command(description = "learn how to summon this kitty anywhere with a flick of your paw.")]
    Inline,
}

/// Answers the command.
async fn answer_command(
    bot: ThrottledBot,
    msg: Message,
    cmd: Command,
    db: Arc<Mutex<Database>>,
    api: ApiClient,
) -> ResponseResult<()> {
    let chat_id = msg.chat.id;
    let message = match cmd {
        Command::Help => {
            Ok(bot.send_message(chat_id, Command::descriptions().to_string()))
        }
        Command::Search(query) => {
            answer_search(&bot, chat_id, api, &query, db).await.map(|reply| bot.send_message(chat_id, reply))
        }
        Command::Inline => {
            Ok(bot.send_message(chat_id, "🐾 Just mention me in any chat, followed by your query, and I'll pounce into action to fetch the purr-fect meme for you! 😼✨"))
        }
    };
    match message {
        Ok(message) => {
            message
                .reply_parameters(ReplyParameters::new(msg.id))
                .await?;
        }
        Err(error) => {
            answer_error(&bot, msg, error).await?;
        }
    }

    Ok(())
}

/// Answers the search command.
async fn answer_search(
    bot: &ThrottledBot,
    chat_id: ChatId,
    api: ApiClient,
    query: &str,
    db: Arc<Mutex<Database>>,
) -> Result<String, &'static str> {
    if query.is_empty() {
        return Ok("😾 Please prrr-ovide a query...".to_string());
    }
    let Ok(raw_embedding) = api.embed(query).await else {
        return Err("Failed to embed the query");
    };
    let embedding: Embedding = raw_embedding.into();
    let mut db = db.lock().await;
    let results = db.search(5, &embedding);
    let Ok(results) = results.await else {
        return Err("Failed to search the database");
    };
    if results.is_empty() {
        return Ok("😿 No results found...".to_string());
    }
    // Format the results
    let message = results
        .iter()
        .map(|(path, similarity)| {
            let percent = similarity * 100.0;
            format!("🐾 {percent:.2}: {path}")
        })
        .collect::<Vec<String>>()
        .join("\n");
    Ok(message)
}

/// Fallback message.
async fn answer_fallback(bot: &ThrottledBot, msg: Message) -> ResponseResult<()> {
    // Choose a pseudo-random message from the fallback messages.
    let idx = msg.id.0.abs() as usize % FALLBACK_MESSAGES.len();
    bot.send_message(msg.chat.id, FALLBACK_MESSAGES[idx])
        .await?;

    Ok(())
}

/// Error message.
async fn answer_error(bot: &ThrottledBot, msg: Message, error: &str) -> ResponseResult<()> {
    bot.send_message(
        msg.chat.id,
        format!("😿 Oops! Something went wrong...\n{error}"),
    )
    .await?;

    Ok(())
}
