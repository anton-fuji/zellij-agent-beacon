use std::collections::BTreeMap;

use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    permissions_granted: bool,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[EventType::PermissionRequestResult, EventType::Key]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                self.permissions_granted = true;
                subscribe(&[
                    EventType::TabUpdate,
                    EventType::PaneUpdate,
                    EventType::PaneClosed,
                    EventType::CommandPaneExited,
                ]);
                true
            }
            Event::PermissionRequestResult(PermissionStatus::Denied) => {
                self.permissions_granted = false;
                true
            }
            Event::Key(key) => {
                if matches!(key.bare_key, BareKey::Char('q')) && key.has_no_modifiers() {
                    close_self();
                }
                false
            }
            Event::TabUpdate(_) | Event::PaneUpdate(_) | Event::PaneClosed(_) => true,
            Event::CommandPaneExited(_, _, _) => true,
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        let width = cols.max(1);
        let status = if self.permissions_granted {
            "ready"
        } else {
            "waiting for permissions"
        };

        println!("{:<width$}", "AI Agents", width = width);
        println!("{:<width$}", status, width = width);

        for _ in 2..rows {
            println!("{:<width$}", "", width = width);
        }
    }
}
