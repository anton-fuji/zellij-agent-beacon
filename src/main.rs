mod agent;
mod detection;
mod ui;

use std::collections::{BTreeMap, BTreeSet};

use zellij_tile::prelude::*;

use agent::Agent;
use detection::detect_agents;
use ui::{render_sidebar, SidebarMode, SidebarView};

const BUILD_LABEL: &str = "phase7-startup-stability";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Close,
    Hide,
    ToggleCompact,
    ToggleDiagnostics,
    ToggleHelp,
    SelectNext,
    SelectPrevious,
    FocusSelected,
}

#[derive(Clone)]
struct State {
    permissions_granted: bool,
    tabs: Vec<TabInfo>,
    pane_manifest: PaneManifest,
    exited_terminal_panes: BTreeSet<u32>,
    running_commands: BTreeMap<u32, String>,
    enable_running_command_detection: bool,
    agents: Vec<Agent>,
    selected_agent_index: Option<usize>,
    sidebar_mode: SidebarMode,
    status_message: Option<String>,
    show_diagnostics: bool,
    show_help: bool,
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
            enable_running_command_detection: false,
            agents: Vec::new(),
            selected_agent_index: None,
            sidebar_mode: SidebarMode::Normal,
            status_message: None,
            show_diagnostics: false,
            show_help: false,
            last_event: "",
        }
    }
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.enable_running_command_detection = configuration
            .get("running_command_detection")
            .is_some_and(|value| is_truthy(value));

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
                    self.handle_command(Command::Close);
                } else if key.is_key_without_modifier(BareKey::Char('h')) {
                    self.handle_command(Command::Hide);
                    return true;
                } else if key.is_key_without_modifier(BareKey::Char('c')) {
                    self.handle_command(Command::ToggleCompact);
                    return true;
                } else if key.is_key_without_modifier(BareKey::Char('d')) {
                    self.handle_command(Command::ToggleDiagnostics);
                    return true;
                } else if key.is_key_without_modifier(BareKey::Char('?')) {
                    self.handle_command(Command::ToggleHelp);
                    return true;
                } else if key.is_key_without_modifier(BareKey::Char('j'))
                    || key.is_key_without_modifier(BareKey::Down)
                {
                    self.handle_command(Command::SelectNext);
                    return true;
                } else if key.is_key_without_modifier(BareKey::Char('k'))
                    || key.is_key_without_modifier(BareKey::Up)
                {
                    self.handle_command(Command::SelectPrevious);
                    return true;
                } else if key.is_key_without_modifier(BareKey::Enter) {
                    self.handle_command(Command::FocusSelected);
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        let Some(command) = command_from_pipe(&pipe_message) else {
            return false;
        };

        self.handle_command(command);
        true
    }

    fn render(&mut self, rows: usize, cols: usize) {
        let diagnostics = self.diagnostic_lines();
        let view = SidebarView {
            permissions_granted: self.permissions_granted,
            agents: &self.agents,
            selected_agent_index: self.selected_agent_index,
            mode: self.sidebar_mode,
            status_message: self.status_message.as_deref(),
            show_help: self.show_help,
            show_diagnostics: self.show_diagnostics,
            diagnostics: &diagnostics,
        };

        for line in render_sidebar(rows, cols, view) {
            println!("{line}");
        }
    }
}

impl State {
    fn handle_command(&mut self, command: Command) {
        match command {
            Command::Close => {
                self.last_event = "close requested";
                close_plugin();
            }
            Command::Hide => {
                self.last_event = "hide requested";
                hide_plugin();
            }
            Command::ToggleCompact => {
                self.sidebar_mode = self.sidebar_mode.toggled();
                self.last_event = "sidebar mode toggled";
            }
            Command::ToggleDiagnostics => {
                self.show_diagnostics = !self.show_diagnostics;
                self.last_event = "diagnostics toggled";
            }
            Command::ToggleHelp => {
                self.show_help = !self.show_help;
                self.last_event = "help toggled";
            }
            Command::SelectNext => {
                self.select_next_agent();
                self.last_event = "selected next agent";
                self.status_message =
                    selected_agent_message(self.selected_agent_index, &self.agents);
            }
            Command::SelectPrevious => {
                self.select_previous_agent();
                self.last_event = "selected previous agent";
                self.status_message =
                    selected_agent_message(self.selected_agent_index, &self.agents);
            }
            Command::FocusSelected => self.focus_selected_agent(),
        }
    }

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
        self.running_commands.clear();

