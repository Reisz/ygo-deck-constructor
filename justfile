run:
    trunk serve --open \
    -w index.html \
    -w Trunk.toml \
    -w Cargo.toml \
    -w Cargo.lock \
    -w src \
    -w data-processor

lint *FLAGS:
    cargo clippy --workspace --all-targets {{FLAGS}} -- -D warnings
    cargo machete

test:
    cargo test --workspace

check_fmt:
    cargo fmt --all -- --check
    leptosfmt -q --check .

all_checks: lint test check_fmt

clean:
    trunk clean --cargo
