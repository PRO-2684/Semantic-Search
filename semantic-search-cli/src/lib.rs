//! # Semantic Search CLI
//!
//! This library provides basic functionality for the semantic search CLI.

#![deny(missing_docs)]
#![warn(clippy::all, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions, reason = "Dependencies")]

pub mod commands;
mod config;
mod util;

use anyhow::Result;
use argh::FromArgs;
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
#[allow(clippy::future_not_send, reason = "Main function")]
pub async fn execute(command: Command, config: Config) -> Result<()> {
    debug!("Executing command: {:?}", command);
    debug!("Config: {:?}", config);

    match command {
        Command::Index(index) => {
            info!("Indexing files...");
            let summary = index.execute(config).await?;
            let attention_required = summary.changed + summary.new > 0;
            info!("Indexing complete!");
            if attention_required {
                info!(
                    "Summary: {} file(s) changed, {} file(s) created, {} file(s) deleted. 📝",
                    summary.changed, summary.new, summary.deleted
                );
            } else if summary.deleted > 0 {
                info!("{} file(s) deleted since last index. 🗑️", summary.deleted);
            } else {
                info!("No changes detected. ☕");
            }
        }
        Command::Search(search) => {
            let results = search.execute(config).await?;
            for (file_path, similarity) in results {
                let percent = similarity * 100.0;
                println!("{percent:.2}%: {file_path}");
            }
        }
        Command::Telegram(telegram) => telegram.execute(config).await?,
        Command::Serve(serve) => serve.execute(config).await?,
    };

    Ok(())
}
