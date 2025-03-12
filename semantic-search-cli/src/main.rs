#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

use anyhow::{Context, Result};
use env_logger::Env;
use log::debug;
use semantic_search_cli::{Args, execute, parse_config};
use std::io::Write;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let level = record.level();
            let style = buf.default_level_style(level);
            writeln!(buf, "[{style}{level}{style:#}] {}", record.args())
        })
        .init();

    let args: Args = argh::from_env();
    debug!("Args: {:?}", args);
    debug!("Working directory: {:?}", std::env::current_dir()?);

    let config = parse_config(Path::new(".sense/config.toml"))
        .with_context(|| "Failed to parse config file, consider creating one")?;

    Box::pin(execute(args.command, config)).await?;

    Ok(())
}
