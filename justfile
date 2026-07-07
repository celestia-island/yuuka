import "./celestia-devtools.just"

set shell := ["bash", "-c"]

default:
    @just --list

fmt:
    just fmt-markdown .
    cargo +nightly fmt --all
    cargo clippy -- -D warnings

fmt-check:
    just fmt-markdown . --check
    cargo +nightly fmt --all -- --check --unstable-features

check:
    cargo check

test:
    cargo test

build:
    just cache-guard
    cargo build

clean:
    cargo clean

ci: fmt-check check test
