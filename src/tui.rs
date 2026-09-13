//! TUI render model and headless-safe snapshot/export helpers.

use std::collections::BTreeMap;
use std::fs;
use std::io::IsTerminal;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::Result;

use crate::ipc::IpcRecord;

#[derive(Debug, Default, Clone)]
pub struct Counters {
    pub total: usize,
    pub denied: usize,
    pub warned: usize,
    pub masked: usize,
    by_agent: BTreeMap<String, usize>,
    by_tool: BTreeMap<String, usize>,
    by_action: BTreeMap<String, usize>,
    by_verdict: BTreeMap<String, usize>,
}

impl Counters {
    pub fn from_records(records: &[IpcRecord]) -> Self {
        let mut counters = Self::default();
        for record in records {
            counters.total += 1;
            match record.verdict {
                crate::event::VerdictAction::Deny => counters.denied += 1,
                crate::event::VerdictAction::Warn => counters.warned += 1,
                crate::event::VerdictAction::Mask => counters.masked += 1,
                crate::event::VerdictAction::Allow => {}
            }
            *counters.by_agent.entry(record.agent.clone()).or_default() += 1;
            *counters.by_tool.entry(record.tool.clone()).or_default() += 1;
            *counters
                .by_action
                .entry(record.action.to_string())
                .or_default() += 1;
            *counters
                .by_verdict
                .entry(record.verdict.to_string())
                .or_default() += 1;
        }
        counters
    }

    pub fn by_agent(&self) -> &BTreeMap<String, usize> {
        &self.by_agent
    }
    pub fn by_tool(&self) -> &BTreeMap<String, usize> {
        &self.by_tool
    }
    pub fn by_action(&self) -> &BTreeMap<String, usize> {
        &self.by_action
    }
    pub fn by_verdict(&self) -> &BTreeMap<String, usize> {
        &self.by_verdict
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRow {
    pub timestamp: String,
    pub agent: String,
    pub action: String,
    pub target: String,
    pub verdict: String,
    pub rule: String,
    pub severity: String,
    pub reason: String,
}

fn target(record: &IpcRecord) -> String {
    record
        .command
        .clone()
        .or_else(|| record.paths.first().map(|p| p.display().to_string()))
        .unwrap_or_default()
}

pub fn row(record: &IpcRecord) -> EventRow {
    EventRow {
        timestamp: record.timestamp.clone(),
        agent: record.agent.clone(),
        action: record.action.to_string(),
        target: target(record),
        verdict: record.verdict.to_string(),
        rule: record.rule.clone(),
        severity: rule_severity(record),
        reason: record.reason.clone(),
    }
}

fn rule_severity(record: &IpcRecord) -> String {
    let lower = record.reason.to_lowercase();
    ["low", "medium", "high", "critical"]
        .into_iter()
        .find(|severity| lower.contains(severity))
        .unwrap_or("unknown")
        .to_string()
}

pub fn visible(records: &[IpcRecord], filter: &str) -> Vec<EventRow> {
    let filter = filter.to_lowercase();
    records
        .iter()
        .filter(|r| {
            filter.is_empty() || {
                let row = row(r);
                format!(
                    "{} {} {} {} {} {}",
                    row.timestamp, row.agent, row.action, row.target, row.verdict, row.rule
                )
                .to_lowercase()
                .contains(&filter)
            }
        })
        .map(row)
        .collect()
}

pub fn export(records: &[IpcRecord], format: &str) -> Result<PathBuf> {
    let dir = PathBuf::from("./vigil-export");
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("events.{format}"));
    match format {
        "json" => fs::write(&path, serde_json::to_string_pretty(records)?)?,
        "csv" => {
            let mut out =
                String::from("timestamp,agent,tool,action,target,verdict,rule,severity,reason\n");
            for row in visible(records, "") {
                out.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{}\n",
                    row.timestamp.replace(',', ";"),
                    row.agent,
                    target(
                        records
                            .iter()
                            .find(|record| record.timestamp == row.timestamp)
                            .expect("row came from records"),
                    ),
                    row.action,
                    row.target.replace(',', ";"),
                    row.verdict,
                    row.rule,
                    row.severity,
                    row.reason.replace(',', ";")
                ));
            }
            fs::write(&path, out)?;
        }
        _ => anyhow::bail!("unsupported export format: {format}"),
    }
    Ok(path)
}

