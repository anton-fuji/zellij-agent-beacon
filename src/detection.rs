use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use zellij_tile::prelude::{PaneId, PaneInfo, PaneManifest, TabInfo};

use crate::agent::{Agent, AgentKind, AgentStatus};

pub fn detect_agent_kind(command_or_title: &str) -> Option<AgentKind> {
    command_or_title
        .split_whitespace()
        .find_map(agent_kind_from_token)
}

pub fn detect_agents(
    pane_manifest: &PaneManifest,
    tabs: &[TabInfo],
    exited_terminal_panes: &BTreeSet<u32>,
    running_commands: &BTreeMap<u32, String>,
) -> Vec<Agent> {
    let tab_names = tab_name_by_position(tabs);
    let mut agents = Vec::new();

    let mut tab_positions = pane_manifest.panes.keys().copied().collect::<Vec<_>>();
    tab_positions.sort_unstable();

    for tab_position in tab_positions {
        let Some(panes) = pane_manifest.panes.get(&tab_position) else {
            continue;
        };

        for pane in panes {
            if let Some(agent) = detect_agent_in_pane(
                pane,
                tab_position,
                tab_names.get(&tab_position).cloned(),
                exited_terminal_panes,
                running_commands,
            ) {
                agents.push(agent);
            }
        }
    }

    agents.sort_by(|left, right| {
        left.tab_position
            .cmp(&right.tab_position)
            .then_with(|| pane_id_sort_key(left.pane_id).cmp(&pane_id_sort_key(right.pane_id)))
    });
    agents
}

fn detect_agent_in_pane(
    pane: &PaneInfo,
    tab_position: usize,
    tab_name: Option<String>,
    exited_terminal_panes: &BTreeSet<u32>,
    running_commands: &BTreeMap<u32, String>,
) -> Option<Agent> {
    if pane.is_plugin {
        return None;
    }

    let command = running_commands
        .get(&pane.id)
        .cloned()
        .or_else(|| pane.terminal_command.clone());
    let has_command = command
        .as_deref()
        .is_some_and(|command| !command.trim().is_empty());

    let detection_source = command
        .as_deref()
        .filter(|command| !command.trim().is_empty())
        .unwrap_or(&pane.title);

    let kind = detect_agent_kind(detection_source)?;
    let pane_id = PaneId::Terminal(pane.id);
    let status = if pane.exited
        || pane.is_held
        || pane.exit_status.is_some()
        || exited_terminal_panes.contains(&pane.id)
    {
        AgentStatus::Exited
    } else if has_command {
        AgentStatus::Running
    } else {
        AgentStatus::Unknown
    };

    Some(Agent {
        kind,
        status,
        exit_status: pane.exit_status,
        pane_id,
        tab_position,
        tab_name,
        pane_title: non_empty_string(&pane.title),
        command,
    })
}

fn tab_name_by_position(tabs: &[TabInfo]) -> HashMap<usize, String> {
    tabs.iter()
        .map(|tab| (tab.position, tab.name.clone()))
        .collect()
}

fn agent_kind_from_token(token: &str) -> Option<AgentKind> {
    let normalized = normalize_command_token(token)?;

    match normalized.as_str() {
        "codex" => Some(AgentKind::Codex),
        "claude" => Some(AgentKind::ClaudeCode),
        "opencode" => Some(AgentKind::OpenCode),
        _ => None,
    }
}

fn normalize_command_token(token: &str) -> Option<String> {
    let token = token
        .trim_matches(|c: char| matches!(c, '"' | '\'' | '`' | '[' | ']' | '(' | ')' | ','))
        .trim();

    if token.is_empty() {
        return None;
    }

    let basename = Path::new(token)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(token);

    Some(basename.to_ascii_lowercase())
}

fn pane_id_sort_key(pane_id: PaneId) -> (u8, u32) {
    match pane_id {
        PaneId::Terminal(id) => (0, id),
        PaneId::Plugin(id) => (1, id),
    }
}

fn non_empty_string(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_supported_agent_commands() {
        assert_eq!(detect_agent_kind("codex"), Some(AgentKind::Codex));
        assert_eq!(
            detect_agent_kind("/opt/homebrew/bin/claude --dangerously-skip-permissions"),
            Some(AgentKind::ClaudeCode)
        );
        assert_eq!(detect_agent_kind("npx opencode"), Some(AgentKind::OpenCode));
    }

    #[test]
    fn does_not_match_substrings_inside_other_commands() {
        assert_eq!(detect_agent_kind("my-codex-wrapper"), None);
        assert_eq!(detect_agent_kind("claude-helper"), None);
        assert_eq!(detect_agent_kind("opencode.log"), None);
    }

    #[test]
    fn skips_plugin_panes() {
        let pane = PaneInfo {
            id: 1,
            is_plugin: true,
            title: "codex".to_owned(),
            terminal_command: Some("codex".to_owned()),
            ..PaneInfo::default()
        };

        let agent = detect_agent_in_pane(
            &pane,
            0,
            Some("dev".to_owned()),
            &BTreeSet::new(),
            &BTreeMap::new(),
        );

        assert_eq!(agent, None);
    }

    #[test]
    fn marks_exited_command_panes() {
        let pane = PaneInfo {
            id: 7,
            title: "codex".to_owned(),
            terminal_command: Some("codex".to_owned()),
            ..PaneInfo::default()
        };
        let exited_panes = BTreeSet::from([7]);

        let agent = detect_agent_in_pane(
            &pane,
            0,
            Some("dev".to_owned()),
            &exited_panes,
            &BTreeMap::new(),
        )
        .expect("agent should be detected");

        assert_eq!(agent.status, AgentStatus::Exited);
        assert_eq!(agent.exit_status, None);
        assert_eq!(agent.pane_id, PaneId::Terminal(7));
    }

    #[test]
    fn preserves_exit_status() {
        let pane = PaneInfo {
            id: 8,
            title: "codex".to_owned(),
            terminal_command: Some("codex".to_owned()),
            exited: true,
            exit_status: Some(1),
            ..PaneInfo::default()
        };

        let agent = detect_agent_in_pane(&pane, 0, None, &BTreeSet::new(), &BTreeMap::new())
            .expect("agent should be detected");

        assert_eq!(agent.status, AgentStatus::Exited);
        assert_eq!(agent.exit_status, Some(1));
    }

    #[test]
    fn detects_commands_from_running_command_overrides() {
        let pane = PaneInfo {
            id: 9,
            title: "~/project".to_owned(),
            terminal_command: None,
            ..PaneInfo::default()
        };
        let running_commands = BTreeMap::from([(9, "codex".to_owned())]);

        let agent = detect_agent_in_pane(
            &pane,
            0,
            Some("dev".to_owned()),
            &BTreeSet::new(),
            &running_commands,
        )
        .expect("agent should be detected from command override");

        assert_eq!(agent.kind, AgentKind::Codex);
        assert_eq!(agent.status, AgentStatus::Running);
        assert_eq!(agent.command.as_deref(), Some("codex"));
    }

    #[test]
    fn marks_title_only_detection_as_unknown() {
        let pane = PaneInfo {
            id: 10,
            title: "codex".to_owned(),
            terminal_command: None,
            ..PaneInfo::default()
        };

        let agent = detect_agent_in_pane(&pane, 0, None, &BTreeSet::new(), &BTreeMap::new())
            .expect("agent should be detected");

        assert_eq!(agent.status, AgentStatus::Unknown);
    }
}
