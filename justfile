alias b := build
# alias v := bump
alias r := run
alias d := debug
alias t := test

# Build release binary
[doc("\u{001b}[4mB\u{001b}[24muild release binary")]
build:
    cargo build --release --bin sense

# Bump version
# [doc("Bump \u{001b}[4mv\u{001b}[24mersion")]
# bump:
#     ./scripts/bump-version.sh

# Compile and run
[doc("Compile and \u{001b}[4mr\u{001b}[24mun")]
run *args:
    cd tests && cargo run -- {{args}}

# Compile and run (with debug logging)
[doc("Compile and run (with \u{001b}[4md\u{001b}[24mebug logging)")]
debug *args:
    RUST_LOG=DEBUG cargo run -- {{args}}

# Tests
[doc("\u{001b}[4mT\u{001b}[24mests")]
test *args:
    cargo test {{args}}
