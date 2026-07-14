use crate::agent::{Agent, AgentStatus};

pub fn render_sidebar(
    rows: usize,
    cols: usize,
    permissions_granted: bool,
    agents: &[Agent],
    selected_agent_index: Option<usize>,
    diagnostics: &[String],
) -> Vec<String> {
    let width = cols.max(1);
    let mut lines = Vec::with_capacity(rows.max(1));

    push_line(&mut lines, width, "AI Agents");

    if !permissions_granted {
        push_line(&mut lines, width, "waiting for permissions");
    } else if agents.is_empty() {
        push_line(&mut lines, width, "no agents detected");
        push_line(&mut lines, width, "");
        for diagnostic in diagnostics {
            push_line(&mut lines, width, diagnostic);
        }
    } else {
        push_line(&mut lines, width, &format!("{} detected", agents.len()));
        push_line(&mut lines, width, "");

        for (index, agent) in agents.iter().enumerate() {
            let selector = if Some(index) == selected_agent_index {
                ">"
            } else {
                " "
            };
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
            push_line(&mut lines, width, &format!("  {}", agent.status.label()));
            push_line(&mut lines, width, "");
        }
    }

    while lines.len() < rows {
        push_line(&mut lines, width, "");
    }

    lines.truncate(rows);
    lines
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

#[cfg(test)]
mod tests {
    use zellij_tile::prelude::PaneId;

    use super::*;
    use crate::agent::{AgentKind, AgentStatus};

    #[test]
    fn renders_empty_state() {
        let lines = render_sidebar(3, 20, true, &[], None, &[]);

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].trim_end(), "AI Agents");
        assert_eq!(lines[1].trim_end(), "no agents detected");
    }

    #[test]
    fn clamps_lines_to_width_and_rows() {
        let agent = Agent {
            kind: AgentKind::ClaudeCode,
            status: AgentStatus::Running,
            pane_id: PaneId::Terminal(2),
            tab_position: 0,
            tab_name: Some("very-long-tab-name".to_owned()),
            pane_title: Some("very-long-pane-title".to_owned()),
            command: Some("claude".to_owned()),
        };

        let lines = render_sidebar(4, 10, true, &[agent], Some(0), &[]);

        assert_eq!(lines.len(), 4);
        assert!(lines.iter().all(|line| line.chars().count() == 10));
    }

    #[test]
    fn marks_selected_agent() {
        let agents = vec![
            Agent {
                kind: AgentKind::Codex,
                status: AgentStatus::Running,
                pane_id: PaneId::Terminal(1),
                tab_position: 0,
                tab_name: Some("one".to_owned()),
                pane_title: Some("first".to_owned()),
                command: Some("codex".to_owned()),
            },
            Agent {
                kind: AgentKind::OpenCode,
                status: AgentStatus::Running,
                pane_id: PaneId::Terminal(2),
                tab_position: 0,
                tab_name: Some("two".to_owned()),
                pane_title: Some("second".to_owned()),
                command: Some("opencode".to_owned()),
            },
        ];

        let lines = render_sidebar(10, 24, true, &agents, Some(1), &[]);

        assert!(lines.iter().any(|line| line.trim_end() == "> * OpenCode"));
    }
}
