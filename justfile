lint:
    CARGO_BUILD_RUSTFLAGS="-Dwarnings" cargo clippy

test:
    cargo test
