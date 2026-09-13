//! `vigil rules` subcommands and `vigil reload`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;

use vigil::config::{AgentMode, Config};
use vigil::ext::load_extensions;
use vigil::sync::{load_effective_rules, rules_dirs, sync_remote_rules};

/// Summary of reloadable state: rules by source plus known agents.
/// A free function on purpose - the daemon (later task) calls the same
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

fn source_summary(s: &ReloadSummary) -> String {
    s.rules_by_source
        .iter()
        .map(|(k, v)| format!("{k}: {v}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn print_reload_summary(s: &ReloadSummary) {
    let by_source = source_summary(s);
    println!(
        "{} rules ({by_source}), {} agents",
        s.total_rules,
        s.agents.len()
    );
    for (id, mode) in &s.agents {
        println!("  {id}: {mode}");
    }
}

/// `vigil rules list` - table of id, action, severity, scope, source.
pub fn rules_list(cfg: &Config) -> Result<()> {
    let ruleset = load_effective_rules(cfg);
    let extensions = load_extensions(vigil::ext::extensions_root());
    if !extensions.is_empty() {
        println!("EXTENSIONS");
        for extension in &extensions {
            println!(
                "{}\t{}\t{}",
                extension.manifest.name,
                extension.manifest.version,
                extension.root.display()
            );
        }
    }
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

/// `vigil rules path` - print the rules directories in precedence order
/// (builtin is compiled in, not shown as a path).
pub fn rules_path() -> Result<()> {
    for (source, dir) in rules_dirs() {
        let marker = if dir.is_dir() { "" } else { " (missing)" };
        println!("{:<8} {}{}", source, dir.display(), marker);
    }
    Ok(())
}

/// `vigil rules update` - manual remote sync; never fails on network.
pub fn rules_update(cfg: &Config) -> Result<()> {
    let note = sync_remote_rules(cfg)?;
    println!("{note}");
    Ok(())
}

/// `vigil rules test` - evaluate a synthetic event without touching a
/// protected agent or writing the event log.
pub fn rules_test(args: RuleTestArgs) -> Result<()> {
    let cfg = Config::load();
    let ruleset = load_effective_rules(&cfg);
    let event = vigil::event::Event {
        schema: 1,
        agent: args.agent.unwrap_or_else(|| "test".to_string()),
        tool: args.tool.unwrap_or_else(|| "manual".to_string()),
        action: args.action,
        paths: args.paths.into_iter().map(PathBuf::from).collect(),
        command: args.command,
        cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        session: "rules-test".to_string(),
    };
    let verdict = ruleset.evaluate(&event, cfg.general.mode);
    println!("{}\t{}\t{}", verdict.action, verdict.rule, verdict.reason);
    Ok(())
}

/// `vigil reload` - reload rules + agents, print summary.
pub fn reload(cfg: &Config) -> Result<()> {
    let s = reload_state(cfg);
    print_reload_summary(&s);
    Ok(())
}

/// `vigil doctor` - one-shot health report for CI, setup debugging and
/// support workflows. Every check is read-only.
pub fn doctor() -> Result<()> {
    let cfg = Config::load();
    let summary = reload_state(&cfg);
    let detected = vigil::agents::detect_agents();
    let extensions = load_extensions(vigil::ext::extensions_root());
    let masking = if cfg.masking.enabled { "on" } else { "off" };
    let mode = if cfg.general.enabled {
        cfg.general.mode.to_string()
    } else {
        "disabled".to_string()
    };

    println!("mode: {mode}");
    println!(
        "rules: {} ({})",
        summary.total_rules,
        source_summary(&summary)
    );
    println!("masking: {masking}");
    println!("agents detected: {}", detected.len());
    for agent in &detected {
        println!(
            "  {}: {} ({})",
            agent.id,
            agent.display,
            cfg.agents
                .get(agent.id)
                .map(|mode| mode.to_string())
                .unwrap_or_else(|| "unconfigured".to_string())
        );
    }
    println!("extensions: {}", extensions.len());
    let log_path = vigil::agents::home().join(".vigil/events.jsonl");
    let log = vigil::log::EventLog::new(log_path)
        .with_retention(Some(cfg.rules.retention_days), Some(cfg.rules.max_events));
    let records = log.records()?;
    let counters = vigil::tui::Counters::from_records(&records);
    println!(
        "events: {} retained (denied {}, warned {}, masked {})",
        counters.total, counters.denied, counters.warned, counters.masked
    );

    let mut failures = Vec::new();
    if !cfg.general.enabled {
        failures.push("Vigil is disabled".to_string());
    }
    if summary.total_rules == 0 {
        failures.push("no rules loaded".to_string());
    }
    if !cfg.rules.remote_update {
        failures.push("remote rule updates disabled".to_string());
    }
    if extensions
        .iter()
        .any(|extension| !extension.root.join("manifest.toml").is_file())
    {
        failures.push("extension missing manifest".to_string());
    }
    if let Some(room) = (cfg.rules.max_events > 0)
        .then(|| cfg.rules.max_events.saturating_sub(records.len()))
        .filter(|room| *room <= 100)
    {
        failures.push(format!("event capacity nearly full ({room} remaining)"));
    }
    if failures.is_empty() {
        println!("status: ok");
    } else {
        println!("status: attention");
        for failure in failures {
            println!("  {failure}");
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct RuleTestArgs {
    pub action: vigil::event::Action,
    pub paths: Vec<String>,
    pub command: Option<String>,
    pub agent: Option<String>,
    pub tool: Option<String>,
}
