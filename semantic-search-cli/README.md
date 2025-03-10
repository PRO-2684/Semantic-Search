# Semantic Search

🔎 Semantic search cli.

## Setup

### Installation

There are two ways to install the CLI:

- From source: `cargo install semantic-search-cli`
- From binary: Download respective release from the Releases page, or use `cargo binstall semantic-search-cli`

### Configuration

The configuration file is located at `.sense/config.toml`. You should create this file if it does not exist.

#### Sample Configuration

```toml
[api]
key = "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" # API key for SiliconCloud (Required)
model = "BAAI/bge-large-zh-v1.5" # Model to use for embedding (Optional)

[bot] # Only required for `sense bot`
token = "1234567890:xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx" # Telegram bot token (Required)
owner = 1234567890 # Telegram user ID of the bot owner (Required)
whitelist = [] # Whitelisted user IDs (Optional)
sticker_set = "meme" # Sticker set id prefix for the bot (Optional, an additional `_by_<bot_username>` will be appended to form the full sticker set id)
num_results = 8 # Number of results to return (Optional)

[server]
port = 8080 # Default port for the server (Optional)
```

#### API Configuration (`[api]` section)

- `api.key`: Required. API key for SiliconCloud. You can get one from [SiliconCloud](https://cloud.siliconflow.cn/account/ak) for free.
- `api.model`: Optional. Model to use for embedding. Available models:
    - [`BAAI/bge-large-zh-v1.5`](https://cloud.siliconflow.cn/open/models?target=BAAI/bge-large-zh-v1.5) (Default)
    - [`BAAI/bge-large-en-v1.5`](https://cloud.siliconflow.cn/open/models?target=BAAI/bge-large-en-v1.5)
    - [`netease-youdao/bce-embedding-base_v1`](https://cloud.siliconflow.cn/open/models?target=netease-youdao/bce-embedding-base_v1)
    - [`BAAI/bge-m3`](https://cloud.siliconflow.cn/open/models?target=BAAI/bge-m3)
    - [`Pro/BAAI/bge-m3`](https://cloud.siliconflow.cn/open/models?target=Pro/BAAI/bge-m3)

#### Telegram Bot Configuration (`[bot]` section)

This section is only required if you want to deploy the Telegram bot (`sense bot`).

- `bot.token`: Required. Telegram bot token. You can get one from [BotFather](https://t.me/BotFather).
- `bot.owner`: Required. Telegram user ID of the bot owner. You can get your user ID from [IDBot](https://t.me/myidbot).
- `bot.whitelist`: Optional. Whitelisted user IDs. Only these users can use the bot. If not set or set to an empty array, all users can use the bot.
- `bot.sticker_set`: Optional. Sticker set id prefix for the bot. An additional `_by_<bot_username>` will be appended to form the full sticker set id, as required by Telegram. If not set, the bot will not create any sticker set.
- `bot.num_results`: Optional. Number of results to return. Default is 8.

#### (TBD) Server Configuration (`[server]` section)

TBD

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
sense search "cute cat" -n 8 # Default
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
    - [x] Support `@` in commands
    - [ ] Graceful shutdown
    - [x] Parallel processing
    - [x] Empty sticker set every 120 stickers
- [x] Auto release binary
- [ ] Improve indexing performance
