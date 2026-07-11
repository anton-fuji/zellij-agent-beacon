#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$repo_root"

if [[ ! -f Cargo.toml ]]; then
  echo "error: Cargo.toml not found at $repo_root" >&2
  echo "This hook expects to run from the zellij-agent-beacon Rust workspace." >&2
  exit 1
fi

cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
