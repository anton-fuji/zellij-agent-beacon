use crate::agent::{Agent, AgentStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarMode {
    Normal,
    Compact,
}

impl SidebarMode {
    pub fn toggled(self) -> Self {
        match self {
            SidebarMode::Normal => SidebarMode::Compact,
            SidebarMode::Compact => SidebarMode::Normal,
        }
    }
}

pub struct SidebarView<'a> {
    pub permissions_granted: bool,
    pub agents: &'a [Agent],
    pub selected_agent_index: Option<usize>,
    pub mode: SidebarMode,
    pub status_message: Option<&'a str>,
    pub show_help: bool,
    pub show_diagnostics: bool,
    pub diagnostics: &'a [String],
}

pub fn render_sidebar(rows: usize, cols: usize, view: SidebarView<'_>) -> Vec<String> {
    let SidebarView {
        permissions_granted,
        agents,
        selected_agent_index,
        mode,
        status_message,
        show_help,
        show_diagnostics,
        diagnostics,
    } = view;

    let width = cols.max(1);
    let mut lines = Vec::with_capacity(rows.max(1));
    let mode = if width < 24 {
        SidebarMode::Compact
    } else {
        mode
    };

    push_line(&mut lines, width, &format!("AI Agents ({})", agents.len()));
    if let Some(message) = status_message.filter(|message| !message.trim().is_empty()) {
        push_line(&mut lines, width, message);
    }

    if show_help {
        render_help(&mut lines, width, mode);
    } else if !permissions_granted {
        push_line(&mut lines, width, "waiting for permissions");
        push_line(&mut lines, width, "grant plugin access");
    } else if agents.is_empty() {
        push_line(&mut lines, width, "no agents detected");
        push_line(&mut lines, width, "start codex/claude");
        push_line(&mut lines, width, "or run mock session");
        if show_diagnostics {
            push_line(&mut lines, width, "");
            for diagnostic in diagnostics {
                push_line(&mut lines, width, diagnostic);
            }
        }
    } else {
        push_line(&mut lines, width, "");

        for (index, agent) in agents.iter().enumerate() {
            let selector = if Some(index) == selected_agent_index {
                ">"
            } else {
                " "
            };
            match mode {
                SidebarMode::Normal => {
                    push_line(
                        &mut lines,
                        width,
                        &format!(
                            "{} {} {}",
                            selector,
                            status_marker(agent.status),
                            agent.kind.display_name()
                        ),
                    );
                    push_line(&mut lines, width, &format!("  {}", agent.location_label()));
                    push_line(&mut lines, width, &format!("  {}", agent.status_label()));
                    push_line(&mut lines, width, "");
                }
                SidebarMode::Compact => {
                    push_line(
                        &mut lines,
                        width,
                        &format!(
                            "{}{} {}",
                            selector,
                            status_marker(agent.status),
                            compact_agent_label(agent)
                        ),
                    );
                }
            }
        }

        if show_diagnostics {
            push_line(&mut lines, width, "");
            for diagnostic in diagnostics {
                push_line(&mut lines, width, diagnostic);
            }
        }
    }

    while lines.len() < rows {
        push_line(&mut lines, width, "");
    }

    lines.truncate(rows);
    lines
}

fn render_help(lines: &mut Vec<String>, width: usize, mode: SidebarMode) {
    push_line(lines, width, "");
    push_line(lines, width, "Controls");
    match mode {
        SidebarMode::Normal => {
            push_line(lines, width, "j/down next");
            push_line(lines, width, "k/up previous");
            push_line(lines, width, "enter focus");
            push_line(lines, width, "q close");
            push_line(lines, width, "r refresh");
            push_line(lines, width, "c compact");
            push_line(lines, width, "d diagnostics");
            push_line(lines, width, "? help");
        }
        SidebarMode::Compact => {
            push_line(lines, width, "j next");
            push_line(lines, width, "k prev");
            push_line(lines, width, "ent focus");
            push_line(lines, width, "q close");
            push_line(lines, width, "r scan");
            push_line(lines, width, "? help");
        }
    }
}

fn push_line(lines: &mut Vec<String>, width: usize, line: &str) {
    lines.push(fit_line(line, width));
}

fn fit_line(line: &str, width: usize) -> String {
    let mut fitted = line.chars().take(width).collect::<String>();
    let padding = width.saturating_sub(fitted.chars().count());
    fitted.extend(std::iter::repeat_n(' ', padding));
    fitted
}

fn status_marker(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Running => "*",
        AgentStatus::Exited => "x",
        AgentStatus::Unknown => "?",
    }
}

fn compact_agent_label(agent: &Agent) -> String {
    let tab = agent
        .tab_name
        .as_deref()
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("tab");

    format!("{} {}", agent.kind.display_name(), tab)
}

#[cfg(test)]
mod tests {
    use zellij_tile::prelude::PaneId;

    use super::*;
    use crate::agent::{AgentKind, AgentStatus};

