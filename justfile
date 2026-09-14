check: fmt-check clippy test

fmt:
    cargo fmt

fmt-check:
    cargo fmt --check

clippy:
    cargo clippy --all-targets -- -D warnings

test:
    cargo test

regenerate:
    cargo run --manifest-path tools/regen/Cargo.toml
    cargo fmt

publish-check:
    cargo publish --dry-run
