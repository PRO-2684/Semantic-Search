//! `index` subcommand

use crate::{util::{hash_file, Database, iter_files, Record}, Config};
use argh::FromArgs;
use log::{debug, warn};
use semantic_search::{ApiClient, Embedding, Model};

/// generate index of the files
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "index")]
pub struct Index {
    /// skip prompting for labels and use empty labels
    #[argh(switch, short = 'y')]
    pub yes: bool,
}

/// Summary of the index operation.
#[derive(Debug, Default)]
pub struct IndexSummary {
    /// Number of unchanged files
    pub unchanged: usize,
    /// Number of changed files
    pub changed: usize,
    /// Number of new files
    pub new: usize,
}

impl Index {
    /// Index files.
    pub async fn execute(&self, config: &Config) -> Result<IndexSummary, Box<dyn std::error::Error>> {
        let db = Database::open()?;
        let mut summary = IndexSummary::default();

        // Initialize the API client
        let api = ApiClient::new(config.key().to_owned(), Model::BgeLargeZhV1_5)?;

        // For all files, calculate hash and write to database
        let cwd = std::env::current_dir()?.canonicalize()?;
        // walk_dir(&cwd, &cwd, &mut async |path, relative| {
        // })?;

        let files = iter_files(&cwd, &cwd)?;
        for (path, relative) in files {
            let hash = hash_file(path)?;
            let relative = relative.to_string();
            let existing = db.get(&relative)?;

            let record = match existing {
                // If the file is already indexed
                Some(mut record) => {
                    // Warn if the hash has changed
                    if record.file_hash != hash {
                        summary.changed += 1;
                        debug!("[CHANGED] {relative}: {} -> {hash}", record.file_hash);
                        warn!("Hash of {relative} has changed, consider relabeling");
                        if !self.yes {
                            // Prompt for label
                            println!("Existing label: {}", record.label);
                            print!("Label for {relative} (empty to keep): ");
                            let mut label = String::new();
                            std::io::stdin().read_line(&mut label)?;
                            label = label.trim().to_owned();
                            if !label.is_empty() {
                                record.label = label;
                            }
                        }
                        record.file_hash = hash;
                    } else {
                        summary.unchanged += 1;
                        debug!("[SAME] {relative}: {hash}");
                    }
                    // Reuse the record
                    record
                }
                // Generate a new record
                None => {
                    summary.new += 1;
                    debug!("[NEW] {hash}: {relative}");
                    println!("New file: {relative}");
                    let (label, embedding) = if self.yes {
                        ("".into(), Embedding::default())
                    } else {
                        print!("Label for {relative} (empty to skip): ");
                        let mut label = String::new();
                        std::io::stdin().read_line(&mut label)?;
                        let label = label.trim().to_owned();
                        if label.is_empty() {
                            (label, Embedding::default())
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


#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    // #[test]
    // fn serialize() -> Result<(), Box<dyn Error>> {
    //     let mut writer = csv::Writer::from_writer(vec![]);
    //     writer.serialize(IndexRecord {
    //         path: "LICENSE".into(),
    //         hash: "3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986".into(),
    //         label: "My Label".into(),
    //     })?;

    //     let data = String::from_utf8(writer.into_inner()?)?;
    //     assert_eq!(
    //         data,
    //         "path,hash,label\nLICENSE,3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986,My Label\n"
    //     );

    //     Ok(())
    // }
}
