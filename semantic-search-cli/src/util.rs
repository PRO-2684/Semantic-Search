//! Utility functions for the semantic search CLI.

use log::info;
use semantic_search::{Embedding, embedding::EmbeddingBytes};
use sha2::{Digest, Sha256};
use std::{
    borrow::Cow,
    fs::File,
    io::{Read, Result as IOResult, Write},
    path::{Path, PathBuf},
};
use rusqlite::{blob::{Blob, ZeroBlob}, Connection, Result as SqlResult};

const TABLE_NAME: &str = "files";

/// Calculate SHA-256 hash of a file.
pub fn hash_file<T: AsRef<Path>>(file: T) -> IOResult<String> {
    let mut file = File::open(file)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    let result = base16ct::lower::encode_string(&result);

    Ok(result)
}

/// Walk a directory and call a function on each file path and its relative path to `ref_path`, skipping entries starting with `.`.
///
/// Note that it assumes `dir` and `ref_path` are canonicalized directories.
pub fn walk_dir<T1: AsRef<Path>, T2: AsRef<Path>>(
    dir: T1,
    ref_path: T2,
    func: &mut impl FnMut(&PathBuf, Cow<'_, str>) -> Result<(), Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ref_path = ref_path.as_ref();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.file_name().unwrap().to_string_lossy().starts_with('.') {
            continue;
        }
        let path = path.canonicalize()?;
        if path.is_dir() {
            walk_dir(&path, ref_path, func)?;
        } else {
            // Get relative path (relative to `ref_path`)
            let relative = path.strip_prefix(ref_path).unwrap().to_string_lossy();

            func(&path, relative)?;
        }
    }

    Ok(())
}

/// Check if a file is hidden.
fn is_hidden(entry: &Path) -> bool {
    entry.file_name().unwrap().to_string_lossy().starts_with('.')
}

/// Returns an iterator of all files in a directory, skipping hidden files.
pub fn iter_files<'a, T1: AsRef<Path>, T2: AsRef<Path> + 'a>(dir: T1, ref_path: T2) -> IOResult<impl Iterator<Item = (PathBuf, String)>> {
    // TODO: Recursive
    let iter = std::fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && !is_hidden(path))
        .map(move |path| {
            let relative = path.strip_prefix(ref_path.as_ref()).unwrap().to_string_lossy().to_string();
            (path, relative)
        });

    Ok(iter)
}

/// A record in the database.
pub struct Record {
    /// Path to the file (relative to working directory)
    pub file_path: String,
    /// SHA-256 hash of the file
    pub file_hash: String,
    /// Label of the file
    pub label: String,
    /// Embedding of the file
    pub embedding: Embedding,
}

/// Simple database wrapper.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open a database connection, creating if not exists.
    pub fn open() -> SqlResult<Self> {
        let db_path = Path::new(".sense/index.db3");
        let exists = db_path.exists();
        let conn = Connection::open(db_path)?;

        if exists {
            return Ok(Self { conn });
        }

        info!("Initializing database...");

        conn.execute(
            &format!(
                "CREATE TABLE {TABLE_NAME} (
                    file_path TEXT PRIMARY KEY,
                    file_hash TEXT NOT NULL,
                    label TEXT NOT NULL,
                    embedding BLOB NOT NULL
                )"
            ),
            [],
        )?;

        Ok(Self { conn })
    }

    /// Insert a record into the database, replacing if exists.
    pub fn insert(&self, record: Record) -> SqlResult<()> {
        self.conn.execute(
            &format!(
                "INSERT OR REPLACE INTO {TABLE_NAME} (file_path, file_hash, label, embedding) VALUES (?, ?, ?, ZEROBLOB(4096))"
            ),
            [
                &record.file_path,
                &record.file_hash,
                &record.label,
            ],
        )?;
        let row_id = self.conn.last_insert_rowid();
        let mut blob = self.conn.blob_open(rusqlite::DatabaseName::Main, TABLE_NAME, "embedding", row_id, false)?;
        let bytes: EmbeddingBytes = record.embedding.into();
        blob.write_at(&bytes, 0);

        Ok(())
    }

    /// Get a record from the database.
    pub fn get(&self, file_path: &str) -> SqlResult<Option<Record>> {
        let mut stmt = self.conn.prepare(
            &format!(
                "SELECT file_hash, label, embedding FROM {TABLE_NAME} WHERE file_path = ?"
            ),
        )?;
        let mut rows = stmt.query([&file_path])?;

        if let Some(row) = rows.next()? {
            let file_hash: String = row.get(0)?;
            let label: String = row.get(1)?;
            let bytes: EmbeddingBytes = row.get(2)?;
            let embedding: Embedding = bytes.into();

            Ok(Some(Record {
                file_path: file_path.to_owned(),
                file_hash,
                label,
                embedding,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_license() {
        // Hash `LICENSE` file, which should be stable enough
        let hash = hash_file(&Path::new("LICENSE")).unwrap();

        assert_eq!(
            hash,
            "3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986"
        );
    }
}
