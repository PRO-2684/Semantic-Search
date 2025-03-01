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
