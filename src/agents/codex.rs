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
    let merged = set_vigil_table(&current, mode)?;
    std::fs::write(path, merged)?;
    Ok(())
}

/// Rewrite the `[vigil]` table in `text`, preserving every other line —
/// including content after the block (other tables, comments). Idempotent.
fn set_vigil_table(text: &str, mode: Mode) -> Result<String> {
    let vigil_block = format!(
        "[vigil]\nversion = 1\nmode = \"{}\"\ncommand = \"vigil intercept\"\n",
        mode
    );
    let mut out = String::with_capacity(text.len() + vigil_block.len());
    let mut in_vigil = false;
    let mut vigil_seen = false;
    for line in text.lines() {
        let is_header = line.trim_start().starts_with('[');
        if is_header {
            let name = line.trim().trim_start_matches('[').trim_end_matches(']');
            if name == "vigil" {
                // Replace or drop the existing block; write the new one at
                // the position of the first occurrence only.
                in_vigil = true;
                if !vigil_seen {
                    vigil_seen = true;
                    out.push_str(&vigil_block);
                }
                continue;
            }
            in_vigil = false;
        }
        if !in_vigil {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !vigil_seen {
        if !out.is_empty() && !out.ends_with("\n\n") {
            out.push('\n');
        }
        out.push_str(&vigil_block);
    }
    // Sanity: result must still parse as TOML.
    let _: toml::Value =
        toml::from_str(&out).context("merged codex config must stay valid TOML")?;
    Ok(out)
}

fn remove_hook(path: &Path) -> Result<()> {
    let current = std::fs::read_to_string(path).context("reading codex config")?;
    let mut out = String::new();
    let mut in_vigil = false;
    for line in current.lines() {
        if line.trim_start().starts_with('[') {
            let name = line.trim().trim_start_matches('[').trim_end_matches(']');
            in_vigil = name == "vigil";
            if in_vigil {
                continue;
            }
        }
        if !in_vigil {
            out.push_str(line);
            out.push('\n');
        }
    }
    let _: toml::Value =
        toml::from_str(&out).context("stripped codex config must stay valid TOML")?;
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
