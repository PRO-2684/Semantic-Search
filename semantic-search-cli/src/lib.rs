//! # Semantic Search CLI
//!
//! This library provides basic functionality for the semantic search CLI.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

pub mod commands;
mod config;
mod util;

use argh::FromArgs;
use anyhow::Result;
use commands::Command;
pub use config::{parse_config, Config};
use log::{debug, info, warn};

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
pub async fn execute(command: Command, config: &Config) -> Result<()> {
    debug!("Executing command: {:?}", command);
    debug!("Config: {:?}", config);

    match command {
        Command::Index(index) => {
            info!("Indexing files...");
            let summary = index.execute(&config).await?;
            let attention_required = summary.changed + summary.unlabeled > 0;
            info!("Indexing complete!");
            if attention_required {
                info!(
                    "Attention: {} file(s) changed, {} file(s) unlabeled. ⭐",
                    summary.changed, summary.unlabeled
                );
            } else if summary.deleted > 0 {
                info!("{} file(s) deleted since last index. 🗑️", summary.deleted);
            } else {
                info!("No changes detected. ☕");
            }
        }
        Command::Search(search) => search.execute(config.key()),
        Command::Serve(serve) => serve.execute(config.port(), config.key()),
    };

    Ok(())
}
