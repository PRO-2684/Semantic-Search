//! Configuration file parser.

use anyhow::Result as AnyResult;
use std::path::Path;

use semantic_search::Model;
use serde::Deserialize;

/// Structure of the configuration file. Example:
///
/// ```toml
/// [server]
/// port = 8080
///
/// [api]
/// key = "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" # API key for SiliconCloud (Required)
/// model = "BAAI/bge-large-zh-v1.5" # Model to use for embedding (Optional)
/// ```
#[derive(Deserialize)]
struct ConfigToml {
    /// Server configuration.
    server: Option<Server>,
    /// API configuration.
    api: ApiConfig,
}

/// Server configuration.
#[derive(Deserialize)]
struct Server {
    /// Port for the server. Default is 8080.
    port: Option<u16>,
}

/// API configuration.
#[derive(Deserialize)]
struct ApiConfig {
    /// API key for Silicon Cloud.
    key: String,
    /// Model to use for embedding.
    model: Option<String>,
}

/// Flattened configuration. Available methods:
///
/// - [`port`](Self::port): Port for the server.
/// - [`key`](Self::key): API key for Silicon Cloud.
/// - [`model`](Self::model): Model to use for embedding.
#[derive(Debug)]
pub struct Config {
    /// Port for the server.
    port: u16,
    /// API key for Silicon Cloud.
    key: String,
    /// Model to use for embedding. FIXME: Unused for now.
    model: String,
}

impl From<ConfigToml> for Config {
    fn from(value: ConfigToml) -> Self {
        let server = value.server.unwrap_or(Server { port: None });
        let port = server.port.unwrap_or(8080);
        let key = value.api.key;
        let model = value.api.model.unwrap_or(Model::default().to_string());

        Self { port, key, model }
    }
}

impl Config {
    /// Get the port for the server.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the API key for Silicon Cloud.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the model to use for embedding.
    pub fn model(&self) -> &str {
        &self.model
    }
}

/// Parse the configuration into a `Config` structure.
///
/// # Errors
///
/// Returns an [`Error`](toml::de::Error) if the configuration file is not valid, like missing fields.
fn parse_config_from_str(content: &str) -> Result<Config, toml::de::Error> {
    let config_toml: ConfigToml = toml::from_str(content)?;

    Ok(config_toml.into())
}

/// Parse the configuration file into a `Config` structure.
///
/// # Errors
///
/// Returns an [IO error](std::io::Error) if reading fails, or a [TOML error](toml::de::Error) if parsing fails.
pub fn parse_config(path: &Path) -> AnyResult<Config> {
    let content = std::fs::read_to_string(path)?;
    Ok(parse_config_from_str(&content)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test(content: &str, port: u16, key: &str, model: &str) {
        let config = parse_config_from_str(content).unwrap();
        assert_eq!(config.port(), port);
        assert_eq!(config.key(), key);
        assert_eq!(config.model(), model);
    }

    #[test]
    fn parse_config_1() {
        let content = r#"
            [server]
            port = 8081

            [api]
            key = "test_key"
        "#;
        test(content, 8081, "test_key", "BAAI/bge-large-zh-v1.5");
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
        test(content, 8080, "test_key", "BAAI/bge-large-zh-v1.5");
    }

    #[test]
    fn parse_config_3() {
        let content = r#"
            [server]

            [api]
            key = "test_key"
            model = "BAAI/bge-large-en-v1.5"
        "#;
        test(content, 8080, "test_key", "BAAI/bge-large-en-v1.5");
    }

    #[test]
    fn parse_config_4() {
        let content = r#"
            [api]
            key = "test_key"
        "#;
        test(content, 8080, "test_key", "BAAI/bge-large-zh-v1.5");
    }

    #[test]
    #[should_panic(expected = "missing field `api`")]
    fn parse_config_fail_1() {
        let content = r#"
            [server]
            port = 8080
        "#;
        test(content, 8080, "test_key", "BAAI/bge-large-zh-v1.5");
    }

    #[test]
    #[should_panic(expected = "missing field `key`")]
    fn parse_config_fail_2() {
        let content = r#"
            [api]
        "#;
        test(content, 8080, "test_key", "BAAI/bge-large-zh-v1.5");
    }
}
