use std::fmt;

use zellij_tile::prelude::PaneId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentKind {
    Codex,
    ClaudeCode,
    OpenCode,
}

impl AgentKind {
    pub fn display_name(self) -> &'static str {
        match self {
            AgentKind::Codex => "Codex",
            AgentKind::ClaudeCode => "Claude Code",
            AgentKind::OpenCode => "OpenCode",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Running,
    Exited,
    Unknown,
}

impl AgentStatus {
    pub fn label(self) -> &'static str {
        match self {
            AgentStatus::Running => "Running",
            AgentStatus::Exited => "Exited",
            AgentStatus::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    pub kind: AgentKind,
    pub status: AgentStatus,
    pub pane_id: PaneId,
    pub tab_position: usize,
    pub tab_name: Option<String>,
    pub pane_title: Option<String>,
    pub command: Option<String>,
}

impl Agent {
    pub fn location_label(&self) -> String {
        let tab = self
            .tab_name
            .as_deref()
            .filter(|name| !name.trim().is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("tab {}", self.tab_position + 1));

        let pane = self
            .pane_title
            .as_deref()
            .filter(|title| !title.trim().is_empty())
            .unwrap_or("untitled");

        format!("{tab} / {pane}")
    }
}

impl fmt::Display for AgentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}
