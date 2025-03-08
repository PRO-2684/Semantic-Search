alias v := bump
alias r := run
alias s := search
alias d := debug
alias t := test

# Bump version
[doc("Bump \u{001b}[4mv\u{001b}[24mersion")]
bump:
    ./scripts/bump-version.sh

# Compile and run
[doc("Compile and \u{001b}[4mr\u{001b}[24mun")]
run *args:
    cd tests && cargo run -- {{args}}

# Compile and run search
[doc("Compile and run \u{001b}[4mS\u{001b}[24mearch")]
search term:
    cd tests && cargo run -- search "{{term}}"

# Compile and run Telegram bot with proxy (http_proxy, https_proxy)
tg *args:
    cd tests && http_proxy="http://127.0.0.1:7890" https_proxy="http://127.0.0.1:7890" cargo run -- tg {{args}}

# Compile and run (with debug logging)
[doc("Compile and run (with \u{001b}[4md\u{001b}[24mebug logging)")]
debug *args:
    RUST_LOG=DEBUG cargo run -- {{args}}

# Tests
[doc("\u{001b}[4mT\u{001b}[24mests")]
test *args:
    cargo test {{args}}
