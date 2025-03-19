//! Utility functions for the semantic search CLI.

use futures_core::stream::BoxStream;
use futures_util::stream::StreamExt;
use log::info;
use semantic_search::{Embedding, embedding::EmbeddingBytes};
use sha2::{Digest, Sha256};
use sqlx::{
    Connection, Executor, Result as SqlResult, Row, SqliteConnection, sqlite::SqliteConnectOptions,
};
use std::{
    fs::File,
    io::{self, Read, Result as IOResult, Write},
    iter,
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

/// Iterate over all files in a directory recursively, skipping hidden files.
pub fn iter_files<'a, T1: AsRef<Path>>(
    dir: T1,
    ref_path: &'a Path,
) -> Box<dyn Iterator<Item = (PathBuf, String)> + 'a> {
    let iter = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if is_hidden(&path) { None } else { Some(path) }
        })
        .flat_map(move |path| {
            if path.is_dir() {
                iter_files(&path, ref_path)
            } else {
                let relative = path
                    .strip_prefix(ref_path)
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                Box::new(iter::once((path, relative)))
            }
        });

    Box::new(iter)
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
#[derive(Debug, PartialEq, Clone, sqlx::FromRow)]
pub struct Record {
    /// Path to the file (relative to working directory)
    pub file_path: String,
    /// SHA-256 hash of the file
    pub file_hash: String,
    /// File id used in Telegram
    pub file_id: Option<String>,
    /// Label of the file
    pub label: String,
    /// Embedding of the file
    #[sqlx(try_from = "Vec<u8>")]
    pub embedding: Embedding,
}

/// Simple database wrapper.
pub struct Database {
    conn: SqliteConnection,
}

impl Database {
    /// Open a database connection, creating if not exists.
    #[allow(clippy::future_not_send, reason = "Should be `Send` if `T: Send`")]
    pub async fn open<T: AsRef<Path>>(path: T, read_only: bool) -> SqlResult<Self> {
        let path = path.as_ref();
        let exists = path.exists();
        let options = SqliteConnectOptions::new()
            .filename(path)
            .read_only(read_only)
            .create_if_missing(!exists);
        let mut conn = SqliteConnection::connect_with(&options).await?;

        if !exists {
            // Should error when initializing connection
            assert!(!read_only, "Database does not exist");
            info!("Initializing database...");
            Self::init(&mut conn).await?;
        }

        Ok(Self { conn })
    }

    /// Open a database connection in memory for testing.
    #[cfg(test)]
    pub async fn dummy() -> SqlResult<Self> {
        let mut conn = SqliteConnection::connect("sqlite::memory:").await?;
        Self::init(&mut conn).await?;

        Ok(Self { conn })
    }

    /// Initialize the database.
    async fn init(conn: &mut SqliteConnection) -> SqlResult<()> {
        let query = format!(
            "CREATE TABLE IF NOT EXISTS {TABLE_NAME} (
            file_path TEXT PRIMARY KEY,
            file_hash TEXT NOT NULL,
            file_id TEXT,
            label TEXT NOT NULL,
            embedding BLOB NOT NULL
            )"
        );
        conn.execute(query.as_str()).await?;

