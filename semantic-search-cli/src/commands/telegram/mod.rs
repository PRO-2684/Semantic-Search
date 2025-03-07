//! `tg` subcommand

#![allow(unused_imports, unused_variables, reason = "Not implemented yet.")]

mod common;
mod inline;
mod message;

use crate::{config::BotConfig, util::Database, Config};
use anyhow::{Context, Result};
use argh::FromArgs;
use common::upload_or_reuse;
use frankenstein::{client_reqwest::Bot, AsyncTelegramApi, GetUpdatesParams, UpdateContent};
use log::{debug, error};
use semantic_search::ApiClient;

/// start a server to search for files
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "tg", help_triggers("-h", "--help"))]
pub struct Telegram {
    // ...
}

impl Telegram {
    pub async fn execute(&self, config: Config) -> Result<()> {
        let mut db = Database::open(".sense/index.db3", false)
            .await
            .with_context(|| "Failed to open database, consider indexing first.")?;
        let api = ApiClient::new(config.api.key, config.api.model)?;

        let BotConfig {
            token, whitelist, ..
        } = &config.bot;
        if token.is_empty() {
            anyhow::bail!("No token provided for the Telegram bot.");
        }
        let bot = Bot::new(token); // TODO: throttle
        let me = bot.get_me().await?.result;

        let mut update_params = GetUpdatesParams::builder().build();
        loop {
            match bot.get_updates(&update_params).await {
                Ok(updates) => {
                    for update in updates.result {
                        debug!("Received update: {update:?}");
                        update_params.offset.replace(i64::from(update.update_id) + 1);

                        match update.content {
                            UpdateContent::Message(msg) => {
                                let Some(sender) = &msg.from else {
                                    continue;
                                };
                                let sender = sender.id;
                                if !whitelist.is_empty() && !whitelist.contains(&sender) {
                                    continue;
                                }

                                message::message_handler(&bot, &me, msg, &mut db, &api, &config.bot).await?;
                            },
                            UpdateContent::InlineQuery(query) => {
                                let sender = query.from.id;
                                if !whitelist.is_empty() && !whitelist.contains(&sender) {
                                    continue;
                                }

                                inline::inline_handler(&bot, &me, query, &mut db, &api, &config.bot).await?;
                            },
                            _ => {},
                        }
                    }
                },
                Err(error) => {
                    error!("Failed to get updates: {error:?}");
                },
            };
        }

        // Ok(())
    }
}
