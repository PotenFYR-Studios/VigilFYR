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

    /// True when a hook-array entry is one Vigil wrote (any nested command
    /// starting with "vigil ").
    fn is_vigil_entry(entry: &Value) -> bool {
        entry["hooks"].as_array().is_some_and(|hs| {
            hs.iter().any(|h| {
                h["command"]
                    .as_str()
                    .is_some_and(|c| c.starts_with("vigil "))
            })
        })
    }

    /// Merge `entry` into `hooks.<event>` (Claude Code native hook shape:
    /// `hooks.PreToolUse: [{matcher, hooks: [{type: "command", command}]}]`).
    /// An existing vigil entry in the array is replaced in place - re-run
    /// updates, never duplicates. All other keys and entries survive.
    pub fn set_hook_entry(text: &str, event: &str, entry: &Value) -> Result<String> {
        let mut v: Value = serde_json::from_str(text)?;
        let obj = v
            .as_object_mut()
            .ok_or_else(|| anyhow!("settings root is not a JSON object"))?;
        let hooks = obj
            .entry("hooks")
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        let hooks_obj = hooks
            .as_object_mut()
            .ok_or_else(|| anyhow!("hooks is not a JSON object"))?;
        let arr = hooks_obj
            .entry(event.to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
        let arr = arr
            .as_array_mut()
            .ok_or_else(|| anyhow!("hooks.{event} is not an array"))?;
        match arr.iter().position(is_vigil_entry) {
            Some(i) => arr[i] = entry.clone(),
            None => arr.push(entry.clone()),
        }
        Ok(serde_json::to_string_pretty(&v)?)
    }

    /// Remove every vigil entry from every `hooks.<event>` array; drops
    /// emptied arrays and the `hooks` key itself when nothing is left.
    /// Non-vigil entries and all other top-level keys survive.
    pub fn strip_hook_entries(text: &str) -> Result<String> {
        let mut v: Value = serde_json::from_str(text)?;
        if let Some(hooks) = v.get_mut("hooks").and_then(|h| h.as_object_mut()) {
            for (_, e) in hooks.iter_mut() {
                if let Some(arr) = e.as_array_mut() {
                    arr.retain(|e| !is_vigil_entry(e));
                }
            }
            let empty_events: Vec<String> = hooks
                .iter()
                .filter(|(_, e)| e.as_array().is_some_and(|a| a.is_empty()))
                .map(|(k, _)| k.clone())
                .collect();
            for event in empty_events {
                hooks.remove(&event);
            }
        }
        if let Some(obj) = v.as_object_mut() {
            if obj
                .get("hooks")
                .and_then(|h| h.as_object())
                .is_some_and(|h| h.is_empty())
            {
                obj.remove("hooks");
            }
        }
        Ok(serde_json::to_string_pretty(&v)?)
    }

    /// Merge flat `{"command": ...}` entries into `hooks.<event>` arrays
    /// (Cursor hooks.json shape). Vigil entries (command starting with
    /// "vigil ") are replaced in place; other entries survive.
    pub fn set_flat_hook_entries(text: &str, entries: &[(&str, &str)]) -> Result<String> {
        let mut v: Value = serde_json::from_str(text)?;
        let obj = v
            .as_object_mut()
            .ok_or_else(|| anyhow!("settings root is not a JSON object"))?;
        let hooks = obj
            .entry("hooks")
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        let hooks_obj = hooks
            .as_object_mut()
            .ok_or_else(|| anyhow!("hooks is not a JSON object"))?;
        for (event, command) in entries {
            let arr = hooks_obj
                .entry((*event).to_string())
                .or_insert_with(|| Value::Array(Vec::new()));
            let arr = arr
                .as_array_mut()
                .ok_or_else(|| anyhow!("hooks.{event} is not an array"))?;
            let new_entry = serde_json::json!({ "command": command });
            match arr.iter().position(|e| {
                e["command"]
                    .as_str()
                    .is_some_and(|c| c.starts_with("vigil "))
            }) {
                Some(i) => arr[i] = new_entry,
                None => arr.push(new_entry),
            }
        }
        Ok(serde_json::to_string_pretty(&v)?)
    }

    /// Remove every vigil entry from every `hooks.<event>` array (flat
    /// shape), dropping emptied arrays.
    pub fn strip_flat_hook_entries(text: &str) -> Result<String> {
        let mut v: Value = serde_json::from_str(text)?;
        if let Some(hooks) = v.get_mut("hooks").and_then(|h| h.as_object_mut()) {
            for (_, e) in hooks.iter_mut() {
                if let Some(arr) = e.as_array_mut() {
                    arr.retain(|e| {
                        !e["command"]
                            .as_str()
                            .is_some_and(|c| c.starts_with("vigil "))
                    });
                }
            }
            let empty: Vec<String> = hooks
                .iter()
                .filter(|(_, e)| e.as_array().is_some_and(|a| a.is_empty()))
                .map(|(k, _)| k.clone())
                .collect();
            for event in empty {
                hooks.remove(&event);
            }
        }
        Ok(serde_json::to_string_pretty(&v)?)
    }
}
