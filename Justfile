set dotenv-load
set positional-arguments

run *ARGS:
    cargo run -- "$@"

test *ARGS:
    cargo test -- "$@"

build:
    cargo build

check:
    cargo check

publish:
    cargo publish