pub fn report_records(records: &[IpcRecord]) -> String {
    let counters = Counters::from_records(records);
    let mut lines = vec![
        format!("total: {}", counters.total),
        format!("denied: {}", counters.denied),
        format!("warned: {}", counters.warned),
        format!("masked: {}", counters.masked),
    ];
    for (verdict, count) in counters.by_verdict() {
        lines.push(format!("verdict.{verdict}: {count}"));
    }
    for (agent, count) in counters.by_agent() {
        lines.push(format!("agent.{agent}: {count}"));
    }
    for (action, count) in counters.by_action() {
        lines.push(format!("action.{action}: {count}"));
    }

    let mut out = lines.join("\n");
    out.push_str(
        "\n\nTIME | AGENT | TOOL | ACTION | TARGET | VERDICT | RULE | SEVERITY | REASON\n",
    );
    for record in records {
        out.push_str(&format!(
            "{} | {} | {} | {} | {} | {} | {} | {} | {}\n",
            record.timestamp,
            record.agent,
            record.tool,
            record.action,
            target(record),
            record.verdict,
            record.rule,
            rule_severity(record),
            record.reason.replace('\n', " ")
        ));
    }
    out
}

pub fn snapshot(records: &[IpcRecord]) -> String {
    let mut out = String::from("TIME | AGENT | ACTION | TARGET | VERDICT | RULE\n");
    for row in visible(records, "")
        .iter()
        .rev()
        .take(50)
        .collect::<Vec<_>>()
        .iter()
        .rev()
    {
        out.push_str(&format!(
            "{} | {} | {} | {} | {} | {}\n",
            row.timestamp, row.agent, row.action, row.target, row.verdict, row.rule
        ));
    }
    out
}

pub fn run_once(once: bool, report: bool) -> Result<()> {
    if !once {
        run_interactive()?;
        return Ok(());
    }
    let cfg = crate::config::Config::load();
    let log = crate::log::EventLog::new(crate::agents::home().join(".vigil/events.jsonl"))
        .with_retention(Some(cfg.rules.retention_days), Some(cfg.rules.max_events));
    let records = log.records()?;
    print!(
        "{}",
        if report {
            report_records(&records)
        } else {
            snapshot(&records)
        }
    );
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Events,
    Agents,
    Rules,
}

impl Tab {
    pub fn from_key(key: char) -> Option<Self> {
        match key {
            '1' => Some(Self::Dashboard),
            '2' => Some(Self::Events),
            '3' => Some(Self::Agents),
            '4' => Some(Self::Rules),
            _ => None,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Events => "Events",
            Self::Agents => "Agents",
            Self::Rules => "Rules",
        }
    }
}

#[derive(Debug)]
pub struct TuiState {
    pub tab: Tab,
    pub selected: usize,
    pub filter: String,
    pub editing_filter: bool,
    pub records: Vec<IpcRecord>,
    pub agents: Vec<String>,
    pub rules: Vec<String>,
    pub status: String,
}

impl TuiState {
    pub fn new(records: Vec<IpcRecord>, agents: Vec<String>, rules: Vec<String>) -> Self {
        Self {
            tab: Tab::Dashboard,
            selected: 0,
            filter: String::new(),
            editing_filter: false,
            records,
            agents,
            rules,
            status: "1-4 tabs | / filter | e export | q quit".to_string(),
        }
    }

    pub fn rows(&self) -> Vec<EventRow> {
        visible(&self.records, &self.filter)
    }

    pub fn counters(&self) -> Counters {
        Counters::from_records(&self.records)
    }

    pub fn handle_key(&mut self, code: char) -> bool {
        match code {
            'q' if !self.editing_filter => return false,
            '/' => self.editing_filter = true,
            'j' if !self.editing_filter => {
                self.selected = (self.selected + 1).min(self.rows().len().saturating_sub(1));
            }
            'k' if !self.editing_filter => self.selected = self.selected.saturating_sub(1),
            'e' if !self.editing_filter => match export(&self.records, "json") {
                Ok(path) => self.status = format!("exported {}", path.display()),
                Err(error) => self.status = format!("export failed: {error}"),
            },
            _ => {}
        }
        true
    }

