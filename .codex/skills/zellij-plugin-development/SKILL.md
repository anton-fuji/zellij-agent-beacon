---
name: zellij-plugin-development
description: Use when developing this repository's Rust Zellij plugin, especially work involving zellij-tile APIs, pane and tab metadata, agent pane detection, sidebar rendering, focus navigation, or MVP safety behavior for missing and exited panes.
---

# Zellij Plugin Development

Use this skill for changes to the `zellij-agent-beacon` plugin implementation.

## Workflow

1. Read `AGENTS.md` before changing code.
2. Keep agent detection separate from UI rendering.
3. Use official `zellij-tile` APIs and verify exact names against the installed crate version.
4. Keep the MVP limited to detecting Codex, Claude Code, and OpenCode panes, displaying them, and focusing their panes.
5. Handle empty, missing, hidden, floating, and exited panes without panics.
6. Do not add terminal-output parsing, permission-prompt detection, token tracking, persistence, restart, termination, daemons, or MCP integration unless explicitly requested.

## Zellij API Notes

Before implementing or reviewing Zellij API interactions, read `references/zellij-api-notes.md`.

## Validation

For Rust-only checks, run:

```bash
.codex/hooks/check-rust.sh
```

For WASM release validation, run:

```bash
.codex/hooks/check-wasm.sh
```

When finishing code changes, report the exact commands run and any checks skipped or blocked.
