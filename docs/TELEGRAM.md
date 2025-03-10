# Deploying Telegram bot

## Setup

See [README of `semantic-search-cli`](../semantic-search-cli/README.md#setup) for instructions on installation, configuration and indexing.

Note that for best results, all files must be of .JP(E)G, .PNG or .WEBP format. The program would ignore files with other extensions, and error on images with incorrect extensions. You can use [`fix-ext.py`](../scripts/fix-ext.py) to fix extensions automatically:

```bash
python scripts/fix-ext.py /path/to/images/
```

A set of indexed sample images is provided [here](https://github.com/PRO-2684/Semantic-Search/releases/download/v0.1.3/sample.tar.gz) for you to try out. Configure `.sense/config.toml` and rename `.sense/empty-file_id.db3` to `.sense/index.db3` before running the bot.

## Creating a bot

1. Go to [@BotFather](https://t.me/BotFather) and create a bot.
2. Enable [Inline mode](https://core.telegram.org/bots/inline) by sending `/setinline` command to BotFather and following the instructions.
3. Customize name, desc etc. as u wish.

## Running

```bash
sense tg
```

## Modification

Each time you changed files, you should run `sense index` to re-index and restart your bot for the changes to take effect.
