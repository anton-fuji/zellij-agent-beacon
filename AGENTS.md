# AGENTS.md

## Project

`zellij-agent-beacon` is a Zellij plugin for monitoring and navigating AI coding agents running in Zellij panes.

The plugin is written in Rust using the official `zellij-tile` SDK and compiled to `wasm32-wasip1`.

## Current goal

Build an MVP that:

* Detects Codex, Claude Code, and OpenCode panes
* Displays detected agents in a left sidebar
* Shows the pane or tab where each agent is running
* Allows users to select an agent and focus its pane
* Handles empty, missing, and exited panes safely

## Scope

Keep the MVP simple.

Do not implement these features unless explicitly requested:

* Terminal-output parsing
* Permission-prompt detection
* Token usage tracking
* Session persistence
* Agent restart or termination
* Background daemons
* MCP integration

## Development rules

* Use official Zellij APIs.
* Keep agent detection separate from UI rendering.
* Avoid unnecessary dependencies.
* Do not use `unsafe` without a documented reason.
* Do not modify unrelated files.
* Do not commit files under `target/`.
* Do not claim a feature is supported unless it is implemented and tested.

## Validation

Before completing a code change, run the relevant checks:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release --target wasm32-wasip1
```

Report which commands were actually executed and any remaining limitations.

## Repository guidance

Use the repository skills when the task matches them:

* Zellij plugin development
* AI agent pane detection
* WASM release preparation
