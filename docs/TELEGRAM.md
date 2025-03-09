# Deploying Telegram bot

## Creating a bot

1. Go to [@BotFather](https://t.me/BotFather) and create a bot.
2. Enable [Inline mode](https://core.telegram.org/bots/inline) by sending `/setinline` command to BotFather and following the instructions.
3. Customize name, desc etc. as u wish.

## Setting up environment

TBD

Note that for best results, all files must be of .JP(E)G, .PNG or .WEBP format. The program would error on images with incorrect extensions. You can use [`fix-ext.py`](../scripts/fix-ext.py) to fix extensions automatically:

```bash
python scripts/fix-ext.py /path/to/images/
```

## Configuration

TBD

## Running

TBD

## Modification

Each time you changed files, you should run `sense index` to re-index for the changes to take effect on the bot.
