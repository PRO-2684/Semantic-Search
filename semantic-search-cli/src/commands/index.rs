//! `index` subcommand

use crate::{
    util::{hash_file, iter_files, prompt, Database, Record},
    Config,
};
use anyhow::{Context, Result};
use argh::FromArgs;
use log::{debug, info, warn};
use semantic_search::ApiClient;

/// generate index of the files
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "index", help_triggers("-h", "--help"))]
pub struct Index {
    /// skip prompting for labels and use filename or existing label
    #[argh(switch, short = 'y')]
    pub yes: bool,
    /// re-embedding files that hash has changed, useful when you edited the labels externally and conveyed the changes by changing the hash
    #[argh(switch, short = 'r')]
    pub re_embed: bool,
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
    #[allow(clippy::future_not_send, reason = "Main function")]
    pub async fn execute(&self, config: Config) -> Result<IndexSummary> {
        // The option `yes` and `re_embed` should not be used together
        if self.yes && self.re_embed {
            anyhow::bail!("Options -y and -r should not be used together");
        }
        let mut db = Database::open(".sense/index.db3", false)
            .await
            .with_context(|| "Failed to open database")?;
        let mut summary = IndexSummary::default();
        let api = ApiClient::new(&config.api.key, config.api.model)?;
        let cwd = std::env::current_dir()?.canonicalize()?;
        summary.deleted = db.clean(&cwd).await?;
        let files = iter_files(&cwd, &cwd);

        // For all files, calculate hash and write to database
        for (path, relative) in files {
            let hash = hash_file(&path)?;
            let relative = relative.to_string();
            let existing = db.get(&relative).await?;

            // Get updated record
            let record = if let Some(mut record) = existing {
                let hash_changed = record.file_hash != hash;
                // Warn if the hash has changed
                if hash_changed {
                    summary.changed += 1;
                    debug!("[CHANGED] {relative}: {} -> {hash}", record.file_hash);
                    warn!("Hash of {relative} has changed, consider relabeling");
                    record.file_hash = hash;
                    record.file_id = None; // Reset file_id

                    if self.re_embed {
                        // Re-embed existing label
                        info!("Re-embedding {relative}");
                        record.embedding = api.embed(&record.label).await?.into();
                    } else if !self.yes {
                        // Prompt for label
                        println!("Existing label: {}", record.label);
                        let label = prompt(&format!("Label for {relative} (empty to keep): "))?;
                        if label.is_empty() {
                            println!("Label kept as: {}", record.label);
                        } else {
                            record.label = label;
                            println!("Label updated to: {}", record.label);
                            record.embedding = api.embed(&relative).await?.into();
                        }
                    } else {
                        // Do nothing if `yes` is set - keep the existing label and embedding
                        info!("Skipping {relative}");
                    }
                } else {
                    // Nothing changed
                    debug!("[SAME] {relative}: {hash}");
                    continue; // Skip to next file - this should improve performance
                }
                // Reuse the record
                record
            } else {
                summary.new += 1;
                debug!("[NEW] {hash}: {relative}");
                warn!("New file: {relative}, consider labeling");

                let (label, embedding) = if self.yes {
                    // Use filename as label
                    let label = path.file_stem().unwrap().to_string_lossy();
                    (label.to_string(), api.embed(&relative).await?.into())
                } else {
                    let label = prompt(&format!("Label for {relative} (empty to use filename): "))?;
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
                    file_id: None,
                    label,
                    embedding,
                }
            };

            db.insert(record).await?;
        }

        Ok(summary)
    }
}
