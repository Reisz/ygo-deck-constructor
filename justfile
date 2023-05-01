lint:
    cargo clippy --workspace -- -W clippy::pedantic -A clippy::missing-errors-doc -A clippy::module-name-repetitions -D warnings

test:
    cargo test --workspace

run:
    trunk serve --open

clean:
    trunk clean --cargo
