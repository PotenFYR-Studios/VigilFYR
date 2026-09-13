//! Event ingestion types: hook input from agents, and verdict output.

use std::io::Read;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Event {
    pub schema: u8,
    pub agent: String,
    pub tool: String,
    pub action: Action,
    pub paths: Vec<PathBuf>,
    pub command: Option<String>,
    pub cwd: PathBuf,
    pub session: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Read,
    Write,
    Search,
    Exec,
    Net,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Action::Read => "read",
            Action::Write => "write",
            Action::Search => "search",
            Action::Exec => "exec",
            Action::Net => "net",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Serialize)]
pub struct Verdict {
    pub action: VerdictAction,
    pub rule: String,
    pub reason: String,
    pub masked_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VerdictAction {
    Deny,
    Allow,
    Warn,
    Mask,
}

impl std::fmt::Display for VerdictAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            VerdictAction::Deny => "deny",
            VerdictAction::Allow => "allow",
            VerdictAction::Warn => "warn",
            VerdictAction::Mask => "mask",
        };
        f.write_str(s)
    }
}

impl Event {
    pub fn from_reader<R: Read>(r: R) -> serde_json::Result<Event> {
        serde_json::from_reader(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_event_from_json() {
        let json = r#"{"schema":1,"agent":"claude-code","tool":"Read","action":"read",
                       "paths":["/repo/.env"],"command":null,"cwd":"/repo","session":"s1"}"#;
        let e = Event::from_reader(json.as_bytes()).unwrap();
        assert_eq!(e.action, Action::Read);
        assert_eq!(e.paths, vec![PathBuf::from("/repo/.env")]);
    }
}
