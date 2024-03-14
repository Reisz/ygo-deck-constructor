run:
    trunk serve --open \
    -w index.html \
    -w Trunk.toml \
    -w Cargo.toml \
    -w Cargo.lock \
    -w src \
    -w data-processor

lint *FLAGS:
    cargo clippy --workspace --all-targets {{FLAGS}} -- \
    -W clippy::pedantic \
    -A clippy::empty-docs \
    -A clippy::missing-errors-doc \
    -A clippy::missing-panics-doc \
    -A clippy::module-name-repetitions \
    -D warnings

test:
    cargo test --workspace

check_fmt:
    cargo fmt --all -- --check
    leptosfmt -q --check **/*.rs

all_checks: lint test check_fmt

clean:
    trunk clean --cargo
