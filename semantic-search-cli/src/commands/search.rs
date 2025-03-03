//! `search` subcommand

use crate::{util::Database, Config};
use anyhow::{Context, Result};
use argh::FromArgs;
use futures_util::StreamExt;
use semantic_search::{ApiClient, Embedding, Model};

/// search for files based on labels
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "search", help_triggers("-h", "--help"))]
pub struct Search {
    /// query string
    #[argh(positional)]
    pub query: String,
    /// number of results to show
    #[argh(option, short = 'n', default = "5")]
    pub num_results: usize,
}

impl Search {
    pub async fn execute(&self, config: Config) -> Result<Vec<(String, f32)>> {
        let mut db = Database::open(".sense/index.db3", true)
            .await
            .with_context(|| "Failed to open database, consider indexing first.")?;
        let api = ApiClient::new(config.api.key, Model::BgeLargeZhV1_5)?;
        let embedding: Embedding = api.embed(&self.query).await?.into();

        let mut rows = db.iter_embeddings();
        let mut results = Vec::with_capacity(self.num_results);

        while let Some(row) = rows.next().await {
            let (file_path, other_embedding) = row?;
            let similarity = embedding.cosine_similarity(&other_embedding);
            // Top N results
            if results.len() < self.num_results {
                results.push((file_path, similarity));
            } else if results.last().unwrap().1 < similarity {
                results.pop();
                results.push((file_path, similarity));
            }
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        }

        Ok(results)
    }
}
