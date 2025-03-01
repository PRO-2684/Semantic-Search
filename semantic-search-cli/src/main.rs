#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

use std::io::Write;
use env_logger::Env;
use log::debug;
use semantic_search_cli::{execute, parse_config, Args};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let config = parse_config(Path::new(".sense/config.toml")).map_err(|e| {
        // Attach a custom message to the error
        let msg = format!("Failed to parse config: {}", e);
        std::io::Error::new(std::io::ErrorKind::Other, msg)
    })?;
    debug!("Server port: {}", config.port());
    debug!("API key: {}", config.key());

    execute(args.command, &config)?;

    Ok(())
}
