//! `index` subcommand

use crate::{
    util::{hash_file, iter_files, prompt, Database, Record},
    Config,
};
use anyhow::{Context, Result};
use argh::FromArgs;
use log::{debug, warn};
use semantic_search::{ApiClient, Model};

/// generate index of the files
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "index", help_triggers("-h", "--help"))]
pub struct Index {
    /// skip prompting for labels and use empty labels
    #[argh(switch, short = 'y')]
    pub yes: bool,
}

/// Summary of the index operation.
#[derive(Debug, Default)]
pub struct IndexSummary {
    /// Number of changed files
    pub changed: usize,
    /// Number of new files
    pub new: usize,
    /// Number of deleted files
    pub deleted: usize,
}

impl Index {
    /// Index files.
    pub async fn execute(&self, config: &Config) -> Result<IndexSummary> {
        let db =
            Database::open(".sense/index.db3", false).with_context(|| "Failed to open database")?;
        let mut summary = IndexSummary::default();
        let api = ApiClient::new(config.key().to_owned(), Model::BgeLargeZhV1_5)?;
        let cwd = std::env::current_dir()?.canonicalize()?;
        let files = iter_files(&cwd, &cwd);
        summary.deleted = db.clean(&cwd)?;

        // For all files, calculate hash and write to database
        for (path, relative) in files {
            let hash = hash_file(&path)?;
            let relative = relative.to_string();
            let existing = db.get(&relative)?;

            let record = match existing {
                // If the file is already indexed
                Some(mut record) => {
                    let hash_changed = record.file_hash != hash;
                    // Warn if the hash has changed
                    if hash_changed {
                        summary.changed += 1;
                        debug!("[CHANGED] {relative}: {} -> {hash}", record.file_hash);
                        warn!("Hash of {relative} has changed, consider relabeling");
                        record.file_hash = hash;

                        if !self.yes {
                            println!("Existing label: {}", record.label);
                            // Prompt for label
                            let label = prompt(&format!("Label for {relative} (empty to keep): "))?;
                            if !label.is_empty() {
                                record.label = label;
                                println!("Label updated to: {}", record.label);
                                record.embedding = api.embed(&relative).await?.into();
                            } else {
                                println!("Label kept as: {}", record.label);
                            }
                        }
                    } else {
                        debug!("[SAME] {relative}: {hash}");
                    }
                    // Reuse the record
                    record
                }
                // Generate a new record
                None => {
                    summary.new += 1;
                    debug!("[NEW] {hash}: {relative}");
                    warn!("New file: {relative}, consider labeling");

                    let (label, embedding) = if self.yes {
                        // Use filename as label
                        let label = path.file_stem().unwrap().to_string_lossy();
                        (label.to_string(), api.embed(&relative).await?.into())
                    } else {
                        let label =
                            prompt(&format!("Label for {relative} (empty to use filename): "))?;
                        if label.is_empty() {
                            // Use filename as label
                            let label = path.file_stem().unwrap().to_string_lossy();
                            (label.to_string(), api.embed(&relative).await?.into())
                        } else {
                            let embedding = api.embed(&relative).await?;
                            (label, embedding.into())
                        }
                    };
                    Record {
                        file_path: relative,
                        file_hash: hash,
                        label,
                        embedding,
                    }
                }
            };

            db.insert(record)?;
        }

        Ok(summary)
    }
}
