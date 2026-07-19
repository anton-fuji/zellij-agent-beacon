set dotenv-load := false

target := "wasm32-wasip1"
sidebar_width := "25%"

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

dev-layout: dev-wasm init-layout
    zellij --layout zellij.kdl

dev-session: dev-wasm init-layout init-dev-config
    zellij --config-dir .zellij-dev -n zellij.kdl

dev-session-mock: dev-wasm init-mock-layout init-dev-config
    zellij --config-dir .zellij-dev -n zellij.mock.kdl

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
