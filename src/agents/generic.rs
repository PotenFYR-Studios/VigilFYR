//! Generic/unknown agent: audit-only, no hook integration.

use std::path::Path;

use anyhow::Result;

use super::AgentDef;

fn detect() -> bool {
    // Never auto-detected; used when the user names an unknown agent.
    false
}

fn install_hook(_path: &Path, _mode: crate::engine::Mode) -> Result<()> {
    Ok(())
}

fn remove_hook(_path: &Path) -> Result<()> {
    Ok(())
}

pub fn def() -> AgentDef {
    AgentDef {
        id: "generic",
        display: "Generic agent (audit-only)",
        detect,
        hook_paths: Vec::new(),
        install_hook,
        remove_hook,
        supports_hooks: false,
    }
}
