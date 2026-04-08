build:
    cargo build

build-release:
    cargo build --release

run *ARGS:
    cargo run -- {{ARGS}}

demo:
    cargo run -- --demo

fmt:
    cargo fmt

check:
    cargo fmt --check && cargo clippy -- -D warnings

test:
    cargo test
