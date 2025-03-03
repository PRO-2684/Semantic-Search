//! `tg` subcommand

#![allow(unused_imports, unused_variables, reason = "Not implemented yet.")]

use crate::{config::BotConfig, Config};
use anyhow::{Context, Result};
use argh::FromArgs;
use log::debug;
use teloxide::{
    adaptors::throttle::Limits,
    dispatching::{MessageFilterExt, UpdateFilterExt},
    prelude::{Dispatcher, Requester, ResponseResult},
    repls::CommandReplExt,
    requests::RequesterExt,
    types::{Me, Message, Update},
    utils::command::BotCommands,
    Bot,
};

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

        let schema = Update::filter_message()
            .filter(move |update: Update| {
                if let Some(user) = update.from() {
                    debug!("User: {:?}", user);
                    whitelist.is_empty() || whitelist.contains(&user.id.0)
                } else {
                    false
                }
            })
            .branch(Message::filter_text().endpoint(answer));

        Dispatcher::builder(bot, schema).enable_ctrlc_handler().build().dispatch().await;

        Ok(())
    }
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "handle a username.")]
    Username(String),
    #[command(description = "handle a username and an age.", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Username(username) => {
            bot.send_message(msg.chat.id, format!("Your username is @{username}."))
                .await?
        }
        Command::UsernameAndAge { username, age } => {
            bot.send_message(
                msg.chat.id,
                format!("Your username is @{username} and age is {age}."),
            )
            .await?
        }
    };

    Ok(())
}
