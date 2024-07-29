# NOTE: Using the additive list here, because ignoring is too slow 
serve *FLAGS:
    trunk serve {{FLAGS}} \
    -w index.html \
    -w Trunk.toml \
    -w Cargo.toml \
    -w Cargo.lock \
    -w src \
    -w common \
    -w data-processor

lint *FLAGS:
    cargo clippy --workspace --all-targets {{FLAGS}} -- -D warnings
    cargo machete

test:
    cargo nextest run --workspace

check_fmt:
    cargo fmt --all -- --check
    leptosfmt -q --check .

all_checks: lint test check_fmt

clean:
    trunk clean --cargo
