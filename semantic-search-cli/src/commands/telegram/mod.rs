//! `tg` subcommand

mod common;
mod inline;
mod message;

use std::sync::Arc;

use crate::{Config, config::BotConfig, util::Database};
use anyhow::{Context, Result};
use argh::FromArgs;
use frankenstein::{
    AsyncTelegramApi, Error, client_reqwest::Bot, methods::GetUpdatesParams, updates::UpdateContent,
};
use log::{debug, error, info};
use semantic_search::ApiClient;
use tokio::sync::Mutex;

type BotResult<T> = Result<T, Error>;

/// start Telegram bot
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "tg", help_triggers("-h", "--help"))]
pub struct Telegram {
    // ...
}

impl Telegram {
    /// Start the Telegram bot.
    ///
    /// # Memory Leak
    ///
    /// Note that this function leaks `api`, `bot`, `me` and `bot_config`, so it shouldn't be called repeatedly. The rationale is that:
    ///
    /// 1. The function should run indefinitely
    /// 2. Typically it will be called only once in a program's lifetime
    /// 3. The leaked memory is small and will be freed when the program exits
    /// 4. It avoids the need to clone or `Arc` the objects
    pub async fn execute(&self, config: Config) -> Result<()> {
        let mut db = Database::open(".sense/index.db3", false)
            .await
            .with_context(|| "Failed to open database, consider indexing first.")?;
        let api = ApiClient::new(&config.api.key, config.api.model)?;

        let token = &config.bot.token;
        if token.is_empty() {
            anyhow::bail!("No token provided for the Telegram bot.");
        }
        let bot = Bot::new(token); // TODO: throttle
        let me = bot.get_me().await?.result;
        info!("Bot username: {:?}", me.username);

        // Set commands
        info!("Setting commands...");
        message::set_commands(&bot).await?;

        // Upload stickers
        info!("Initializing stickers...");
        let init_result = common::init_stickers(&bot, &me, &mut db, &config.bot).await;
        if let Err(e) = init_result {
            db.close().await?;
            anyhow::bail!("Failed to initialize stickers: {e}");
        }
        info!("Initialized stickers, start handling updates...");

        // Leaking `api`, `bot`, `me` and `bot_config` here
        let bot = Box::leak(Box::new(bot));
        let me = Box::leak(Box::new(me));
        let api = Box::leak(Box::new(api));
        let bot_config = Box::leak(Box::new(config.bot));
        let whitelist = &bot_config.whitelist;

        let db = Arc::new(Mutex::new(db));
        let mut update_params = GetUpdatesParams::builder().build();
        loop {
            match bot.get_updates(&update_params).await {
                Ok(updates) => {
                    for update in updates.result {
                        debug!("Received update: {update:?}");
                        update_params.offset.replace((update.update_id + 1).into());

                        match update.content {
                            UpdateContent::Message(msg) => {
                                let Some(sender) = &msg.from else {
                                    continue;
                                };
                                let sender = sender.id;
                                if !whitelist.is_empty() && !whitelist.contains(&sender) {
                                    continue;
                                }

                                tokio::spawn(message::message_handler(
                                    bot,
                                    me,
                                    msg,
                                    db.clone(),
                                    api,
                                    bot_config,
                                ));
                            }
                            UpdateContent::InlineQuery(query) => {
                                let sender = query.from.id;
                                if !whitelist.is_empty() && !whitelist.contains(&sender) {
                                    continue;
                                }

                                tokio::spawn(inline::inline_handler(
                                    bot,
                                    query,
                                    db.clone(),
                                    api,
                                    bot_config,
                                ));
                            }
                            _ => {}
                        }
                    }
                }
                Err(error) => {
                    error!("Failed to get updates: {error:?}");
                }
            };
        }

        // Ok(())
    }
}
