# Dev Notes

## Running ignored tests

To run ignored tests, an API key must be provided in the `SILICONFLOW_API_KEY` environment variable:

```bash
export SILICONFLOW_API_KEY=sk-1234567890abcdef1234567890abcdef1234567890abcdef
```

Then run the tests with the `--ignored` flag.

```bash
cargo test -- --ignored
```

Alternatively, provide the API key inline:

```bash
SILICONFLOW_API_KEY=sk-1234567890abcdef1234567890abcdef1234567890abcdef cargo test -- --ignored
```

## `.sense` directory structure

- `config.toml`: Configuration file for the CLI.
- `index.db3`: SQLite database containing the index of files and generated embeddings. The schema is as follows:

```sql
CREATE TABLE files (
    file_path TEXT PRIMARY KEY,  -- Unique file identifier
    file_hash TEXT NOT NULL,     -- Hash of the file contents
    label TEXT NOT NULL,         -- Label of the file
    embedding BLOB NOT NULL      -- 4KB binary data (embedding)
);
```
