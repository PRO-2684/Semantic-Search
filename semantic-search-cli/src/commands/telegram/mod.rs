//! `tg` subcommand

#![allow(unused_imports, unused_variables, reason = "Not implemented yet.")]

mod common;
mod inline;
mod message;

use crate::{config::BotConfig, Config};
use anyhow::{Context, Result};
use argh::FromArgs;
use log::debug;
use teloxide::{
    adaptors::throttle::{Limits, Throttle}, dispatching::{UpdateFilterExt, UpdateHandler}, dptree, prelude::Dispatcher, requests::RequesterExt, types::Update, Bot
};

type ThrottledBot = Throttle<Bot>;

/// start a server to search for files
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "tg", help_triggers("-h", "--help"))]
pub struct Telegram {
    // ...
}

impl Telegram {
    pub async fn execute(&self, config: Config) -> Result<()> {
        let BotConfig { token, whitelist } = config.bot;
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
            .branch(Update::filter_message().endpoint(message::message_handler))
            .branch(Update::filter_inline_query().endpoint(inline::inline_handler));

        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;

        Ok(())
    }
}
