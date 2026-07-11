#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$repo_root"

target="wasm32-wasip1"

if [[ ! -f Cargo.toml ]]; then
  echo "error: Cargo.toml not found at $repo_root" >&2
  echo "This script expects to run from the zellij-agent-beacon Rust workspace." >&2
  exit 1
fi

if command -v rustup >/dev/null 2>&1; then
  if ! rustup target list --installed | grep -qx "$target"; then
    echo "error: Rust target '$target' is not installed." >&2
    echo "Install it with: rustup target add $target" >&2
    exit 1
  fi
fi

cargo build --release --target "$target"

artifact_dir="$repo_root/target/$target/release"
echo "release artifacts written under: $artifact_dir"