        Ok(())
    }

    /// Insert a record into the database, replacing if exists.
    pub async fn insert(&mut self, record: Record) -> SqlResult<bool> {
        let bytes: EmbeddingBytes = record.embedding.into();
        let query = format!(
            "INSERT OR REPLACE INTO {TABLE_NAME} (file_path, file_hash, file_id, label, embedding) VALUES (?, ?, ?, ?, ?)"
        );
        let query = sqlx::query(query.as_str());
        let result = query
            .bind(&record.file_path)
            .bind(&record.file_hash)
            .bind(&record.file_id)
            .bind(&record.label)
            .bind(&bytes[..])
            .execute(&mut self.conn)
            .await?;

        Ok(result.rows_affected() == 1)
    }

    /// Get a record from the database.
    pub async fn get(&mut self, file_path: &str) -> SqlResult<Option<Record>> {
        let query = format!(
            "SELECT file_path, file_hash, file_id, label, embedding FROM {TABLE_NAME} WHERE file_path = ?"
        );
        let query = sqlx::query_as::<_, Record>(query.as_str());
        let result = query.bind(file_path).fetch_optional(&mut self.conn).await?;

        Ok(result)
    }

    /// Search for the top-N matches, returning the file path and similarity.
    pub async fn search(
        &mut self,
        n: usize,
        embedding: &Embedding,
    ) -> SqlResult<Vec<(String, f32)>> {
        let mut rows = self.iter_embeddings();
        let mut results = Vec::with_capacity(n);

        while let Some(row) = rows.next().await {
            let (file_path, other_embedding) = row?;
            let similarity = embedding.cosine_similarity(&other_embedding);
            // Top N results
            if results.len() < n {
                results.push((file_path, similarity));
            } else if results.last().unwrap().1 < similarity {
                results.pop();
                results.push((file_path, similarity));
            }
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        }

        Ok(results)
    }

    /// Delete a record from the database.
    async fn delete(&mut self, file_path: &str) -> SqlResult<bool> {
        let query = format!("DELETE FROM {TABLE_NAME} WHERE file_path = ?");
        let query = sqlx::query(query.as_str());
        let result = query.bind(file_path).execute(&mut self.conn).await?;

        Ok(result.rows_affected() == 1)
    }

    /// Iterate over all records in the database. (path only)
    #[allow(
        clippy::iter_not_returning_iterator,
        reason = "It returns a stream, also called async iterator"
    )]
    pub fn iter(&mut self) -> BoxStream<'_, SqlResult<String>> {
        let query = sqlx::query(queries::QUERY_PATH);
        let result = query
            .fetch(&mut self.conn)
            .map(|row| {
                let row = row?;
                Ok(row.get(0))
            })
            .boxed();
        result
    }

    /// Iterate over all records in the database, together with embeddings.
    pub fn iter_embeddings(&mut self) -> BoxStream<'_, SqlResult<(String, Embedding)>> {
        let query = sqlx::query(queries::QUERY_EMBEDDING);
        query
            .fetch(&mut self.conn)
            .map(|row| {
                let row = row?;
                let file_path: String = row.get(0);
                let embedding: &[u8] = row.get(1);
                let embedding: Embedding = embedding.try_into().expect("Invalid embedding size");
                Ok((file_path, embedding))
            })
            .boxed()
    }

    /// Retrieve all records' paths without file id.
    pub async fn paths_without_file_ids(&mut self) -> Vec<String> {
        let query = format!("SELECT file_path FROM {TABLE_NAME} WHERE file_id IS NULL");
        let query = sqlx::query(query.as_str());
        query
            .fetch(&mut self.conn)
            .filter_map(|row| async {
                match row {
                    Ok(row) => Some(row.get(0)),
                    Err(e) => {
                        log::error!("Error fetching row: {e}");
                        None
                    }
                }
            })
            .collect()
            .await
    }

    /// Clean up the database, removing records that no longer exist on disk.
    #[allow(clippy::future_not_send, reason = "Should be `Send` if `T: Send`")]
    pub async fn clean<T>(&mut self, ref_path: T) -> SqlResult<usize>
    where
        T: AsRef<Path>,
    {
        let ref_path = ref_path.as_ref();
        let records = self.iter();
        let to_delete: Vec<_> = records
            .filter_map(|path| async {
                let path = path.ok()?;
                let full_path = ref_path.join(&path);
                if full_path.exists() { None } else { Some(path) }
            })
            .collect()
            .await;
        let count = to_delete.len();

        for path in to_delete {
            self.delete(&path).await?;
        }

        Ok(count)
    }

    /// Search for the top-N matches, returning the file path, similarity and file id, ensuring file id exists.
    pub async fn search_with_id(
        &mut self,
        n: usize,
        embedding: &Embedding,
    ) -> SqlResult<Vec<(String, f32, String)>> {
        let query = format!("SELECT file_path, embedding, file_id FROM {TABLE_NAME}");
        let query = sqlx::query(query.as_str());
        let mut rows = query.fetch(&mut self.conn);

        let mut results = Vec::with_capacity(n);
        while let Some(row) = rows.next().await {
            let row = row?;
            let file_path: String = row.get(0);
            let other_embedding: &[u8] = row.get(1);
            let other_embedding: Embedding =
                other_embedding.try_into().expect("Invalid embedding size");
            let similarity = embedding.cosine_similarity(&other_embedding);
            let file_id: String = row.get(2);
            // Top N results
            if results.len() < n {
                results.push((file_path, similarity, file_id));
            } else if results.last().unwrap().1 < similarity {
                results.pop();
                results.push((file_path, similarity, file_id));
            }
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        }

        Ok(results)
    }

    /// Sets file id for a record.
    pub async fn set_file_id(&mut self, file_path: &str, file_id: &str) -> SqlResult<bool> {
        let query = format!("UPDATE {TABLE_NAME} SET file_id = ? WHERE file_path = ?");
        let query = sqlx::query(query.as_str());
        let result = query
            .bind(Some(file_id))
            .bind(file_path)
            .execute(&mut self.conn)
            .await?;

        Ok(result.rows_affected() == 1)
    }

    /// Close the database connection.
    pub async fn close(self) -> SqlResult<()> {
        self.conn.close().await
    }
}

/// Used query instructions.
mod queries {
    pub const QUERY_PATH: &str = "SELECT file_path FROM files";
    pub const QUERY_EMBEDDING: &str = "SELECT file_path, embedding FROM files";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(windows))]
    #[test]
    fn hash_license() {
        // Hash `LICENSE` file, which should be stable enough
        let hash = hash_file(Path::new("../LICENSE")).unwrap();

        assert_eq!(
            hash,
            "3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986"
        );
    }

    #[tokio::test]
    async fn test_db() {
        let mut db = Database::dummy().await.unwrap();
        let mut record = Record {
            file_path: "test_file_path".to_owned(),
            file_hash: "test_file_hash".to_owned(),
            file_id: None,
            label: "test_label".to_owned(),
            embedding: Embedding::default(),
        };
        let record2 = Record {
            file_path: "test_file_path2".to_owned(),
            file_hash: "test_file_hash2".to_owned(),
            file_id: None,
            label: "test_label2".to_owned(),
            embedding: Embedding::from([2.3; 1024]),
        };

        // Insert record
        db.insert(record.clone()).await.unwrap();
        db.insert(record2.clone()).await.unwrap();
        let result = db.get(&record.file_path).await.unwrap().unwrap();
        assert_eq!(result, record);
        let result = db.get(&record2.file_path).await.unwrap().unwrap();
        assert_eq!(result, record2);

        // Update record
        record.label = "new_label".to_owned();
        record.embedding = Embedding::from([1.2; 1024]);
        db.insert(record.clone()).await.unwrap();
        let result = db.get(&record.file_path).await.unwrap().unwrap();
        assert_eq!(result, record);
        let result = db.get(&record2.file_path).await.unwrap().unwrap();
        assert_eq!(result, record2);

        // Delete record
        db.delete(&record.file_path).await.unwrap();
        let result = db.get(&record.file_path).await.unwrap();
        assert_eq!(result, None);
        let result = db.get(&record2.file_path).await.unwrap().unwrap();
        assert_eq!(result, record2);
    }
}