        if !self.permissions_granted || !self.enable_running_command_detection {
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

        for pane_id in pane_ids {
            let Some(command_parts) = pane_running_command(PaneId::Terminal(pane_id)) else {
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
            format!(
                "run cmd: {}",
                if self.enable_running_command_detection {
                    "on"
                } else {
                    "off"
                }
            ),
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
        show_agent_pane(agent.pane_id);
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

#[cfg(target_arch = "wasm32")]
fn close_plugin() {
    close_self();
}

#[cfg(not(target_arch = "wasm32"))]
fn close_plugin() {}

#[cfg(target_arch = "wasm32")]
fn hide_plugin() {
    hide_self();
}

#[cfg(not(target_arch = "wasm32"))]
fn hide_plugin() {}

#[cfg(target_arch = "wasm32")]
fn show_agent_pane(pane_id: PaneId) {
    show_pane_with_id(pane_id, true, true);
}

#[cfg(not(target_arch = "wasm32"))]
fn show_agent_pane(_pane_id: PaneId) {}

#[cfg(target_arch = "wasm32")]
fn pane_running_command(pane_id: PaneId) -> Option<Vec<String>> {
    get_pane_running_command(pane_id).ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn pane_running_command(_pane_id: PaneId) -> Option<Vec<String>> {
    None
}

fn command_from_pipe(pipe_message: &PipeMessage) -> Option<Command> {
    if pipe_message.name != "zab" && pipe_message.name != "zellij-agent-beacon" {
        return None;
    }

    let command = pipe_message
        .payload
        .as_deref()
        .or_else(|| pipe_message.args.get("command").map(String::as_str))?
        .trim();

    match command {
        "close" | "quit" => Some(Command::Close),
        "hide" => Some(Command::Hide),
        "compact" | "toggle-compact" => Some(Command::ToggleCompact),
        "diagnostics" | "toggle-diagnostics" => Some(Command::ToggleDiagnostics),
        "help" | "toggle-help" => Some(Command::ToggleHelp),
        "next" | "down" => Some(Command::SelectNext),
        "previous" | "prev" | "up" => Some(Command::SelectPrevious),
        "focus" | "enter" => Some(Command::FocusSelected),
        _ => None,
    }
}

fn selected_agent_message(selected_agent_index: Option<usize>, agents: &[Agent]) -> Option<String> {
    let agent = selected_agent_index.and_then(|index| agents.get(index))?;
    Some(format!("Selected {}", agent.kind))
}

fn is_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use zellij_tile::prelude::{PipeMessage, PipeSource};

    use super::{command_from_pipe, is_truthy, next_index, previous_index, Command};

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

    #[test]
    fn parses_pipe_commands_from_payload() {
        let message = PipeMessage {
            source: PipeSource::Keybind,
            name: "zab".to_owned(),
            payload: Some("next".to_owned()),
            args: BTreeMap::new(),
            is_private: false,
        };

        assert_eq!(command_from_pipe(&message), Some(Command::SelectNext));
    }

    #[test]
    fn ignores_unrelated_pipe_messages() {
        let message = PipeMessage {
            source: PipeSource::Keybind,
            name: "other".to_owned(),
            payload: Some("next".to_owned()),
            args: BTreeMap::new(),
            is_private: false,
        };

        assert_eq!(command_from_pipe(&message), None);
    }

    #[test]
    fn parses_pipe_commands_from_args() {
        let mut args = BTreeMap::new();
        args.insert("command".to_owned(), "focus".to_owned());
        let message = PipeMessage {
            source: PipeSource::Keybind,
            name: "zellij-agent-beacon".to_owned(),
            payload: None,
            args,
            is_private: false,
        };

        assert_eq!(command_from_pipe(&message), Some(Command::FocusSelected));
    }

    #[test]
    fn parses_help_pipe_command() {
        let message = PipeMessage {
            source: PipeSource::Keybind,
            name: "zab".to_owned(),
            payload: Some("help".to_owned()),
            args: BTreeMap::new(),
            is_private: false,
        };

        assert_eq!(command_from_pipe(&message), Some(Command::ToggleHelp));
    }

    #[test]
    fn parses_truthy_config_values() {
        assert!(is_truthy("true"));
        assert!(is_truthy("on"));
        assert!(is_truthy("1"));
        assert!(!is_truthy("false"));
        assert!(!is_truthy(""));
    }
}
