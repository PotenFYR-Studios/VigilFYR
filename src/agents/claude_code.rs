//! Claude Code: hooks live in ~/.claude/settings.json (PreToolUse).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::json;

use super::{home, json_settings, AgentDef};
use crate::engine::Mode;

fn settings_path() -> PathBuf {
    home().join(".claude").join("settings.json")
}

fn detect() -> bool {
    settings_path().is_file()
}

/// Native Claude Code hook shape: hooks.PreToolUse[] entry. Mode is not
/// encoded here; `vigil intercept` reads it from config at call time.
fn pre_tool_use_entry() -> serde_json::Value {
    json!({
        "matcher": "*",
        "hooks": [{ "type": "command", "command": "vigil intercept --agent claude-code" }]
    })
}

fn install_hook(path: &Path, _mode: Mode) -> Result<()> {
    if path != settings_path() {
        anyhow::bail!("claude-code hooks install to {}", settings_path().display());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let current = std::fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
    let updated = json_settings::set_hook_entry(&current, "PreToolUse", &pre_tool_use_entry())
        .context("merging vigil hook into settings.json")?;
    std::fs::write(path, updated + "\n")?;
    Ok(())
}

fn remove_hook(path: &Path) -> Result<()> {
    let current = std::fs::read_to_string(path).context("reading settings.json")?;
    let updated = json_settings::strip_hook_entries(&current)
        .context("stripping vigil hook from settings.json")?;
    std::fs::write(path, updated + "\n")?;
    Ok(())
}

pub fn def() -> AgentDef {
    AgentDef {
        id: "claude-code",
        display: "Claude Code",
        detect,
        hook_paths: vec![settings_path()],
        install_hook,
        remove_hook,
        supports_hooks: true,
    }
}
