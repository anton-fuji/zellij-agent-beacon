# Zellij API Notes

These notes are repository guidance, not a replacement for the installed `zellij-tile` crate docs. When code and these notes disagree, inspect the crate version in `Cargo.lock` or local docs and follow the actual API.

## Plugin boundaries

- Build plugins against the official `zellij-tile` SDK.
- Target `wasm32-wasip1` for release builds.
- Avoid background daemons or external process management for the MVP.
- Keep state inside plugin state structures and update it from Zellij events.

## Pane and tab handling

- Treat pane metadata as volatile. A pane may disappear, exit, move tab, or become unavailable between detection, rendering, and focus.
- Do not unwrap pane lookups. Missing panes should render as unavailable or be filtered safely.
- Preserve enough identity to focus a pane when possible, normally pane ID plus tab context when the API exposes it.
- Include tab or pane location in the displayed agent row only when it is known.
- Floating and hidden panes should not crash rendering or navigation.

## Agent detection

- Keep detection logic in its own module or function boundary.
- Prefer process command/name metadata exposed by Zellij over terminal output.
- Detect only implemented agents. For the MVP, expected agent names are Codex, Claude Code, and OpenCode.
- Matching should be conservative and explainable. Avoid broad substring checks that can classify unrelated commands.
- Do not claim support for permission prompts, token usage, session restore, restart, or termination.

## UI rendering

- Sidebar rendering should tolerate an empty agent list.
- Selected index must be clamped or cleared when the list changes.
- Rows should distinguish active, unavailable, and exited panes if that state is available.
- Keep display text compact; this is a sidebar, not a full dashboard.

## Focus behavior

- Focus only when the selected agent has a valid pane target.
- If a pane is missing or exited, ignore the focus request or show a safe status message.
- Avoid changing tabs or panes speculatively when the target cannot be identified.
