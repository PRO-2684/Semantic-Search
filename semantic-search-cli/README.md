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

[bot] # Only required for `sense bot`
token = "1234567890:xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" # Telegram bot token (Required)
owner = 1234567890 # Telegram user ID of the bot owner (Required)
whitelist = [] # Whitelisted user IDs - Only these users can use the bot (Optional, all users can use the bot if not set or set to an empty array)
sticker_set = "meme" # Sticker set id prefix for the bot (Optional, an additional `_by_<bot_username>` will be appended to form the full sticker set id)
num_results = 5 # Number of results to return (Optional)
```

### Indexing

(Re-)Index the files you want to search for by executing the following command:

```bash
sense index
```

This will generate or update index of the files, their hashes, labels and embeddings in `.sense/index.db3`. Note that each time you add or remove files, you need to re-run this process.

If files are created or changed, running this command will prompt you to label them (again). You can use any tool of your choice to label them automatically. See [DEV.md](../docs/DEV.md) for more information on the database schema.

## Usage

### Command Line Interface

To search for files based on labels, execute the following command:

```bash
sense search "cute cat"
```

You can specify how many results you want to display using the `--num-results` flag, or `-n` for short:

```bash
sense search "cute cat" -n 5 # Default
```

(TBD) Also, you can specify the regular expression for the path of the files using the `--path` flag, or `-p` for short:

```bash
sense search "cute cat" -p "path/.*\.jpg"
```

### Telegram Bot

You can start a Telegram bot to search for files using a chat interface:

```bash
sense tg
```

See [TELEGRAM.md](../docs/TELEGRAM.md) for detailed instructions on deploying the bot.

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
    - [x] Labeling
    - [x] Embedding
    - [x] Searching
    - [x] Telegram bot
    - [ ] Server
- [x] Incremental update
- [ ] Better error handling
    - [ ] Reduce using of `unwrap()`, so as to make the program more robust to network failures
    - [ ] Error logging
- [ ] Reducing clone
- [ ] Enhance Telegram bot
    - [ ] Graceful shutdown
    - [ ] Multi-threading
- [ ] Auto release binary
