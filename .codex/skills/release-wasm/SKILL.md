---
name: release-wasm
description: Use when preparing, validating, or troubleshooting a release build of this repository's Zellij plugin for the wasm32-wasip1 target, including release artifact checks and build script usage.
---

# Release WASM

Use this skill when preparing a WASM release build for `zellij-agent-beacon`.

## Workflow

1. Read `AGENTS.md`.
2. Confirm the Rust project has a `Cargo.toml`.
3. Run the Rust checks before release when code changed:

```bash
.codex/hooks/check-rust.sh
```

4. Build the release artifact:

```bash
.codex/skills/release-wasm/scripts/build-release.sh
```

5. Report the artifact path and the commands that actually ran.

## Constraints

- Do not commit or package files under `target/` unless explicitly requested.
- Do not add dependencies just for release packaging.
- Do not claim the plugin is release-ready if formatting, clippy, tests, or the WASM build failed.
