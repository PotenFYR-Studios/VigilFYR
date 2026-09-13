//! Agent registry: known AI agents, filesystem detection, hook installers.

pub mod claude_code;
pub mod codex;
pub mod cursor;
pub mod gemini;
pub mod generic;
pub mod hermes;
pub mod opencode;

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::engine::Mode;

/// A known agent: how to detect it and how to (un)install its hook.
#[derive(Debug, Clone)]
pub struct AgentDef {
    pub id: &'static str,
    pub display: &'static str,
    pub detect: fn() -> bool,
    pub hook_paths: Vec<PathBuf>,
    pub install_hook: fn(&Path, Mode) -> Result<()>,
    pub remove_hook: fn(&Path) -> Result<()>,
    pub supports_hooks: bool,
}

/// Every agent Vigil knows about, detectable or not.
pub fn all_agents() -> Vec<AgentDef> {
    vec![
        claude_code::def(),
        codex::def(),
        gemini::def(),
        cursor::def(),
        opencode::def(),
        hermes::def(),
        generic::def(),
    ]
}

/// Detected agents only (config-dir/binary presence), registry order.
pub fn detect_agents() -> Vec<AgentDef> {
    all_agents().into_iter().filter(|a| (a.detect)()).collect()
}

/// Home-relative helper shared by the per-agent modules.
pub fn home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// True when `path` exists (dir or file).
pub fn exists(path: &Path) -> bool {
    path.exists()
}

/// Heredoc-free settings mutation helpers live in the per-agent modules;
/// this shared helper does idempotent JSON key insertion for agents whose
/// settings are JSON (claude-code today).
pub(crate) mod json_settings {
    use anyhow::{anyhow, Result};
    use serde_json::Value;

    /// Extract the vigil hook block back out (for remove).
    pub fn strip_vigil(text: &str) -> Result<String> {
        let mut v: Value = serde_json::from_str(text)?;
        if let Some(obj) = v.as_object_mut() {
            obj.remove("vigil");
        }
        Ok(serde_json::to_string_pretty(&v)?)
    }

    /// Insert or replace the top-level `vigil` key; keeps all other keys.
    pub fn set_vigil(text: &str, vigil_block: &Value) -> Result<String> {
        let mut v: Value = serde_json::from_str(text)?;
        let obj = v
            .as_object_mut()
            .ok_or_else(|| anyhow!("settings root is not a JSON object"))?;
        obj.insert("vigil".to_string(), vigil_block.clone());
        Ok(serde_json::to_string_pretty(&v)?)
    }
}
