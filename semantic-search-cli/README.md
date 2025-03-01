# Semantic Search

🔎 Semantic search cli.

## Setup

### Installation

You can compile the program from source by executing:

```bash
cargo build --release
```

The compiled binary will be located at `./target/release/sense`. You can move it to a directory in your `PATH` to use it globally.

### Configuration

Grab an API key from [SiliconCloud](https://cloud.siliconflow.cn/account/ak) (free) and set it in the configuration file. The configuration file is located at `.sense/config.toml` and should look like this:

```toml
[server]
port = 8080 # Default port for the server (Optional)

[api]
key = "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" # API key for SiliconCloud (Required)
model = "BAAI/bge-large-zh-v1.5" # Model to use for embedding (Optional)
# Available models: BAAI/bge-large-zh-v1.5, BAAI/bge-large-en-v1.5, netease-youdao/bce-embedding-base_v1, BAAI/bge-m3, Pro/BAAI/bge-m3
```

### Indexing

(Re-)Index the files you want to search for by executing the following command:

```bash
sense index
```

This will generate or update index of the files, their hashes, labels and embeddings in `.sense/index.db`. Note that each time you add or remove files, you need to re-run this process.

If files are created or changed, running this command will prompt you to label them (again). You can use any tool of your choice to label them automatically. See [DEV.md](../docs/DEV.md) for more information on the database schema.

## Usage

### Command Line Interface

To search for files based on labels, execute the following command:

```bash
sense search "cute cat"
```

You can specify how many results you want to display using the `--limit` flag:

```bash
sense search "cute cat" --limit 5 # Default
```

Also, you can specify the extension of the files you want to search for using the `--ext` flag:

```bash
sense search "cute cat" --ext "jpg"
```

### Server

You can start a server to search for files using a REST API:

```bash
sense serve --port 8080
```

Which will start a server on port 8080. You can then search for files using the following endpoint:

```bash
$ curl -X POST http://localhost:8080/search -d '{"query": "cute cat", "limit": 5, "ext": "jpg"}'
{"files":["cute-cat.jpg","cute-cat-2.jpg","cute-cat-3.jpg","cute-cat-4.jpg","cute-cat-5.jpg"]}
```

## ☑️ TODOs

- [ ] Implement aforementioned features
    - [x] Indexing
    - [ ] Labeling
    - [ ] Embedding
    - [ ] Searching
    - [ ] Server
- [ ] Incremental update
