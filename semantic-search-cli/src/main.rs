#![warn(clippy::all, clippy::nursery, clippy::pedantic, clippy::cargo)]

use semantic_search_cli::{execute, parse_config, Args};
use log::debug;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder().init();

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
