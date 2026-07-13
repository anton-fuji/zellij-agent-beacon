mod agent;
mod detection;
mod ui;

use std::collections::{BTreeMap, BTreeSet};

use zellij_tile::prelude::*;

use agent::Agent;
use detection::detect_agents;
use ui::render_sidebar;

const BUILD_LABEL: &str = "phase2-diagnostics";

#[derive(Default, Clone)]
struct State {
    permissions_granted: bool,
    tabs: Vec<TabInfo>,
    pane_manifest: PaneManifest,
    exited_terminal_panes: BTreeSet<u32>,
    running_commands: BTreeMap<u32, String>,
    agents: Vec<Agent>,
    last_event: &'static str,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        subscribe(&[
            EventType::PermissionRequestResult,
            EventType::Key,
            EventType::TabUpdate,
            EventType::PaneUpdate,
            EventType::PaneClosed,
            EventType::CommandPaneExited,
        ]);
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                self.permissions_granted = true;
                self.last_event = "permissions granted";
                true
            }
            Event::PermissionRequestResult(PermissionStatus::Denied) => {
                self.permissions_granted = false;
                self.last_event = "permissions denied";
                true
            }
            Event::TabUpdate(tabs) => {
                self.tabs = tabs;
                self.last_event = "tab update";
                self.refresh_agents();
                true
            }
            Event::PaneUpdate(pane_manifest) => {
                self.pane_manifest = pane_manifest;
                self.last_event = "pane update";
                self.refresh_agents();
                true
            }
            Event::PaneClosed(pane_id) => {
                self.last_event = "pane closed";
                if let PaneId::Terminal(id) = pane_id {
                    self.exited_terminal_panes.remove(&id);
                }
                self.refresh_agents();
                true
            }
            Event::CommandPaneExited(terminal_pane_id, _, _) => {
                self.last_event = "command pane exited";
                self.exited_terminal_panes.insert(terminal_pane_id);
                self.refresh_agents();
                true
            }
            Event::Key(key) => {
                if matches!(key.bare_key, BareKey::Char('q')) && key.has_no_modifiers() {
                    close_self();
                }
                false
            }
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        let diagnostics = self.diagnostic_lines();
        for line in render_sidebar(
            rows,
            cols,
            self.permissions_granted,
            &self.agents,
            &diagnostics,
        ) {
            println!("{line}");
        }
    }
}

impl State {
    fn refresh_agents(&mut self) {
        self.refresh_running_commands();
        self.agents = detect_agents(
            &self.pane_manifest,
            &self.tabs,
            &self.exited_terminal_panes,
            &self.running_commands,
        );
    }

    fn refresh_running_commands(&mut self) {
        if !self.permissions_granted {
            return;
        }

        let pane_ids = self
            .pane_manifest
            .panes
            .values()
            .flat_map(|panes| panes.iter())
            .filter(|pane| !pane.is_plugin)
            .map(|pane| pane.id)
            .collect::<Vec<_>>();

        self.running_commands.clear();

        for pane_id in pane_ids {
            let Ok(command_parts) = get_pane_running_command(PaneId::Terminal(pane_id)) else {
                continue;
            };
            let command = command_parts.join(" ");
            if !command.trim().is_empty() {
                self.running_commands.insert(pane_id, command);
            }
        }
    }

    fn diagnostic_lines(&self) -> Vec<String> {
        let pane_count = self
            .pane_manifest
            .panes
            .values()
            .map(Vec::len)
            .sum::<usize>();
        let mut lines = vec![
            format!("build: {BUILD_LABEL}"),
            format!("last: {}", self.last_event),
            format!("tabs: {}", self.tabs.len()),
            format!("panes: {pane_count}"),
        ];

        let mut tab_positions = self.pane_manifest.panes.keys().copied().collect::<Vec<_>>();
        tab_positions.sort_unstable();

        for tab_position in tab_positions {
            let Some(panes) = self.pane_manifest.panes.get(&tab_position) else {
                continue;
            };

            for pane in panes.iter().filter(|pane| !pane.is_plugin).take(8) {
                let command = pane.terminal_command.as_deref().unwrap_or("-");
                let running_command = self
                    .running_commands
                    .get(&pane.id)
                    .map(String::as_str)
                    .unwrap_or("-");
                let title = if pane.title.trim().is_empty() {
                    "-"
                } else {
                    pane.title.as_str()
                };
                lines.push(format!(
                    "t{} p{} cmd={} title={}",
                    tab_position + 1,
                    pane.id,
                    command,
                    title
                ));
                lines.push(format!("  run={running_command}"));
            }
        }

        lines
    }
}
