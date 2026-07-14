mod agent;
mod detection;
mod ui;

use std::collections::{BTreeMap, BTreeSet};

use zellij_tile::prelude::*;

use agent::Agent;
use detection::detect_agents;
use ui::{render_sidebar, SidebarMode, SidebarView};

const BUILD_LABEL: &str = "phase5-status-handling";

#[derive(Clone)]
struct State {
    permissions_granted: bool,
    tabs: Vec<TabInfo>,
    pane_manifest: PaneManifest,
    exited_terminal_panes: BTreeSet<u32>,
    running_commands: BTreeMap<u32, String>,
    agents: Vec<Agent>,
    selected_agent_index: Option<usize>,
    sidebar_mode: SidebarMode,
    status_message: Option<String>,
    show_diagnostics: bool,
    last_event: &'static str,
}

impl Default for State {
    fn default() -> Self {
        Self {
            permissions_granted: false,
            tabs: Vec::new(),
            pane_manifest: PaneManifest::default(),
            exited_terminal_panes: BTreeSet::new(),
            running_commands: BTreeMap::new(),
            agents: Vec::new(),
            selected_agent_index: None,
            sidebar_mode: SidebarMode::Normal,
            status_message: None,
            show_diagnostics: false,
            last_event: "",
        }
    }
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
                if key.is_key_without_modifier(BareKey::Char('q')) {
                    close_self();
                } else if key.is_key_without_modifier(BareKey::Char('h')) {
                    hide_self();
                    return false;
                } else if key.is_key_without_modifier(BareKey::Char('c')) {
                    self.sidebar_mode = self.sidebar_mode.toggled();
                    self.last_event = "sidebar mode toggled";
                    return true;
                } else if key.is_key_without_modifier(BareKey::Char('d')) {
                    self.show_diagnostics = !self.show_diagnostics;
                    self.last_event = "diagnostics toggled";
                    return true;
                } else if key.is_key_without_modifier(BareKey::Char('j'))
                    || key.is_key_without_modifier(BareKey::Down)
                {
                    self.select_next_agent();
                    return true;
                } else if key.is_key_without_modifier(BareKey::Char('k'))
                    || key.is_key_without_modifier(BareKey::Up)
                {
                    self.select_previous_agent();
                    return true;
                } else if key.is_key_without_modifier(BareKey::Enter) {
                    self.focus_selected_agent();
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        let diagnostics = self.diagnostic_lines();
        let view = SidebarView {
            permissions_granted: self.permissions_granted,
            agents: &self.agents,
            selected_agent_index: self.selected_agent_index,
            mode: self.sidebar_mode,
            status_message: self.status_message.as_deref(),
            show_diagnostics: self.show_diagnostics,
            diagnostics: &diagnostics,
        };

        for line in render_sidebar(rows, cols, view) {
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
        self.clamp_selected_agent();
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

    fn clamp_selected_agent(&mut self) {
        self.selected_agent_index = match (self.selected_agent_index, self.agents.len()) {
            (_, 0) => None,
            (Some(index), len) if index < len => Some(index),
            _ => Some(0),
        };
    }

    fn select_next_agent(&mut self) {
        self.selected_agent_index = next_index(self.selected_agent_index, self.agents.len());
    }

    fn select_previous_agent(&mut self) {
        self.selected_agent_index = previous_index(self.selected_agent_index, self.agents.len());
    }

    fn focus_selected_agent(&mut self) {
        let Some(agent) = self
            .selected_agent_index
            .and_then(|index| self.agents.get(index))
        else {
            self.last_event = "focus skipped: no agent";
            self.status_message = Some("No agent selected".to_owned());
            return;
        };

        if !agent.is_focusable() {
            self.last_event = "focus skipped: agent unavailable";
            self.status_message = Some(format!("{} is {}", agent.kind, agent.status_label()));
            return;
        }

        if !self.pane_exists(agent.pane_id) {
            self.last_event = "focus skipped: pane missing";
            self.status_message = Some("Agent pane is no longer available".to_owned());
            self.refresh_agents();
            return;
        }

        let message = format!("Focused {}", agent.kind);
        show_pane_with_id(agent.pane_id, true, true);
        self.last_event = "focused agent pane";
        self.status_message = Some(message);
    }

    fn pane_exists(&self, pane_id: PaneId) -> bool {
        self.pane_manifest
            .panes
            .values()
            .flat_map(|panes| panes.iter())
            .any(|pane| match pane_id {
                PaneId::Terminal(id) => !pane.is_plugin && pane.id == id && !pane.exited,
                PaneId::Plugin(id) => pane.is_plugin && pane.id == id && !pane.exited,
            })
    }
}

fn next_index(current: Option<usize>, len: usize) -> Option<usize> {
    match (current, len) {
        (_, 0) => None,
        (None, _) => Some(0),
        (Some(index), len) => Some((index + 1) % len),
    }
}

fn previous_index(current: Option<usize>, len: usize) -> Option<usize> {
    match (current, len) {
        (_, 0) => None,
        (None, _) => Some(0),
        (Some(0), len) => Some(len - 1),
        (Some(index), _) => Some(index - 1),
    }
}

#[cfg(test)]
mod tests {
    use super::{next_index, previous_index};

    #[test]
    fn next_index_wraps_and_handles_empty_lists() {
        assert_eq!(next_index(None, 0), None);
        assert_eq!(next_index(None, 3), Some(0));
        assert_eq!(next_index(Some(0), 3), Some(1));
        assert_eq!(next_index(Some(2), 3), Some(0));
    }

    #[test]
    fn previous_index_wraps_and_handles_empty_lists() {
        assert_eq!(previous_index(None, 0), None);
        assert_eq!(previous_index(None, 3), Some(0));
        assert_eq!(previous_index(Some(2), 3), Some(1));
        assert_eq!(previous_index(Some(0), 3), Some(2));
    }
}
