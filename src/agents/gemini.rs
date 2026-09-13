//! Gemini CLI: detected via ~/.gemini/; hooks best-effort via a settings
//! file Vigil manages at ~/.gemini/settings.json.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde_json::json;

use super::{home, json_settings, AgentDef};
use crate::engine::Mode;

fn dir() -> PathBuf {
    home().join(".gemini")
}

fn settings_path() -> PathBuf {
    dir().join("settings.json")
}

fn detect() -> bool {
    dir().is_dir()
}

fn install_hook(path: &Path, mode: Mode) -> Result<()> {
    if path != settings_path() {
        anyhow::bail!("gemini hooks install to {}", settings_path().display());
    }
    std::fs::create_dir_all(dir())?;
    let current = std::fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
    let block = json!({
        "version": 1,
        "mode": mode.to_string(),
        "intercept": { "type": "command", "command": "vigil intercept" }
    });
    let updated = json_settings::set_vigil(&current, &block)?;
    std::fs::write(path, updated + "\n")?;
    Ok(())
}

fn remove_hook(path: &Path) -> Result<()> {
    let current = std::fs::read_to_string(path)?;
    let updated = json_settings::strip_vigil(&current)?;
    std::fs::write(path, updated + "\n")?;
    Ok(())
}

pub fn def() -> AgentDef {
    AgentDef {
        id: "gemini",
        display: "Gemini CLI",
        detect,
        hook_paths: vec![settings_path()],
        install_hook,
        remove_hook,
        supports_hooks: true,
    }
}
