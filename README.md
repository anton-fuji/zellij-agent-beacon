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

When run inside an existing Zellij session, this adds the layout with `zellij action new-tab --layout`. If run from a normal terminal, it starts Zellij with that layout. If you are already working inside Zellij, use this instead of `just dev-session`.

For the same check with a mock Codex pane:

```bash
just dev-layout-mock
```

If the full layout path is unstable in your current session, use the smallest plugin-only check:

```bash
just dev-floating
```

This launches or focuses the plugin as a floating pane in the current Zellij session.

For interactive key testing with the repository's isolated development keybinds,
start a fresh local-config session:

```bash
just dev-session-local
```

Run this from a normal terminal, not from inside an existing Zellij session. The
recipe refuses nested Zellij sessions because they can leave the screen blank or
confuse terminal input modes.

To test the plugin with your personal Zellij configuration instead, use:

```bash
just dev-session-personal
```

This explicitly reads `~/.config/zellij`. It does not load the repository's
development keybinds, so only the keys configured in your personal config apply.

This starts the plugin in the left sidebar with an empty shell on the right. The
agent list will stay empty until `codex`, `claude`, or `opencode` is running in a
terminal pane.

For UI-only testing without starting a real agent:

```bash
just dev-session-local-mock
```

This uses a generated `zellij.mock.kdl` layout whose right pane is named `codex`,
so the sidebar should show one title-detected Codex entry immediately.

Use your personal configuration with the same mock layout via:

```bash
just dev-session-personal-mock
```

This generates both local files:

- `zellij.kdl`: left-sidebar layout with the local WASM path
- `zellij.mock.kdl`: left-sidebar layout with a mock Codex pane for UI checks
- `.zellij-dev/config.kdl`: isolated development keybinds for `Ctrl p` pane mode

To inspect or edit the generated layout without starting Zellij:

```bash
just init-layout
just init-mock-layout
just init-dev-config
```

`zellij.kdl`, `zellij.mock.kdl`, and `.zellij-dev/` are local-only and ignored by git. The committed layout template is `zellij.kdl.example`.

Running-command detection through Zellij's `get_pane_running_command` API is disabled by default because it can time out during startup in some sessions. The MVP still detects agents from pane command/title metadata. Pressing `r` creates a command-detection snapshot; press it again after an agent starts, exits, or changes command, because the snapshot can become stale.

## Personal Launch Key

To launch or focus the plugin with a personal keybinding, generate a local snippet:

```bash
just init-launch-keybind
```

This writes `.zellij-dev/launch-keybind.kdl` with this default binding:

```kdl
shared_except "locked" {
    bind "Alt a" {
        LaunchOrFocusPlugin "file:/absolute/path/to/target/wasm32-wasip1/debug/zellij-agent-beacon.wasm" {
            floating true
            move_to_focused_tab true
            skip_plugin_cache true
        }
        SwitchToMode "Normal"
    }
    bind "Alt ." {
        MessagePlugin {
            name "zab"
            payload "next"
        }
    }
    bind "Alt ," {
        MessagePlugin {
            name "zab"
            payload "previous"
        }
    }
    bind "Alt Enter" {
        MessagePlugin {
            name "zab"
            payload "focus"
        }
        SwitchToMode "Normal"
    }
    bind "Alt r" {
        MessagePlugin {
            name "zab"
            payload "refresh"
        }
    }
    bind "Alt q" {
        MessagePlugin {
            name "zab"
            payload "close"
        }
        SwitchToMode "Normal"
    }
    bind "Alt ?" {
        MessagePlugin {
            name "zab"
            payload "help"
        }
    }
}
```

`Alt a` is intentionally chosen because it is not used in the default keybindings checked for this project. Do not bind this to `Ctrl p`; that is Zellij's default pane mode prefix.

For your personal Zellij config, copy the generated `shared_except "locked"` block into the existing `keybinds` block in `~/.config/zellij/config.kdl`. This launches the plugin as a floating pane. Use a layout such as `zellij.kdl.example` when you want it permanently embedded as a left sidebar.

Personal key summary:

- `Alt a`: launch or focus the plugin
- `Alt .`: select next agent
- `Alt ,`: select previous agent
- `Alt Enter`: focus selected agent pane
- `Alt r`: manually scan running pane commands
- `Alt q`: close the plugin pane
- `Alt ?`: toggle help

## Controls

- `j` / Down: select next available agent
- `k` / Up: select previous available agent
- Enter: request focus for the selected agent pane
- `c`: toggle compact mode
- `d`: toggle diagnostics
- `h`: hide the plugin pane
- `q`: close the plugin pane
- `r`: manually scan running pane commands
- `?`: toggle help

Direct controls only work when the plugin pane receives keyboard input. In Zellij `pane` mode, Zellij usually captures `j`, `k`, arrow keys, and `q` first.

If you move focus to the plugin with `Ctrl p` and `h`/Left, you are still in Zellij `pane` mode. Press Enter or Esc to return to `normal` mode before using direct plugin keys, or use the personal `Alt` bindings above so focus and mode do not matter.

`just dev-session-local` starts Zellij with development keybinds that map `Ctrl p` pane mode to plugin commands:

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
