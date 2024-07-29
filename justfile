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

check:
    cargo clippy --workspace --all-targets -- -D warnings
    cargo machete
    cargo nextest run --workspace
    cargo fmt --all -- --check
    leptosfmt -q --check .

clean:
    trunk clean --cargo
