//! Module for handling messages.

use super::{ApiClient, BotConfig, BotResult, Database, super::super::util::Record};
use doc_for::{doc, doc_impl};
use frankenstein::{
    AsyncTelegramApi, Error, ParseMode,
    client_reqwest::Bot,
    input_file::FileUpload,
    methods::{SendMessageParams, SendStickerParams, SetMyCommandsParams},
    stickers::StickerType,
    types::{BotCommand, ChatType, LinkPreviewOptions, Message, ReplyParameters, User},
};
use log::{error, info};
use semantic_search::Embedding;
use std::sync::Arc;
use tokio::sync::Mutex;

const FALLBACK_MESSAGES: [&str; 5] = [
    "😹 Maow?",
    "😼 Meowww :3",
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
    /// send a sticker by its file id.
    Sticker(String),
    /// add a sticker to database with given description. Only for bot owner.
    Add(String),
}

impl Command {
    fn description(config: &BotConfig) -> String {
        let content = format!(
            "{}\n/help - {}\n/search - {}\n/inline - {}\n/sticker - {}",
            doc!(Command),
            doc!(Command, Help),
            doc!(Command, Search),
            doc!(Command, Inline),
            doc!(Command, Sticker),
        );
        let postscript = config.postscript.trim();
        if postscript.is_empty() {
            content
        } else {
            format!("{content}\n{postscript}")
        }
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
            "sticker" => Some(Self::Sticker(arg.to_string())),
            "add" => Some(Self::Add(arg.to_string())),
            _ => None,
        }
    }
}

/// Set my commands.
pub async fn set_commands(bot: &Bot) -> BotResult<()> {
    let commands = [
        ("/help", doc!(Command, Help)),
        ("/search", doc!(Command, Search)),
        ("/inline", doc!(Command, Inline)),
        ("/sticker", doc!(Command, Sticker)),
        ("/add", doc!(Command, Sticker)),
    ];
    let commands: Vec<_> = commands
        .into_iter()
        .map(|(command, description)| (command.to_string(), description.to_string()))
        .map(|(command, description)| BotCommand {
            command,
            description,
        })
        .collect();
    let set_params = SetMyCommandsParams::builder().commands(commands).build();
    bot.set_my_commands(&set_params).await?;
    Ok(())
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
        // Non-text messages.
        if let Some(sticker) = &msg.sticker
            && matches!(sticker.sticker_type, StickerType::Regular)
        {
            // Get info about stickers.
            let id = &sticker.file_id;
            return reply(bot, &msg, format!("Sticker file_id: <code>{id}</code>")).await;
        } else {
            // Fallback answer.
            return answer_fallback(bot, &msg).await;
        };
    };
    let Some(cmd) = Command::parse(text, username) else {
        // Cannot parse the command
        return answer_fallback(bot, &msg).await;
    };
    info!("Received valid command: `{text}`, parsed as: {cmd:?}");
    match answer_command(bot, &msg, cmd, db, api, config).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to answer the command: {e}");
            Err(e)
        }
    }
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
    let result = match cmd {
        Command::Help => {
            Ok(Command::description(config))
        }
        Command::Search(query) => {
            answer_search(api, &query, db, config).await
        }
        Command::Inline => {
            Ok("🐾 Just mention me in any chat, followed by your query, and I'll pounce into action to fetch the purr-fect meme for you! 😼✨".to_string())
        }
        Command::Sticker(file_id) => {
            if file_id.is_empty() {
                Ok("🐾 Paws and reflect! Please provide a sticker file id... 😾".to_string())
            } else {
                // Send given sticker
                let sticker = FileUpload::String(file_id);
                let send_params = SendStickerParams::builder()
                    .chat_id(msg.chat.id)
                    .sticker(sticker)
                    .build();
                if let Err(e) = bot.send_sticker(&send_params).await {
                    if let Error::Api(e) = e {
                        if e.description.starts_with("Bad Request: wrong remote file identifier specified") {
                            Err("🐾 Paws and reflect! Please provide a valid sticker file id... 😾".to_string())
                        } else {
                            Err(format!("Failed to send the sticker: Api Error {}", e.description))
                        }
                    } else {
                        Err(format!("Failed to send the sticker: {e}"))
                    }
                } else {
                    Ok("🐾 Sticker sent! Hope it made your whiskers twitch! 😼".to_string())
                }
            }
        }
        Command::Add(description) => {
            if let Some(user) = &msg.from {
                if user.id != config.owner {
                    Err("😾 Only my owner can use this command.".to_string())
                } else if let Some(reply) = &msg.reply_to_message && let Some(sticker) = &reply.sticker {
                    insert_sticker(db, api, sticker.file_id.clone(), description).await
                } else {
                    Err("🐾 Paws and reflect! Please provide a sticker file id, or reply to a sticker. 😾".to_string())
                }
            } else {
                Err("😾 Who're you?".to_string())
            }
        }
    };
    let reply_msg = match result {
        Ok(reply) => reply,
        Err(error) => {
            format!("😿 Oops! Something went wrong...\n{error}")
        }
    };

    reply(bot, msg, reply_msg).await
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
    let message: Vec<_> = results
        .iter()
        .map(|(path, similarity, file_id)| {
            let percent = similarity * 100.0;
            format!("🐾 {percent:.2}%: {path} | <code>/sticker {file_id}</code>")
        })
        .collect();
    Ok(message.join("\n"))
}

/// Fallback message.
async fn answer_fallback(bot: &Bot, msg: &Message) -> BotResult<()> {
    // Only answer fallback if the message is a private message.
    if !matches!(msg.chat.type_field, ChatType::Private) {
        return Ok(());
    }
    // Choose a pseudo-random message from the fallback messages.
    let idx = msg.message_id.unsigned_abs() as usize % FALLBACK_MESSAGES.len();
    let reply_msg = FALLBACK_MESSAGES[idx];

    reply(bot, msg, reply_msg.to_string()).await
}

/// Reply to the message.
async fn reply(bot: &Bot, msg: &Message, text: String) -> BotResult<()> {
    let reply_params = ReplyParameters::builder()
        .message_id(msg.message_id)
        .build();
    let link_preview_options = LinkPreviewOptions::DISABLED;
    let send_params = SendMessageParams::builder()
        .chat_id(msg.chat.id)
        .text(text)
        .reply_parameters(reply_params)
        .parse_mode(ParseMode::Html)
        .link_preview_options(link_preview_options)
        .build();
    bot.send_message(&send_params).await?;
    Ok(())
}

/// Insert given sticker to database.
async fn insert_sticker(db: Arc<Mutex<Database>>, api: &ApiClient, file_id: String, description: String) -> Result<String, String> {
    let Ok(raw_embedding) = api.embed(&description).await else {
        return Err("Failed to embed the description".to_string());
    };
    let embedding: Embedding = raw_embedding.into();
    let record = Record {
        embedding,
        file_hash: "Unknown".to_string(),
        file_path: format!("tg-sticker://{file_id}"),
        file_id: Some(file_id),
        label: description,
    };
    let mut db = db.lock().await;
    if let Err(e) = db.insert(record).await {
        Err(format!("Failed to insert record: {e}"))
    } else {
        Ok("Successfully inserted sticker.".to_string())
    }
}
