//! Codex CLI: config at ~/.codex/config.toml; hooks best-effort via a
//! notify block Vigil manages under its own `[vigil]` key.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::{home, AgentDef};
use crate::engine::Mode;

fn config_path() -> PathBuf {
    home().join(".codex").join("config.toml")
}

fn detect() -> bool {
    config_path().is_file()
}

fn install_hook(path: &Path, mode: Mode) -> Result<()> {
    if path != config_path() {
        anyhow::bail!("codex hooks install to {}", config_path().display());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let current = std::fs::read_to_string(path).unwrap_or_default();
    let mut cfg: toml::Value =
        toml::from_str(&current).unwrap_or(toml::Value::Table(Default::default()));
    let table = cfg
        .as_table_mut()
        .context("codex config root is not a table")?;
    let vigil = format!(
        "[vigil]\nversion = 1\nmode = \"{}\"\ncommand = \"vigil intercept\"\n",
        mode
    );
    // Idempotent: parse existing vigil table out, replace wholesale.
    let mut merged = String::new();
    for line in current.lines() {
        if line.trim() == "[vigil]" {
            break;
        }
        merged.push_str(line);
        merged.push('\n');
    }
    merged.push_str(&vigil);
    let _ = table; // kept for structure checks; text merge preserves comments above [vigil]
    std::fs::write(path, merged)?;
    Ok(())
}

fn remove_hook(path: &Path) -> Result<()> {
    let current = std::fs::read_to_string(path).context("reading codex config")?;
    let mut out = String::new();
    let mut in_vigil = false;
    for line in current.lines() {
        if line.trim() == "[vigil]" {
            in_vigil = true;
            continue;
        }
        if in_vigil && line.trim_start().starts_with('[') {
            in_vigil = false;
        }
        if !in_vigil {
            out.push_str(line);
            out.push('\n');
        }
    }
    std::fs::write(path, out)?;
    Ok(())
}

pub fn def() -> AgentDef {
    AgentDef {
        id: "codex",
        display: "Codex CLI",
        detect,
        hook_paths: vec![config_path()],
        install_hook,
        remove_hook,
        supports_hooks: true,
    }
}
