//! Configuration file parser.

use anyhow::Result as AnyResult;
use std::path::Path;

use semantic_search::Model;
use serde::Deserialize;

/// Structure of the configuration file.
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Server configuration.
    #[serde(default)]
    pub server: Server,
    /// API configuration.
    pub api: ApiConfig,
    /// Telegram bot configuration.
    #[serde(default)]
    pub bot: BotConfig,
}

/// Server configuration.
#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Server {
    /// Port for the server. Default is 8080.
    pub port: u16,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            port: 8080,
        }
    }
}

/// API configuration.
#[derive(Deserialize, Debug)]
pub struct ApiConfig {
    /// API key for Silicon Cloud.
    pub key: String,
    /// Model to use for embedding.
    #[serde(default)]
    pub model: Model,
}

/// Telegram bot configuration.
#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct BotConfig {
    /// Telegram bot token.
    pub token: String,
    /// Telegram user ID of the bot owner.
    pub owner: u64,
    /// Whitelisted user IDs.
    pub whitelist: Vec<u64>,
    /// Sticker set id prefix for the bot.
    pub sticker_set: String,
    /// Number of results to return.
    pub num_results: usize,
    /// Postscript to be appended after the help message.
    pub postscript: String,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            token: String::new(),
            owner: 0,
            whitelist: Vec::new(),
            num_results: 8,
            sticker_set: "meme".to_string(),
            postscript: String::new(),
        }
    }
}

/// Parse the configuration into a `Config` structure.
///
/// # Errors
///
/// Returns an [`Error`](toml::de::Error) if the configuration file is not valid, like missing fields.
fn parse_config_from_str(content: &str) -> Result<Config, toml::de::Error> {
    toml::from_str(content)
}

/// Parse the configuration file into a `Config` structure.
///
/// # Errors
///
/// Returns an [IO error](std::io::Error) if reading fails, or a [TOML error](toml::de::Error) if parsing fails.
pub fn parse_config<T>(path: T) -> AnyResult<Config>
where
    T: AsRef<Path>,
{
    let content = std::fs::read_to_string(path)?;
    Ok(parse_config_from_str(&content)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test(content: &str, port: u16, key: &str, model: Model, bot_token: &str) {
        let config = parse_config_from_str(content).unwrap();
        assert_eq!(config.server.port, port);
        assert_eq!(config.api.key, key);
        assert_eq!(config.api.model, model);
        assert_eq!(config.bot.token, bot_token);
    }

    #[test]
    fn parse_config_1() {
        let content = r#"
            [server]
            port = 8081

            [api]
            key = "test_key"

            [bot]
            token = "test_token"
        "#;
        test(
            content,
            8081,
            "test_key",
            Model::BgeLargeZhV1_5,
            "test_token",
        );
    }

    #[test]
    fn parse_config_2() {
        let content = r#"
            [server]
            port = 8080

            [api]
            key = "test_key"
            model = "BAAI/bge-large-zh-v1.5"
        "#;
        test(content, 8080, "test_key", Model::BgeLargeZhV1_5, "");
    }

    #[test]
    fn parse_config_3() {
        let content = r#"
            [server]

            [api]
            key = "test_key"
            model = "BAAI/bge-large-en-v1.5"
        "#;
        test(content, 8080, "test_key", Model::BgeLargeEnV1_5, "");
    }

    #[test]
    fn parse_config_4() {
        let content = r#"
            [api]
            key = "test_key"
        "#;
        test(content, 8080, "test_key", Model::BgeLargeZhV1_5, "");
    }

    #[test]
    fn parse_config_5() {
        let content = r#"
            [server]
            port = 8081

            [api]
            key = "test_key"

            [bot]
        "#;
        test(content, 8081, "test_key", Model::BgeLargeZhV1_5, "");
    }

    #[test]
    #[should_panic(expected = "missing field `api`")]
    fn parse_config_fail_1() {
        let content = r"
            [server]
            port = 8080
        ";
        test(content, 8080, "test_key", Model::BgeLargeZhV1_5, "");
    }

    #[test]
    #[should_panic(expected = "missing field `key`")]
    fn parse_config_fail_2() {
        let content = r"
            [api]
        ";
        test(content, 8080, "test_key", Model::BgeLargeZhV1_5, "");
    }
}
