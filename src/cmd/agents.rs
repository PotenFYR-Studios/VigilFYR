//! `vigil agents list|install|remove` — hook lifecycle per agent.

use anyhow::{bail, Result};

use vigil::agents::{all_agents, detect_agents, AgentDef};
use vigil::config::Config;
use vigil::engine::Mode;

/// Resolve the requested ids (one agent id, "all", or none = all detected).
fn resolve_targets(which: Option<&str>) -> Result<Vec<AgentDef>> {
    let known = all_agents();
    match which {
        None | Some("all") => Ok(known),
        Some(id) => match known.iter().find(|a| a.id == id) {
            Some(a) => Ok(vec![a.clone()]),
            None => bail!(
                "unknown agent '{}'; known: {}",
                id,
                known.iter().map(|a| a.id).collect::<Vec<_>>().join(", ")
            ),
        },
    }
}

/// `vigil agents list` — table of id, hooks support, detection, install state.
pub fn agents_list() -> Result<()> {
    let known = all_agents();
    let detected = detect_agents();
    let cfg = Config::load();
    println!(
        "{:<12} {:<28} {:<9} {:<9} INSTALLED",
        "ID", "AGENT", "HOOKS", "DETECTED"
    );
    for a in &known {
        let det = detected.iter().any(|d| d.id == a.id);
        let installed = a
            .hook_paths
            .iter()
            .any(|p| p.exists() && std::fs::read_to_string(p).is_ok_and(|t| t.contains("vigil ")));
        let mode = cfg.agents.get(a.id).copied();
        let agent_mode = match mode {
            Some(m) if !det => m.to_string(),
            _ => vigil::config::AgentMode::Enforce.to_string(),
        };
        println!(
            "{:<12} {:<28} {:<9} {:<9} {}",
            a.id,
            a.display,
            if a.supports_hooks { "yes" } else { "no" },
            if det { "yes" } else { "no" },
            if installed { &agent_mode } else { "-" },
        );
    }
    Ok(())
}

/// `vigil agents install [agent|all]` — install/update hooks. Agents
/// without hook surfaces get printed guidance and audit-mode marking.
pub fn agents_install(which: Option<&str>) -> Result<()> {
    let targets = resolve_targets(which)?;
    for a in &targets {
        let detected = detect_agents().iter().any(|d| d.id == a.id);
        if !a.supports_hooks || a.hook_paths.is_empty() {
            println!(
                "{}: no hook API; running in audit mode — use `vigil shim -- <cmd>` to monitor it",
                a.id
            );
            continue;
        }
        if !detected {
            println!(
                "{}: not detected; skipping (install {} first)",
                a.id, a.display
            );
            continue;
        }
        let mode = Mode::Enforce;
        for path in &a.hook_paths {
            (a.install_hook)(path, mode)?;
            println!("{}: hooks installed in {}", a.id, path.display());
        }
    }
    Ok(())
}

/// `vigil agents remove [agent|all]` — remove vigil hook entries only.
pub fn agents_remove(which: Option<&str>) -> Result<()> {
    let targets = resolve_targets(which)?;
    for a in &targets {
        if !a.supports_hooks || a.hook_paths.is_empty() {
            continue;
        }
        for path in &a.hook_paths {
            if !path.exists() {
                continue;
            }
            (a.remove_hook)(path)?;
            println!("{}: vigil hooks removed from {}", a.id, path.display());
        }
    }
    Ok(())
}
