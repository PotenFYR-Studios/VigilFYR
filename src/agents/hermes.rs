//! Hermes: detected via `hermes` on PATH; audit-only (no hook surface).

use std::path::Path;

use anyhow::Result;

use super::AgentDef;

fn detect() -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join("hermes").is_file()))
        .unwrap_or(false)
}

fn install_hook(_path: &Path, _mode: crate::engine::Mode) -> Result<()> {
    Ok(())
}

fn remove_hook(_path: &Path) -> Result<()> {
    Ok(())
}

pub fn def() -> AgentDef {
    AgentDef {
        id: "hermes",
        display: "Hermes (audit-only)",
        detect,
        hook_paths: Vec::new(),
        install_hook,
        remove_hook,
        supports_hooks: false,
    }
}
