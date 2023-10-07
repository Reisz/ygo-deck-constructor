run:
    trunk serve --open

lint:
    cargo clippy --workspace -- \
    -W clippy::pedantic \
    -A clippy::missing-errors-doc \
    -A clippy::missing-panics-doc \
    -A clippy::module-name-repetitions \
    -D warnings

test:
    cargo test --workspace

check_fmt:
    cargo fmt --all -- --check
    leptosfmt --check **/*.rs

all_checks: lint test check_fmt

load_images:
    cargo run --release -p data-processor --bin load_images

clean:
    trunk clean --cargo
