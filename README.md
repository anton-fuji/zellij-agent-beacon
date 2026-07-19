# zellij-agent-beacon

A Zellij plugin for monitoring and navigating AI coding agents running in Zellij panes.

## Development

Requirements:

- Rust with the `wasm32-wasip1` target
- Zellij
- `just`

Install the WASM target once:

```bash
rustup target add wasm32-wasip1
```

Run the normal checks:

```bash
just verify
```

## Running Locally

For fast plugin checks, run it as a floating plugin pane:

```bash
just dev-plugin
```

This uses `--skip-plugin-cache`, so it is useful when checking UI changes quickly.

For a quick left-sidebar layout check in the current Zellij session:

```bash
just dev-layout
```

When run inside an existing Zellij session, this adds the layout as a new tab. It uses the keybindings already loaded by that session.

For interactive key testing, start a fresh development session:

```bash
just dev-session
```

This starts the plugin in the left sidebar with an empty shell on the right. The
agent list will stay empty until `codex`, `claude`, or `opencode` is running in a
terminal pane.

For UI-only testing without starting a real agent:

```bash
just dev-session-mock
```

This uses a generated `zellij.mock.kdl` layout whose right pane is named `codex`,
so the sidebar should show one title-detected Codex entry immediately.

This generates both local files:

- `zellij.kdl`: left-sidebar layout with the local WASM path
- `zellij.mock.kdl`: left-sidebar layout with a mock Codex pane for UI checks
- `.zellij-dev/config.kdl`: development keybinds for `Ctrl p` pane mode

To inspect or edit the generated layout without starting Zellij:

```bash
just init-layout
just init-mock-layout
just init-dev-config
```

`zellij.kdl`, `zellij.mock.kdl`, and `.zellij-dev/` are local-only and ignored by git. The committed layout template is `zellij.kdl.example`.

Running-command detection through Zellij's `get_pane_running_command` API is disabled by default because it can time out during startup in some sessions. The MVP still detects agents from pane command/title metadata.

## Controls

- `j` / Down: select next agent
- `k` / Up: select previous agent
- Enter: focus the selected agent pane
- `c`: toggle compact mode
- `d`: toggle diagnostics
- `h`: hide the plugin pane
- `q`: close the plugin pane
- `r`: manually scan running pane commands
- `?`: toggle help

Direct controls only work when the plugin pane receives keyboard input. In Zellij `pane` mode, Zellij usually captures `j`, `k`, arrow keys, and `q` first.

`just dev-session` starts Zellij with development keybinds that map `Ctrl p` pane mode to plugin commands:

- `Ctrl p`, then `j` / Down: select next agent
- `Ctrl p`, then `k` / Up: select previous agent
- `Ctrl p`, then Enter: focus selected agent pane
- `Ctrl p`, then `q`: close the plugin pane
- `Ctrl p`, then `r`: manually scan running pane commands
- `Ctrl p`, then `?`: toggle help

To test the plugin pipe commands directly:

```bash
just zab-next
just zab-previous
just zab-focus
just zab-help
just zab-refresh
just zab-close
```

To add the same keybinds to your normal Zellij config, add this to the `pane` block in `~/.config/zellij/config.kdl`:

```kdl
pane {
    bind "down" {
        MessagePlugin {
            name "zab"
            payload "next"
        }
    }
    bind "j" {
        MessagePlugin {
            name "zab"
            payload "next"
        }
    }
    bind "up" {
        MessagePlugin {
            name "zab"
            payload "previous"
        }
    }
    bind "k" {
        MessagePlugin {
            name "zab"
            payload "previous"
        }
    }
    bind "enter" {
        MessagePlugin {
            name "zab"
            payload "focus"
        }
        SwitchToMode "normal"
    }
    bind "q" {
        MessagePlugin {
            name "zab"
            payload "close"
        }
        SwitchToMode "normal"
    }
    bind "?" {
        MessagePlugin {
            name "zab"
            payload "help"
        }
    }
    bind "r" {
        MessagePlugin {
            name "zab"
            payload "refresh"
        }
    }
}
```

This overrides those keys in Zellij `pane` mode, so keep your normal pane movement bindings elsewhere if you still need them.
