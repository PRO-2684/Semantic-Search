//! Module for handling messages.

use super::{ApiClient, Database};
use doc_for::{doc, doc_impl};
use frankenstein::{
    client_reqwest::Bot, AsyncTelegramApi, ChatId, Error, Message, ReplyParameters,
    SendMessageParams, User,
};
use log::info;
use semantic_search::Embedding;
use std::sync::Arc;
use tokio::sync::Mutex;

type BotResult<T> = Result<T, Error>;

const FALLBACK_MESSAGES: [&str; 5] = [
    "😹 Maow?",
    "😼 Meowww!",
    "🙀 Nyaaa!",
    "😿 Mew...",
    "😾 Prrrrr...!",
];

#[derive(Clone, Debug)]
#[doc_impl(strip = 1)]
/// 😼 Purr-fectly supported commands, just for your whiskers 🐾:
pub enum Command {
    /// paw-some help text, just for curious kitties.
    Help,
    /// sniff out the purr-fect meme.
    Search(String),
    /// learn how to summon this kitty anywhere with a flick of your paw.
    Inline,
}

impl Command {
    fn description() -> String {
        format!(
            "{}\n{}\n{}\n{}",
            doc!(Command),
            doc!(Command, Help),
            doc!(Command, Search),
            doc!(Command, Inline)
        )
    }

    fn parse(text: &str, username: &str) -> Option<Self> {
        let text = text.trim();
        let (command, arg) = text.split_once(' ')?;
        let command = command.to_lowercase();

        // Two possible command formats:
        // 1. /command <arg>
        // 2. /command@bot_username <arg>

        // Trim the command prefix
        let slash = command.starts_with('/');
        if !slash {
            return None;
        }
        let command = &command[1..];

        // Trim the bot username if present
        let at = format!("@{username}");
        let at_bot = command.ends_with(&at);
        let command = if at_bot {
            &command[..command.len() - at.len()]
        } else {
            command
        };

        match command {
            "help" => Some(Self::Help),
            "search" => Some(Self::Search(arg.to_string())),
            "inline" => Some(Self::Inline),
            _ => None,
        }
    }
}

/// Handles incoming messages.
pub async fn message_handler(
    bot: &Bot,
    me: &User,
    msg: Message,
    db: &mut Database,
    api: &ApiClient,
) -> BotResult<()> {
    let Some(username) = &me.username else {
        log::error!("Bot username not found.");
        return Ok(());
    };
    let Some(text) = &msg.text else {
        // Ignore non-text messages.
        answer_fallback(&bot, &msg).await?;
        return Ok(());
    };
    let Some(cmd) = Command::parse(text, &username) else {
        // Cannot parse the command
        answer_fallback(&bot, &msg).await?;
        return Ok(());
    };
    info!("Received valid command: `{text}`, parsed as: {cmd:?}");
    answer_command(&bot, &msg, cmd, db, api).await?;
    Ok(())
}

/// Answers the command.
async fn answer_command(
    bot: &Bot,
    msg: &Message,
    cmd: Command,
    db: &mut Database,
    api: &ApiClient,
) -> BotResult<()> {
    let chat_id = msg.chat.id;
    let result = match cmd {
        Command::Help => {
            Ok(Command::description().to_string())
        }
        Command::Search(query) => {
            answer_search(&bot, chat_id.into(), api, &query, db).await
        }
        Command::Inline => {
            Ok("🐾 Just mention me in any chat, followed by your query, and I'll pounce into action to fetch the purr-fect meme for you! 😼✨".to_string())
        }
    };
    let message = match result {
        Ok(reply) => reply,
        Err(error) => {
            format!("😿 Oops! Something went wrong...\n{error}")
        }
    };

    reply(&bot, &msg, chat_id.into(), &message).await
}

/// Answers the search command.
async fn answer_search(
    bot: &Bot,
    chat_id: ChatId,
    api: &ApiClient,
    query: &str,
    db: &mut Database,
) -> Result<String, String> {
    if query.is_empty() {
        return Ok("😾 Please prrr-ovide a query...".to_string());
    }
    let Ok(raw_embedding) = api.embed(query).await else {
        return Err("Failed to embed the query".to_string());
    };
    let embedding: Embedding = raw_embedding.into();
    let results = db.search(5, &embedding);
    let Ok(results) = results.await else {
        return Err("Failed to search the database".to_string());
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
async fn answer_fallback(bot: &Bot, msg: &Message) -> BotResult<()> {
    // Choose a pseudo-random message from the fallback messages.
    let idx = msg.message_id.abs() as usize % FALLBACK_MESSAGES.len();
    let message = FALLBACK_MESSAGES[idx];

    reply(&bot, &msg, msg.chat.id.into(), message).await
}

/// Reply to the message.
async fn reply(bot: &Bot, msg: &Message, chat_id: ChatId, text: &str) -> BotResult<()> {
    let reply_params = ReplyParameters::builder()
        .message_id(msg.message_id)
        .build();
    let send_params = SendMessageParams::builder()
        .chat_id(chat_id)
        .text(text)
        .reply_parameters(reply_params)
        .build();
    bot.send_message(&send_params).await?;
    Ok(())
}
