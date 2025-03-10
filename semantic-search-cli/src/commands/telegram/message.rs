//! Module for handling messages.

use super::{ApiClient, BotConfig, BotResult, Database};
use doc_for::{doc, doc_impl};
use frankenstein::{
    client_reqwest::Bot, AsyncTelegramApi, ChatId, ChatType, FileUpload, Message, ReplyParameters, SendMessageParams, SendStickerParams, User
};
use log::info;
use semantic_search::Embedding;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    /// a secret paw-session for debugging - sends a sticker with the given id.
    Debug(String),
}

impl Command {
    fn description() -> String {
        format!(
            "{}\n/help - {}\n/search - {}\n/inline - {}\n/debug - {}",
            doc!(Command),
            doc!(Command, Help),
            doc!(Command, Search),
            doc!(Command, Inline),
            doc!(Command, Debug),
        )
    }

    fn parse(text: &str, username: &str) -> Option<Self> {
        let text = text.trim();
        let (command, arg) = text.split_once(' ').unwrap_or((text, ""));

        // Two possible command formats:
        // 1. /command <arg>
        // 2. /command@bot_username <arg>

        // Trim the leading slash
        let slash = command.starts_with('/');
        if !slash {
            return None;
        }
        let command = &command[1..];

        // Split out the mention and check if it's the bot
        let (command, mention) = command.split_once('@').unwrap_or((command, ""));
        if !mention.is_empty() && mention != username {
            return None;
        }

        // Lowercase and match the command
        let command = command.to_lowercase();
        match command.as_str() {
            "help" => Some(Self::Help),
            "search" => Some(Self::Search(arg.to_string())),
            "inline" => Some(Self::Inline),
            "debug" => Some(Self::Debug(arg.to_string())),
            _ => None,
        }
    }
}

/// Handles incoming messages.
pub async fn message_handler(
    bot: &Bot,
    me: &User,
    msg: Message,
    db: Arc<Mutex<Database>>,
    api: &ApiClient,
    config: &BotConfig,
) -> BotResult<()> {
    let Some(username) = &me.username else {
        log::error!("Bot username not found.");
        return Ok(());
    };
    let Some(text) = &msg.text else {
        // Ignore non-text messages.
        return answer_fallback(bot, &msg).await;
    };
    let Some(cmd) = Command::parse(text, username) else {
        // Cannot parse the command
        return answer_fallback(bot, &msg).await;
    };
    info!("Received valid command: `{text}`, parsed as: {cmd:?}");
    answer_command(bot, &msg, cmd, db, api, config).await?;
    Ok(())
}

/// Answers the command.
async fn answer_command(
    bot: &Bot,
    msg: &Message,
    cmd: Command,
    db: Arc<Mutex<Database>>,
    api: &ApiClient,
    config: &BotConfig,
) -> BotResult<()> {
    let chat_id = msg.chat.id;
    let result = match cmd {
        Command::Help => {
            Ok(Command::description())
        }
        Command::Search(query) => {
            answer_search(api, &query, db, config).await
        }
        Command::Inline => {
            Ok("🐾 Just mention me in any chat, followed by your query, and I'll pounce into action to fetch the purr-fect meme for you! 😼✨".to_string())
        }
        Command::Debug(arg) => {
            if arg.is_empty() {
                Ok("🐾 Paws and reflect! Please provide a sticker file id... 🐱".to_string())
            } else {
                // Send given sticker
                let sticker = FileUpload::String(arg);
                let send_params = SendStickerParams::builder()
                    .chat_id(chat_id)
                    .sticker(sticker)
                    .build();
                bot.send_sticker(&send_params).await?;
                Ok("🐾 Sticker sent! Hope it made your whiskers twitch! 😼".to_string())
            }
        }
    };
    let message = match result {
        Ok(reply) => reply,
        Err(error) => {
            format!("😿 Oops! Something went wrong...\n{error}")
        }
    };

    reply(bot, msg, chat_id.into(), &message).await
}

/// Answers the search command.
async fn answer_search(
    api: &ApiClient,
    query: &str,
    db: Arc<Mutex<Database>>,
    config: &BotConfig,
) -> Result<String, String> {
    if query.is_empty() {
        return Ok("😾 Please prrr-ovide a query...".to_string());
    }
    let Ok(raw_embedding) = api.embed(query).await else {
        return Err("Failed to embed the query".to_string());
    };
    let embedding: Embedding = raw_embedding.into();
    let results = {
        let mut db = db.lock().await;
        db.search_with_id(config.num_results, &embedding).await
    };
    let Ok(results) = results else {
        return Err("Failed to search the database".to_string());
    };
    if results.is_empty() {
        return Ok("😿 No results found...".to_string());
    }
    // Format the results
    let message = results
        .iter()
        .map(|(path, similarity, file_id)| {
            let percent = similarity * 100.0;
            format!("🐾 {percent:.2}: {path} | /debug {file_id}")
        })
        .collect::<Vec<String>>()
        .join("\n");
    Ok(message)
}

/// Fallback message.
async fn answer_fallback(bot: &Bot, msg: &Message) -> BotResult<()> {
    // Only answer fallback if the message is a private message.
    if !matches!(msg.chat.type_field, ChatType::Private) {
        return Ok(());
    }
    // Choose a pseudo-random message from the fallback messages.
    let idx = msg.message_id.unsigned_abs() as usize % FALLBACK_MESSAGES.len();
    let message = FALLBACK_MESSAGES[idx];

    reply(bot, msg, msg.chat.id.into(), message).await
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
