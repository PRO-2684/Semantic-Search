//! Utility functions for the semantic search CLI.

use log::info;
use rusqlite::{Connection, OpenFlags, OptionalExtension, Result as SqlResult};
use semantic_search::{embedding::EmbeddingBytes, Embedding};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read, Result as IOResult, Write},
    ops::Deref,
    path::{Path, PathBuf},
};

pub const TABLE_NAME: &str = "files";

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

/// Check if a file is hidden.
fn is_hidden(entry: &Path) -> bool {
    entry
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with('.')
}

/// Returns an iterator of all files in a directory, skipping hidden files.
pub fn iter_files<'a, T1: AsRef<Path>, T2: AsRef<Path> + 'a>(
    dir: T1,
    ref_path: T2,
) -> IOResult<impl Iterator<Item = (PathBuf, String)>> {
    // TODO: Recursive
    let iter = std::fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && !is_hidden(path))
        .map(move |path| {
            let relative = path
                .strip_prefix(ref_path.as_ref())
                .unwrap()
                .to_string_lossy()
                .to_string();
            (path, relative)
        });

    Ok(iter)
}

/// Prompt for user input.
pub fn prompt(message: &str) -> IOResult<String> {
    print!("{message}");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_owned())
}

/// A record in the database.
#[derive(Debug, PartialEq, Clone)]
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
    pub fn open<T: AsRef<Path>>(path: T, read_only: bool) -> SqlResult<Self> {
        let path = path.as_ref();
        let exists = path.exists();
        let conn = if read_only {
            Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?
        } else {
            Connection::open(path)?
        };

        if !exists {
            // Should error when initializing connection
            assert!(!read_only, "Database does not exist");
            info!("Initializing database...");
            Self::init(&conn)?;
        }

        Ok(Self { conn })
    }

    /// Open a database connection in memory for testing.
    #[cfg(test)]
    pub fn dummy() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        Self::init(&conn)?;

        Ok(Self { conn })
    }

    /// Initialize the database.
    fn init(conn: &Connection) -> SqlResult<()> {
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

        Ok(())
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
        let mut blob = self.conn.blob_open(
            rusqlite::DatabaseName::Main,
            TABLE_NAME,
            "embedding",
            row_id,
            false,
        )?;
        let bytes: EmbeddingBytes = record.embedding.into();
        blob.write_at(&bytes, 0)?;

        Ok(())
    }

    /// Get a record from the database.
    pub fn get(&self, file_path: &str) -> SqlResult<Option<Record>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT file_hash, label, embedding FROM {TABLE_NAME} WHERE file_path = ?"
        ))?;

        stmt.query_row([&file_path], |row| {
            let file_hash: String = row.get(0)?;
            let label: String = row.get(1)?;
            let bytes: EmbeddingBytes = row.get(2)?;
            let embedding: Embedding = bytes.into();

            Ok(Record {
                file_path: file_path.to_owned(),
                file_hash,
                label,
                embedding,
            })
        })
        .optional()
    }

    /// Delete a record from the database.
    fn delete(&self, file_path: &str) -> SqlResult<()> {
        self.conn.execute(
            &format!("DELETE FROM {TABLE_NAME} WHERE file_path = ?"),
            [&file_path],
        )?;

        Ok(())
    }

    /// Filter out all satisfying records (path only).
    fn filter<T>(&self, predicate: T) -> Vec<String>
    where
        T: FnMut(&String) -> bool,
    {
        self.conn
            .prepare(&format!("SELECT file_path FROM {TABLE_NAME}"))
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|result| result.unwrap())
            .filter(predicate)
            .collect()
    }

    /// Clean up the database, removing records that no longer exist on disk.
    pub fn clean<T>(&self, ref_path: T) -> SqlResult<usize>
    where
        T: AsRef<Path>,
    {
        let ref_path = ref_path.as_ref();
        let to_delete = self.filter(|path| !ref_path.join(path).exists());
        let count = to_delete.len();

        for path in to_delete {
            self.delete(&path)?;
        }

        Ok(count)
    }
}

impl Deref for Database {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.conn
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

    #[tokio::test]
    async fn test_db() {
        let db = Database::dummy().unwrap();
        let mut record = Record {
            file_path: "test_file_path".to_owned(),
            file_hash: "test_file_hash".to_owned(),
            label: "test_label".to_owned(),
            embedding: Embedding::default(),
        };
        let record2 = Record {
            file_path: "test_file_path2".to_owned(),
            file_hash: "test_file_hash2".to_owned(),
            label: "test_label2".to_owned(),
            embedding: Embedding::from([2.3; 1024]),
        };

        // Insert record
        db.insert(record.clone()).unwrap();
        db.insert(record2.clone()).unwrap();
        let result = db.get(&record.file_path).unwrap().unwrap();
        assert_eq!(result, record);
        let result = db.get(&record2.file_path).unwrap().unwrap();
        assert_eq!(result, record2);

        // Update record
        record.label = "new_label".to_owned();
        record.embedding = Embedding::from([1.2; 1024]);
        db.insert(record.clone()).unwrap();
        let result = db.get(&record.file_path).unwrap().unwrap();
        assert_eq!(result, record);
        let result = db.get(&record2.file_path).unwrap().unwrap();
        assert_eq!(result, record2);

        // Delete record
        db.delete(&record.file_path).unwrap();
        let result = db.get(&record.file_path).unwrap();
        assert_eq!(result, None);
        let result = db.get(&record2.file_path).unwrap().unwrap();
        assert_eq!(result, record2);
    }
}
