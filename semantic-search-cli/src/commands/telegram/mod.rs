//! `tg` subcommand

#![allow(unused_imports, unused_variables, reason = "Not implemented yet.")]

mod common;
mod inline;
mod message;

use crate::{config::BotConfig, util::Database, Config};
use anyhow::{Context, Result};
use argh::FromArgs;
use log::debug;
use semantic_search::ApiClient;
use std::sync::Arc;
use teloxide::{
    adaptors::throttle::{Limits, Throttle},
    dispatching::{UpdateFilterExt, UpdateHandler},
    dptree,
    prelude::Dispatcher,
    requests::RequesterExt,
    types::{Me, Message, Update},
    Bot,
};
use tokio::sync::Mutex;

type ThrottledBot = Throttle<Bot>;

/// start a server to search for files
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "tg", help_triggers("-h", "--help"))]
pub struct Telegram {
    // ...
}

impl Telegram {
    pub async fn execute(&self, config: Config) -> Result<()> {
        let db = Database::open(".sense/index.db3", true)
            .await
            .with_context(|| "Failed to open database, consider indexing first.")?;
        let db = Arc::new(Mutex::new(db));
        let api = ApiClient::new(config.api.key, config.api.model)?;

        let BotConfig {
            token,
            whitelist,
            num_results,
        } = config.bot;
        if token.is_empty() {
            anyhow::bail!("No token provided for the Telegram bot.");
        }
        let bot = Bot::new(token).throttle(Limits::default());

        let handler = dptree::entry()
            .filter(move |update: Update| {
                if let Some(user) = update.from() {
                    whitelist.is_empty() || whitelist.contains(&user.id.0)
                } else {
                    false
                }
            })
            .branch(Update::filter_message().endpoint(
                |bot: ThrottledBot,
                 msg: Message,
                 me: Me,
                 db: Arc<Mutex<Database>>,
                 api: ApiClient| async move {
                    message::message_handler(bot, msg, me, db, api).await
                },
            ))
            .branch(Update::filter_inline_query().endpoint(inline::inline_handler));

        Dispatcher::builder(bot, handler)
            .dependencies(dptree::deps![db, api, num_results])
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}