    pub fn render(&self) -> String {
        let mut lines = vec![
            format!(
                "[1] Dashboard [2] Events [3] Agents [4] Rules | tab={} filter={}",
                self.tab.title(),
                if self.filter.is_empty() {
                    "-"
                } else {
                    &self.filter
                }
            ),
            self.status.clone(),
        ];
        match self.tab {
            Tab::Dashboard => {
                let counters = self.counters();
                lines.push(format!("total events: {}", counters.total));
                for (agent, count) in counters.by_agent() {
                    lines.push(format!("{agent}: {count}"));
                }
            }
            Tab::Events => lines.push(snapshot(&self.records).trim().to_string()),
            Tab::Agents => lines.extend(self.agents.iter().cloned()),
            Tab::Rules => lines.extend(self.rules.iter().cloned()),
        }
        lines.push("j/k navigate | / filter | e export | q quit".to_string());
        lines.join("\n")
    }
}

pub fn run_interactive() -> Result<()> {
    if !io::stdout().is_terminal() {
        anyhow::bail!("interactive TUI requires a TTY; use --once in scripts");
    }
    let log = crate::log::EventLog::new(crate::agents::home().join(".vigil/events.jsonl"));
    let mut state = TuiState::new(log.tail(1000)?, Vec::new(), Vec::new());
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    let _ = crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen);
    loop {
        stdout.write_all(state.render().as_bytes())?;
        stdout.write_all(b"\n")?;
        stdout.flush()?;
        if crossterm::event::poll(std::time::Duration::from_millis(250))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    if let crossterm::event::KeyCode::Char(c) = key.code {
                        if let Some(tab) = Tab::from_key(c) {
                            state.tab = tab;
                            state.selected = 0;
                        } else if !state.handle_key(c) {
                            break;
                        }
                    } else if key.code == crossterm::event::KeyCode::Esc {
                        state.editing_filter = false;
                    }
                }
            }
        }
        state.records = log.tail(1000)?;
    }
    let _ = crossterm::execute!(stdout, crossterm::terminal::LeaveAlternateScreen);
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Action, VerdictAction};

    fn record() -> IpcRecord {
        IpcRecord {
            schema: 1,
            kind: "event".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            agent: "claude-code".to_string(),
            tool: "Read".to_string(),
            action: Action::Read,
            paths: vec![PathBuf::from("/repo/.env")],
            command: None,
            cwd: PathBuf::from("/repo"),
            session: "s".to_string(),
            verdict: VerdictAction::Deny,
            rule: "deny-env".to_string(),
            reason: "test".to_string(),
            masked_paths: Vec::new(),
        }
    }

    #[test]
    fn builds_rows_and_counters() {
        let counters = Counters::from_records(&[record(), record()]);
        assert_eq!(counters.total, 2);
        assert_eq!(counters.denied, 2);
        assert_eq!(counters.by_verdict().get("deny"), Some(&2));
        assert_eq!(counters.by_agent().get("claude-code"), Some(&2));
        let rows = visible(&[record()], "deny-env");
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn report_includes_counts_and_details() {
        let report = report_records(&[record()]);
        assert!(report.contains("total: 1"));
        assert!(report.contains("denied: 1"));
        assert!(report.contains("| deny-env"));
        assert!(report.contains("SEVERITY"));
        assert!(report.contains("test"));
    }

    #[test]
    fn state_tabs_filter_navigation_and_render() {
        let mut state = TuiState::new(
            vec![record()],
            vec!["claude-code".into()],
            vec!["deny-env".into()],
        );
        assert_eq!(state.rows().len(), 1);
        state.handle_key('2');
        state.handle_key('/');
        state.filter.push_str("deny");
        assert_eq!(state.rows().len(), 1);
        assert!(state.render().contains("total events") || state.render().contains("deny-env"));
        state.editing_filter = false;
        if let Some(tab) = Tab::from_key('4') {
            state.tab = tab;
        }
        assert!(matches!(state.tab, Tab::Rules));
        assert!(state.render().contains("deny-env"));
        assert!(!state.handle_key('q'));
    }
}
