//! `vigil rules` subcommands and `vigil reload`.

use std::collections::BTreeMap;

use anyhow::Result;

use vigil::config::{AgentMode, Config};
use vigil::sync::{load_effective_rules, rules_dirs, sync_remote_rules};

/// Summary of reloadable state: rules by source plus known agents.
/// A free function on purpose — the daemon (later task) calls the same
/// seam on SIGHUP/IPC reload.
pub struct ReloadSummary {
    pub total_rules: usize,
    pub rules_by_source: BTreeMap<String, usize>,
    pub agents: BTreeMap<String, AgentMode>,
}

/// Re-run rule loading and agent detection. Agent detection lands with
/// the intercept task; for now agents come from the `[agents]` config map.
pub fn reload_state(cfg: &Config) -> ReloadSummary {
    let ruleset = load_effective_rules(cfg);
    let mut rules_by_source: BTreeMap<String, usize> = BTreeMap::new();
    for r in &ruleset.rules {
        *rules_by_source.entry(r.source.clone()).or_default() += 1;
    }
    ReloadSummary {
        total_rules: ruleset.rules.len(),
        rules_by_source,
        agents: cfg.agents.iter().map(|(k, v)| (k.clone(), *v)).collect(),
    }
}

pub fn print_reload_summary(s: &ReloadSummary) {
    let by_source = s
        .rules_by_source
        .iter()
        .map(|(k, v)| format!("{k}: {v}"))
        .collect::<Vec<_>>()
        .join(", ");
    println!(
        "{} rules ({by_source}), {} agents",
        s.total_rules,
        s.agents.len()
    );
    for (id, mode) in &s.agents {
        println!("  {id}: {mode}");
    }
}

/// `vigil rules list` — table of id, action, severity, scope, source.
pub fn rules_list(cfg: &Config) -> Result<()> {
    let ruleset = load_effective_rules(cfg);
    println!(
        "{:<38} {:<7} {:<10} {:<20} SOURCE",
        "ID", "ACTION", "SEVERITY", "SCOPE"
    );
    for r in &ruleset.rules {
        let scope: Vec<String> = r.scope.iter().map(|a| a.to_string()).collect();
        println!(
            "{:<38} {:<7} {:<10} {:<20} {}",
            r.id,
            r.action,
            r.severity,
            scope.join(","),
            r.source
        );
    }
    println!("{} rules", ruleset.rules.len());
    Ok(())
}

/// `vigil rules path` — print the rules directories in precedence order
/// (builtin is compiled in, not shown as a path).
pub fn rules_path() -> Result<()> {
    for (source, dir) in rules_dirs() {
        let marker = if dir.is_dir() { "" } else { " (missing)" };
        println!("{:<8} {}{}", source, dir.display(), marker);
    }
    Ok(())
}

/// `vigil rules update` — manual remote sync; never fails on network.
pub fn rules_update(cfg: &Config) -> Result<()> {
    let note = sync_remote_rules(cfg)?;
    println!("{note}");
    Ok(())
}

/// `vigil reload` — reload rules + agents, print summary.
pub fn reload(cfg: &Config) -> Result<()> {
    let s = reload_state(cfg);
    print_reload_summary(&s);
    Ok(())
}
