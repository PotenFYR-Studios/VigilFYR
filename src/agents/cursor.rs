//! Cursor: detected via ~/.cursor/; hooks go into ~/.cursor/hooks.json
//! (Cursor's native hook config: {hooks: {beforeReadFile: [{command}]}}).

use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{home, json_settings, AgentDef};
use crate::engine::Mode;

fn dir() -> PathBuf {
    home().join(".cursor")
}

fn settings_path() -> PathBuf {
    dir().join("hooks.json")
}

fn detect() -> bool {
    dir().is_dir()
}

/// Cursor's documented hook events for file reads/writes.
const CURSOR_EVENTS: &[&str] = &["beforeReadFile", "beforeWriteFile"];

fn install_hook(path: &Path, _mode: Mode) -> Result<()> {
    if path != settings_path() {
        anyhow::bail!("cursor hooks install to {}", settings_path().display());
    }
    std::fs::create_dir_all(dir())?;
    let current = std::fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
    let entries: Vec<(&str, &str)> = CURSOR_EVENTS
        .iter()
        .map(|e| (*e, "vigil intercept --agent cursor"))
        .collect();
    let updated = json_settings::set_flat_hook_entries(&current, &entries)?;
    std::fs::write(path, updated + "\n")?;
    Ok(())
}

fn remove_hook(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let current = std::fs::read_to_string(path)?;
    let updated = json_settings::strip_flat_hook_entries(&current)?;
    std::fs::write(path, updated + "\n")?;
    Ok(())
}

pub fn def() -> AgentDef {
    AgentDef {
        id: "cursor",
        display: "Cursor",
        detect,
        hook_paths: vec![settings_path()],
        install_hook,
        remove_hook,
        supports_hooks: true,
    }
}
