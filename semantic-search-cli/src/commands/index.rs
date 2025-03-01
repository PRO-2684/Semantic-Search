//! `index` subcommand

use crate::util::{hash_file, init, walk_dir};
use argh::FromArgs;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Result as IOResult};

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

/// Index files.
pub fn index() -> IOResult<()> {
    init()?;
    println!("Indexing files...");

    // Overwrite `.sense/index.csv` if it exists (always start afresh)
    let index = File::create(".sense/index.csv")?;
    let mut writer = csv::Writer::from_writer(index);
    debug!("Index file created");

    // For all files, calculate hash and write to `.sense/index.csv`
    let cwd = std::env::current_dir()?.canonicalize()?;
    walk_dir(&cwd, &cwd,  &mut |path, relative| {
        let hash = hash_file(path)?;
        debug!("{hash}: {relative}");

        // Write to `.sense/index.csv`
        writer.serialize(IndexRecord {
            path: relative.to_string(),
            hash,
            label: "".into(),
        })?;

        Ok(())
    })?;

    println!("Indexing complete! Consider labeling the files at `.sense/index.csv`.");
    Ok(())
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
