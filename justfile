set dotenv-load := false

target := "wasm32-wasip1"

default:
    @just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

check:
    cargo check

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test

dev-wasm:
    cargo build --target {{target}}

build-wasm:
    cargo build --release --target {{target}}

verify: fmt-check clippy test build-wasm

dev-plugin:
    cargo build --target {{target}}
    zellij plugin -s --floating --width 25% -- file:{{justfile_directory()}}/target/{{target}}/debug/zellij-agent-beacon.wasm
