//! Cursor: detected via ~/.cursor/; hooks best-effort via Vigil's own
//! settings file inside the Cursor dir (Cursor has no native hook API).

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde_json::json;

use super::{home, json_settings, AgentDef};
use crate::engine::Mode;

fn dir() -> PathBuf {
    home().join(".cursor")
}

fn settings_path() -> PathBuf {
    dir().join("vigil.json")
}

fn detect() -> bool {
    dir().is_dir()
}

fn install_hook(path: &Path, mode: Mode) -> Result<()> {
    if path != settings_path() {
        anyhow::bail!("cursor hooks install to {}", settings_path().display());
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
    if path.exists() {
        std::fs::remove_file(path)?;
    }
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
