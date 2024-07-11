tools:
    mise install

lint:
    CARGO_BUILD_RUSTFLAGS="-Dwarnings" cargo clippy

test:
    cargo test

cargo:
    cargo autoinherit
    cargo ws version
