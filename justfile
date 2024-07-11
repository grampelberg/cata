normal := '\033[0m'
red := '\033[31m'
green := '\033[32m'
yellow := '\033[33m'

tools:
    mise install

check: fmt-check lint audit workspace-dependencies

audit:
    cargo audit

fmt-check:
    cargo +nightly fmt --all --check
    just --fmt --unstable --check

lint:
    CARGO_BUILD_RUSTFLAGS="-Dwarnings" cargo clippy

workspace-dependencies:
    cargo autoinherit 2>/dev/null
    @test -z "$(git status --porcelain)" || \
        ( \
            echo "{{ red }}run 'cargo autoinherit' to tidy dependencies{{ normal }}" && exit 1 \
        )

test:
    cargo test
