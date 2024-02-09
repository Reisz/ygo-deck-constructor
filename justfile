run:
    trunk serve --open \
    -w index.html \
    -w Trunk.toml \
    -w Cargo.toml \
    -w Cargo.lock \
    -w data \
    -w src \
    -w data-processor

lint *FLAGS:
    cargo clippy --workspace {{FLAGS}} -- \
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
