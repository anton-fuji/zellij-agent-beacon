set dotenv-load := false

target := "wasm32-wasip1"
sidebar_width := "25%"
launch_key := "Alt a"

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

_assert-outside-zellij:
    @if zellij action query-tab-names >/dev/null 2>&1; then printf '%s\n' "Refusing to start a nested Zellij session." "Inside Zellij, use 'just dev-layout' or 'just dev-layout-mock'." "From a normal terminal, use a dev-session-local or dev-session-personal recipe."; exit 1; fi

dev-plugin:
    cargo build --target {{target}}
    zellij plugin -s --floating --width 25% -- file:{{justfile_directory()}}/target/{{target}}/debug/zellij-agent-beacon.wasm

init-layout:
    @printf '%s\n' \
        'layout {' \
        '    pane size=1 borderless=true {' \
        '        plugin location="zellij:tab-bar"' \
        '    }' \
        '    pane split_direction="vertical" {' \
        '        pane size="{{sidebar_width}}" {' \
        '            plugin location="file:{{justfile_directory()}}/target/{{target}}/debug/zellij-agent-beacon.wasm"' \
        '        }' \
        '        pane' \
        '    }' \
        '    pane size=1 borderless=true {' \
        '        plugin location="zellij:status-bar"' \
        '    }' \
        '}' \
        > zellij.kdl
    @echo "Wrote zellij.kdl"

init-mock-layout:
    @printf '%s\n' \
        'layout {' \
        '    pane size=1 borderless=true {' \
        '        plugin location="zellij:tab-bar"' \
        '    }' \
        '    pane split_direction="vertical" {' \
        '        pane size="{{sidebar_width}}" {' \
        '            plugin location="file:{{justfile_directory()}}/target/{{target}}/debug/zellij-agent-beacon.wasm"' \
        '        }' \
        '        pane name="codex"' \
        '    }' \
        '    pane size=1 borderless=true {' \
        '        plugin location="zellij:status-bar"' \
        '    }' \
        '}' \
        > zellij.mock.kdl
    @echo "Wrote zellij.mock.kdl"

init-dev-config:
    @mkdir -p .zellij-dev
    @printf '%s\n' \
        'keybinds {' \
        '    pane {' \
        '        bind "down" {' \
        '            MessagePlugin {' \
        '                name "zab"' \
        '                payload "next"' \
        '            }' \
        '        }' \
        '        bind "j" {' \
        '            MessagePlugin {' \
        '                name "zab"' \
        '                payload "next"' \
        '            }' \
        '        }' \
        '        bind "up" {' \
        '            MessagePlugin {' \
        '                name "zab"' \
        '                payload "previous"' \
        '            }' \
        '        }' \
        '        bind "k" {' \
        '            MessagePlugin {' \
        '                name "zab"' \
        '                payload "previous"' \
        '            }' \
        '        }' \
        '        bind "enter" {' \
        '            MessagePlugin {' \
        '                name "zab"' \
        '                payload "focus"' \
        '            }' \
        '            SwitchToMode "normal"' \
        '        }' \
        '        bind "q" {' \
        '            MessagePlugin {' \
        '                name "zab"' \
        '                payload "close"' \
        '            }' \
        '            SwitchToMode "normal"' \
        '        }' \
        '        bind "?" {' \
        '            MessagePlugin {' \
        '                name "zab"' \
        '                payload "help"' \
        '            }' \
        '        }' \
        '        bind "r" {' \
        '            MessagePlugin {' \
        '                name "zab"' \
        '                payload "refresh"' \
        '            }' \
        '        }' \
        '        bind "Ctrl p" { SwitchToMode "normal"; }' \
        '    }' \
        '}' \
        > .zellij-dev/config.kdl
    @echo "Wrote .zellij-dev/config.kdl"

init-launch-keybind:
    @mkdir -p .zellij-dev
    @printf '%s\n' \
        'keybinds {' \
        '    shared_except "locked" {' \
        '        bind "{{launch_key}}" {' \
        '            LaunchOrFocusPlugin "file:{{justfile_directory()}}/target/{{target}}/debug/zellij-agent-beacon.wasm" {' \
        '                floating true' \
        '                move_to_focused_tab true' \
        '                skip_plugin_cache true' \
        '            }' \
        '            SwitchToMode "Normal"' \
        '        }' \
        '        bind "Alt ." {' \
        '            MessagePlugin {' \
        '                name "zab"' \
        '                payload "next"' \
        '            }' \
        '        }' \
        '        bind "Alt ," {' \
        '            MessagePlugin {' \
        '                name "zab"' \
        '                payload "previous"' \
        '            }' \
        '        }' \
        '        bind "Alt Enter" {' \
        '            MessagePlugin {' \
        '                name "zab"' \
        '                payload "focus"' \
        '            }' \
        '            SwitchToMode "Normal"' \
        '        }' \
        '        bind "Alt r" {' \
        '            MessagePlugin {' \
        '                name "zab"' \
        '                payload "refresh"' \
        '            }' \
        '        }' \
        '        bind "Alt q" {' \
        '            MessagePlugin {' \
        '                name "zab"' \
        '                payload "close"' \
        '            }' \
        '            SwitchToMode "Normal"' \
        '        }' \
        '        bind "Alt ?" {' \
        '            MessagePlugin {' \
        '                name "zab"' \
        '                payload "help"' \
        '            }' \
        '        }' \
        '    }' \
        '}' \
        > .zellij-dev/launch-keybind.kdl
    @echo "Wrote .zellij-dev/launch-keybind.kdl"

dev-layout: dev-wasm init-layout
    @if [ -n "$ZELLIJ" ]; then zellij action new-tab --layout zellij.kdl --name zab-dev; else zellij --layout zellij.kdl; fi

dev-session-local: _assert-outside-zellij dev-wasm init-layout init-dev-config
    zellij --config-dir .zellij-dev -n zellij.kdl

dev-session-local-mock: _assert-outside-zellij dev-wasm init-mock-layout init-dev-config
    zellij --config-dir .zellij-dev -n zellij.mock.kdl

dev-session-personal: _assert-outside-zellij dev-wasm init-layout
    zellij --config-dir "$HOME/.config/zellij" -n zellij.kdl

dev-session-personal-mock: _assert-outside-zellij dev-wasm init-mock-layout
    zellij --config-dir "$HOME/.config/zellij" -n zellij.mock.kdl

dev-session: dev-session-local

dev-session-mock: dev-session-local-mock

dev-layout-mock: dev-wasm init-mock-layout
    @if [ -n "$ZELLIJ" ]; then zellij action new-tab --layout zellij.mock.kdl --name zab-dev-mock; else zellij --layout zellij.mock.kdl; fi

dev-floating: dev-wasm
    zellij action launch-or-focus-plugin --floating --move-to-focused-tab --skip-plugin-cache file:{{justfile_directory()}}/target/{{target}}/debug/zellij-agent-beacon.wasm

dev-sesions: dev-session

dev-sessions: dev-session

zab-next:
    zellij action pipe --name zab -- next

zab-previous:
    zellij action pipe --name zab -- previous

zab-focus:
    zellij action pipe --name zab -- focus

zab-help:
    zellij action pipe --name zab -- help

zab-refresh:
    zellij action pipe --name zab -- refresh

zab-close:
    zellij action pipe --name zab -- close
