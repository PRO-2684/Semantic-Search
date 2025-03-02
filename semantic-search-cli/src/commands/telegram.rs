//! `tg` subcommand

#![allow(unused_imports, unused_variables, reason = "Not implemented yet.")]

use crate::Config;
use anyhow::{Context, Result};
use argh::FromArgs;

/// start a server to search for files
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "tg", help_triggers("-h", "--help"))]
pub struct Telegram {
    // ...
}

impl Telegram {
    pub async fn execute(&self, config: Config) -> Result<()> {
        // ...
        Ok(())
    }
}
