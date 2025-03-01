//! # Semantic Search CLI
//!
//! This library provides basic functionality for the semantic search CLI.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

pub mod commands;
mod config;
mod util;

use argh::FromArgs;
use commands::Command;
pub use config::{parse_config, Config};
use log::debug;
use std::io::Result as IOResult;

/// 🔎 Semantic search.
#[derive(FromArgs, Debug)]
#[argh(help_triggers("-h", "--help"))]
pub struct Args {
    /// the command to execute.
    #[argh(subcommand)]
    pub command: Command,
}

/// Execute the command.
///
/// # Errors
///
/// Returns an [IO error](std::io::Error) if reading or writing fails.
pub fn execute(command: Command, config: &Config) -> IOResult<()> {
    debug!("Executing command: {:?}", command);
    debug!("Config: {:?}", config);

    match command {
        Command::Index(_) => commands::index()?,
        Command::Embed(embed) => embed.execute(&config.key()),
        Command::Search(search) => search.execute(&config.key()),
        Command::Serve(serve) => serve.execute(config.port(), &config.key()),
    };

    Ok(())
}
