lint:
    cargo clippy --workspace -- -W clippy::pedantic -D warnings

test:
    cargo test --workspace
