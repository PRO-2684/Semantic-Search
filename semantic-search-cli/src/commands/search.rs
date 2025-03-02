//! `search` subcommand

use crate::{
    util::{Database, TABLE_NAME},
    Config,
};
use anyhow::{Context, Result};
use argh::FromArgs;
use semantic_search::{embedding::EmbeddingBytes, ApiClient, Embedding, Model};

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
    pub async fn execute(&self, config: &Config) -> Result<Vec<(String, f32)>> {
        let db = Database::open(".sense/index.db3", true)
            .with_context(|| "Failed to open database, did you index files?")?;
        let api = ApiClient::new(config.key().to_owned(), Model::BgeLargeZhV1_5)?;
        let embedding: Embedding = api.embed(&self.query).await?.into();

        let mut stmt = db.prepare(&format!("SELECT file_path, embedding FROM {TABLE_NAME}"))?;
        let rows = stmt.query_map([], |row| {
            let file_path: String = row.get(0)?;
            let embedding: EmbeddingBytes = row.get(1)?;
            let embedding: Embedding = embedding.into();
            Ok((file_path, embedding))
        })?;

        let mut results = Vec::with_capacity(self.num_results);
        for row in rows {
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
