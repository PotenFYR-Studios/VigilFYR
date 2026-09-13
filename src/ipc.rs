//! Shared IPC/event-record types.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::event::{Action, VerdictAction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRecord {
    pub schema: u8,
    pub kind: String,
    pub timestamp: String,
    pub agent: String,
    pub tool: String,
    pub action: Action,
    pub paths: Vec<PathBuf>,
    pub command: Option<String>,
    pub cwd: PathBuf,
    pub session: String,
    pub verdict: VerdictAction,
    pub rule: String,
    pub reason: String,
    pub masked_paths: Vec<PathBuf>,
}

impl IpcRecord {
    pub fn from_verdict(event: &crate::event::Event, verdict: &crate::event::Verdict) -> Self {
        Self {
            schema: event.schema,
            kind: "event".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            agent: event.agent.clone(),
            tool: event.tool.clone(),
            action: event.action,
            paths: event.paths.clone(),
            command: event.command.clone(),
            cwd: event.cwd.clone(),
            session: event.session.clone(),
            verdict: verdict.action,
            rule: verdict.rule.clone(),
            reason: verdict.reason.clone(),
            masked_paths: verdict.masked_paths.clone(),
        }
    }

    pub fn to_line(&self) -> serde_json::Result<String> {
        let mut json = serde_json::to_string(self)?;
        json.push('\n');
        Ok(json)
    }
}
