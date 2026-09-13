//! `vigil shim` — monitoring wrapper for agents (and plain commands)
//! without hook APIs. Spawns the child with a notify watcher over the
//! loaded rules' paths and emits event records for observed filesystem
//! activity; in enforce mode denied paths are logged as deny events.
//!
//! This is deliberately a monitoring wrapper: blocking the child's own
//! syscalls from userspace is out of scope. For real enforcement, install
//! hooks (`vigil agents install`).

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use notify::{RecursiveMode, Watcher};

use vigil::config::Config;
use vigil::engine::Mode;
use vigil::event::{Action, VerdictAction};
use vigil::mask::select_patterns;
use vigil::sync::load_effective_rules;

#[derive(Parser)]
pub struct ShimArgs {
    /// Working directory for the child and the watch root.
    #[arg(long, default_value = ".")]
    cwd: PathBuf,
    /// Watch directories recursively (defaults to the cwd).
    #[arg(long)]
    watch: Vec<PathBuf>,
    /// Print the JSON event records for observed events on stdout.
    #[arg(long, default_value_t = true)]
    events: bool,
    /// The command to run under the shim, after `--`.
    #[arg(last = true)]
    pub command: Vec<String>,
}

/// One observed filesystem event, in IPC-record shape (schema 1).
fn event_record(
    action: Action,
    paths: &[PathBuf],
    cwd: &std::path::Path,
    rule: &str,
    verdict: VerdictAction,
    masked_paths: &[PathBuf],
) -> serde_json::Value {
    serde_json::json!({
        "schema": 1,
        "event": "fs",
        "agent": "shim",
        "tool": "shim",
        "action": action.to_string(),
        "paths": paths,
        "cwd": cwd,
        "verdict": verdict.to_string(),
        "rule": rule,
        "masked_paths": masked_paths,
    })
}

/// `vigil shim -- <cmd> …` — run the child, watch, record.
pub fn shim(args: ShimArgs, cmd: &[String]) -> Result<()> {
    if cmd.is_empty() {
        anyhow::bail!("usage: vigil shim [--cwd DIR] -- <cmd> [args…]");
    }
    std::env::set_current_dir(&args.cwd)?;

    let cfg = Config::load();
    let mode = cfg.general.mode;
    let ruleset = load_effective_rules(&cfg);
    let patterns = if cfg.masking.enabled {
        select_patterns(&cfg.masking.patterns)
    } else {
        Vec::new()
    };
    let mut events_out: Vec<String> = Vec::new();
    record_command_reads(cmd, &ruleset, mode, &args.cwd, &patterns, &mut events_out);

    // Watch roots: explicit --watch dirs, else the cwd.
    let roots = if args.watch.is_empty() {
        vec![args.cwd.clone()]
    } else {
        args.watch.clone()
    };

    let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })?;
    for root in &roots {
        watcher.watch(root, RecursiveMode::Recursive)?;
    }

    // Spawn the child in its own process group semantics (plain spawn is
    // enough; the shim never signals it beyond normal exit propagation).
    let mut child = std::process::Command::new(&cmd[0])
        .args(&cmd[1..])
        .spawn()
        .map_err(|e| anyhow::anyhow!("spawning {}: {e}", cmd[0]))?;

    loop {
        match child.try_wait()? {
            Some(status) => {
                while let Ok(res) = rx.try_recv() {
                    if let Ok(ev) = res {
                        record_event(
                            ev,
                            &ruleset,
                            mode,
                            &args.cwd,
                            args.events,
                            &patterns,
                            &mut events_out,
                        );
                    }
                }
                for line in events_out {
                    println!("{line}");
                }
                let code = status.code().unwrap_or(1);
                std::process::exit(code);
            }
            None => {
                while let Ok(res) = rx.try_recv() {
                    if let Ok(ev) = res {
                        record_event(
                            ev,
                            &ruleset,
                            mode,
                            &args.cwd,
                            args.events,
                            &patterns,
                            &mut events_out,
                        );
                    }
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

/// inotify does not expose ordinary file reads on Linux. Shell command
/// arguments are the best portable source for shim-level read masking:
/// matching existing files become mask records for the outer proxy to act on.
fn record_command_reads(
    cmd: &[String],
    ruleset: &vigil::rules::Ruleset,
    mode: Mode,
    cwd: &std::path::Path,
    patterns: &[vigil::mask::MaskPattern],
    out: &mut Vec<String>,
) {
    if patterns.is_empty() {
        return;
    }
    for argument in cmd {
        for token in argument.split([' ', '\t', '\n', '|', '>', '<', ';', '&', '"', '\'']) {
            if token.is_empty() || token == "/dev/null" {
                continue;
            }
            let path = PathBuf::from(token);
            let path = if path.is_absolute() {
                path
            } else {
                cwd.join(path)
            };
            if !path.is_file() {
                continue;
            }
            let event = vigil::event::Event {
                schema: 1,
                agent: "shim".to_string(),
                tool: "shim".to_string(),
                action: Action::Read,
                paths: vec![path.clone()],
                command: None,
                cwd: cwd.to_path_buf(),
                session: "shim".to_string(),
            };
            let verdict = ruleset.evaluate(&event, mode);
            if verdict.action != VerdictAction::Mask {
                continue;
            }
            let record = event_record(
                Action::Read,
                &[path],
                cwd,
                &verdict.rule,
                VerdictAction::Mask,
                &verdict.masked_paths,
            );
            let record = record.to_string();
            if !out.contains(&record) {
                out.push(record);
            }
        }
    }
}

/// Match an inotify event against the ruleset and emit its record line.
fn record_event(
    ev: notify::Event,
    ruleset: &vigil::rules::Ruleset,
    mode: Mode,
    cwd: &std::path::Path,
    emit: bool,
    _patterns: &[vigil::mask::MaskPattern],
    out: &mut Vec<String>,
) {
    let action = match ev.kind {
        notify::EventKind::Access(notify::event::AccessKind::Open(_))
        | notify::EventKind::Access(notify::event::AccessKind::Read) => Action::Read,
        notify::EventKind::Modify(_)
        | notify::EventKind::Create(_)
        | notify::EventKind::Remove(_) => Action::Write,
        _ => return,
    };
    let paths: Vec<PathBuf> = ev.paths.iter().filter(|p| p.is_file()).cloned().collect();
    if paths.is_empty() {
        return;
    }
    let event = vigil::event::Event {
        schema: 1,
        agent: "shim".to_string(),
        tool: "shim".to_string(),
        action,
        paths: paths.clone(),
        command: None,
        cwd: cwd.to_path_buf(),
        session: "shim".to_string(),
    };
    let verdict = ruleset.evaluate(&event, mode);
    let record = event_record(
        action,
        &paths,
        cwd,
        &verdict.rule,
        verdict.action,
        &verdict.masked_paths,
    );
    if emit {
        out.push(record.to_string());
    }
    if verdict.action == VerdictAction::Deny {
        eprintln!(
            "vigil shim: deny {} {} (rule {}; monitoring only — hooks enforce)",
            action,
            paths[0].display(),
            verdict.rule
        );
    } else if verdict.action == VerdictAction::Warn {
        eprintln!(
            "vigil shim: warn {} {} (rule {})",
            action,
            paths[0].display(),
            verdict.rule
        );
    }
}
