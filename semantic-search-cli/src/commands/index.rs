//! `index` subcommand

use crate::util::{hash_file, init, walk_dir};
use argh::FromArgs;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Result as IOResult};

/// generate index of the files
#[derive(FromArgs, PartialEq, Eq, Debug)]
#[argh(subcommand, name = "index")]
pub struct Index {}

/// An index record.
#[derive(Serialize, Deserialize)]
pub struct IndexRecord {
    /// Path to the file (relative to working directory)
    pub path: String,
    /// SHA-256 hash of the file
    pub hash: String,
    /// Label of the file
    pub label: String,
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

/// Index files.
pub fn index() -> IOResult<IndexSummary> {
    init()?;

    // Read `.sense/index.csv` if it exists
    let mut existing = HashMap::new();
    if let Ok(index) = File::open(".sense/index.csv") {
        let mut reader = csv::Reader::from_reader(index);
        for result in reader.deserialize() {
            let record: IndexRecord = result?;
            existing.insert(record.path.clone(), record);
        }
    }

    // Overwrite `.sense/index.csv` if it exists
    let mut summary = IndexSummary::default();
    let index = File::create(".sense/index.csv")?;
    let mut writer = csv::Writer::from_writer(index);
    debug!("Index file created.");

    // For all files, calculate hash and write to `.sense/index.csv`
    let cwd = std::env::current_dir()?.canonicalize()?;
    walk_dir(&cwd, &cwd, &mut |path, relative| {
        let hash = hash_file(path)?;
        let relative = relative.to_string();

        let record = match existing.remove(&relative) {
            // If the file is already indexed
            Some(mut record) => {
                // Warn if the hash has changed
                if record.hash != hash {
                    summary.changed += 1;
                    warn!("Hash of {relative} has changed, consider relabeling",);
                    debug!("[CHANGED] {relative}: {} -> {hash}", record.hash);
                    record.hash = hash;
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
                IndexRecord {
                    path: relative,
                    hash,
                    label: "".into(),
                }
            }
        };

        writer.serialize(record)?;
        Ok(())
    })?;

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn serialize() -> Result<(), Box<dyn Error>> {
        let mut writer = csv::Writer::from_writer(vec![]);
        writer.serialize(IndexRecord {
            path: "LICENSE".into(),
            hash: "3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986".into(),
            label: "My Label".into(),
        })?;

        let data = String::from_utf8(writer.into_inner()?)?;
        assert_eq!(
            data,
            "path,hash,label\nLICENSE,3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986,My Label\n"
        );

        Ok(())
    }
}
