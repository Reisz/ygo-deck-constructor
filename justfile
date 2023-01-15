lint:
    cargo clippy --workspace -- -W clippy::pedantic -A clippy::module-name-repetitions -D warnings

test:
    cargo test --workspace

run:
    trunk serve --open
