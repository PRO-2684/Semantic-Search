//! `search` subcommand

use crate::{util::Database, Config};
use anyhow::{Context, Result};
use argh::FromArgs;
use semantic_search::{ApiClient, Embedding};

/// search for files based on labels
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "search", help_triggers("-h", "--help"))]
pub struct Search {
    /// query string
    #[argh(positional)]
    pub query: String,
    /// number of results to show
    #[argh(option, short = 'n', default = "8")]
    pub num_results: usize,
}

impl Search {
    pub async fn execute(&self, config: Config) -> Result<Vec<(String, f32)>> {
        let mut db = Database::open(".sense/index.db3", true)
            .await
            .with_context(|| "Failed to open database, consider indexing first.")?;
        let api = ApiClient::new(&config.api.key, config.api.model)?;
        let embedding: Embedding = api.embed(&self.query).await?.into();
        let results = db.search(self.num_results, &embedding).await?;

        Ok(results)
    }
}