    #[test]
    fn renders_empty_state() {
        let view = SidebarView {
            permissions_granted: true,
            agents: &[],
            selected_agent_index: None,
            mode: SidebarMode::Normal,
            status_message: None,
            show_help: false,
            show_diagnostics: false,
            diagnostics: &[],
        };

        let lines = render_sidebar(5, 20, view);

        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0].trim_end(), "AI Agents (0)");
        assert_eq!(lines[1].trim_end(), "no agents detected");
        assert_eq!(lines[2].trim_end(), "start codex/claude");
        assert_eq!(lines[3].trim_end(), "or run mock session");
    }

    #[test]
    fn hides_diagnostics_by_default() {
        let diagnostics = ["debug line".to_owned()];
        let view = SidebarView {
            permissions_granted: true,
            agents: &[],
            selected_agent_index: None,
            mode: SidebarMode::Normal,
            status_message: None,
            show_help: false,
            show_diagnostics: false,
            diagnostics: &diagnostics,
        };

        let lines = render_sidebar(5, 24, view);

        assert!(!lines.iter().any(|line| line.contains("debug line")));
    }

    #[test]
    fn clamps_lines_to_width_and_rows() {
        let agent = Agent {
            kind: AgentKind::ClaudeCode,
            status: AgentStatus::Running,
            exit_status: None,
            pane_id: PaneId::Terminal(2),
            tab_position: 0,
            tab_name: Some("very-long-tab-name".to_owned()),
            pane_title: Some("very-long-pane-title".to_owned()),
            command: Some("claude".to_owned()),
        };

        let agents = [agent];
        let view = SidebarView {
            permissions_granted: true,
            agents: &agents,
            selected_agent_index: Some(0),
            mode: SidebarMode::Normal,
            status_message: None,
            show_help: false,
            show_diagnostics: false,
            diagnostics: &[],
        };

        let lines = render_sidebar(4, 10, view);

        assert_eq!(lines.len(), 4);
        assert!(lines.iter().all(|line| line.chars().count() == 10));
    }

    #[test]
    fn marks_selected_agent() {
        let agents = vec![
            Agent {
                kind: AgentKind::Codex,
                status: AgentStatus::Running,
                exit_status: None,
                pane_id: PaneId::Terminal(1),
                tab_position: 0,
                tab_name: Some("one".to_owned()),
                pane_title: Some("first".to_owned()),
                command: Some("codex".to_owned()),
            },
            Agent {
                kind: AgentKind::OpenCode,
                status: AgentStatus::Running,
                exit_status: None,
                pane_id: PaneId::Terminal(2),
                tab_position: 0,
                tab_name: Some("two".to_owned()),
                pane_title: Some("second".to_owned()),
                command: Some("opencode".to_owned()),
            },
        ];

        let view = SidebarView {
            permissions_granted: true,
            agents: &agents,
            selected_agent_index: Some(1),
            mode: SidebarMode::Normal,
            status_message: None,
            show_help: false,
            show_diagnostics: false,
            diagnostics: &[],
        };

        let lines = render_sidebar(10, 24, view);

        assert!(lines.iter().any(|line| line.trim_end() == "> * OpenCode"));
    }

    #[test]
    fn renders_compact_rows_when_requested() {
        let agent = Agent {
            kind: AgentKind::Codex,
            status: AgentStatus::Running,
            exit_status: None,
            pane_id: PaneId::Terminal(1),
            tab_position: 0,
            tab_name: Some("api".to_owned()),
            pane_title: Some("pane".to_owned()),
            command: Some("codex".to_owned()),
        };

        let agents = [agent];
        let view = SidebarView {
            permissions_granted: true,
            agents: &agents,
            selected_agent_index: Some(0),
            mode: SidebarMode::Compact,
            status_message: None,
            show_help: false,
            show_diagnostics: false,
            diagnostics: &[],
        };

        let lines = render_sidebar(4, 30, view);

        assert!(lines.iter().any(|line| line.trim_end() == ">* Codex api"));
    }

    #[test]
    fn renders_focus_status_message() {
        let view = SidebarView {
            permissions_granted: true,
            agents: &[],
            selected_agent_index: None,
            mode: SidebarMode::Normal,
            status_message: Some("pane unavailable"),
            show_help: false,
            show_diagnostics: false,
            diagnostics: &[],
        };

        let lines = render_sidebar(4, 24, view);

        assert!(lines
            .iter()
            .any(|line| line.trim_end() == "pane unavailable"));
    }

    #[test]
    fn renders_exit_status_when_available() {
        let agent = Agent {
            kind: AgentKind::Codex,
            status: AgentStatus::Exited,
            exit_status: Some(1),
            pane_id: PaneId::Terminal(1),
            tab_position: 0,
            tab_name: Some("api".to_owned()),
            pane_title: Some("pane".to_owned()),
            command: Some("codex".to_owned()),
        };
        let agents = [agent];
        let view = SidebarView {
            permissions_granted: true,
            agents: &agents,
            selected_agent_index: Some(0),
            mode: SidebarMode::Normal,
            status_message: None,
            show_help: false,
            show_diagnostics: false,
            diagnostics: &[],
        };

        let lines = render_sidebar(6, 30, view);

        assert!(lines.iter().any(|line| line.trim_end() == "  Exited 1"));
    }

    #[test]
    fn renders_help_when_requested() {
        let view = SidebarView {
            permissions_granted: true,
            agents: &[],
            selected_agent_index: None,
            mode: SidebarMode::Normal,
            status_message: None,
            show_help: true,
            show_diagnostics: false,
            diagnostics: &[],
        };

        let lines = render_sidebar(10, 24, view);

        assert!(lines.iter().any(|line| line.trim_end() == "Controls"));
        assert!(lines.iter().any(|line| line.trim_end() == "j/down next"));
    }
}
