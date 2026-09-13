//! `vigil intercept` — the hot path agents call through their hooks.
//! Event JSON on stdin, verdict JSON on stdout, human note on stderr.

use std::io::Read;
use std::sync::OnceLock;

use anyhow::Result;
use clap::Parser;

use vigil::config::Config;
use vigil::engine::Mode;
use vigil::event::{Event, Verdict, VerdictAction};
use vigil::sync::load_effective_rules;

#[derive(Parser)]
pub struct InterceptArgs {
    /// Override the event's agent id (e.g. when the hook knows better).
    #[arg(long)]
    agent: Option<String>,
    /// Fail closed: unparseable input denies instead of allowing.
    #[arg(long)]
    strict: bool,
}

/// Memoized effective ruleset + mode; built once per process.
struct Effective {
    ruleset: vigil::rules::Ruleset,
    mode: Mode,
    enabled: bool,
}

static EFFECTIVE: OnceLock<Effective> = OnceLock::new();

fn effective() -> &'static Effective {
    EFFECTIVE.get_or_init(|| {
        let cfg = Config::load();
        Effective {
            ruleset: load_effective_rules(&cfg),
            mode: cfg.general.mode,
            enabled: cfg.general.enabled,
        }
    })
}

fn verdict_json(v: &Verdict) -> String {
    // Compact single-line JSON; serde_json is already a dependency.
    serde_json::to_string(v).expect("verdict serializes")
}

fn fail_verdict(strict: bool, reason: &str) -> (Verdict, i32) {
    if strict {
        (
            Verdict {
                action: VerdictAction::Deny,
                rule: "fail-closed".to_string(),
                reason: reason.to_string(),
                masked_paths: Vec::new(),
            },
            2,
        )
    } else {
        (
            Verdict {
                action: VerdictAction::Allow,
                rule: "fail-open".to_string(),
                reason: reason.to_string(),
                masked_paths: Vec::new(),
            },
            0,
        )
    }
}

/// Run the intercept path; returns the process exit code.
pub fn run(args: InterceptArgs, mut stdin: impl Read) -> Result<i32> {
    let (verdict, code) = match serde_json::from_reader::<_, Event>(&mut stdin) {
        Ok(mut event) => {
            if let Some(agent) = &args.agent {
                event.agent = agent.clone();
            }
            let eff = effective();
            if !eff.enabled {
                let v = Verdict {
                    action: VerdictAction::Allow,
                    rule: "disabled".to_string(),
                    reason: "vigil disabled by config".to_string(),
                    masked_paths: Vec::new(),
                };
                (v, 0)
            } else {
                let v = eff.ruleset.evaluate(&event, eff.mode);
                let code = match v.action {
                    VerdictAction::Allow | VerdictAction::Warn => 0,
                    VerdictAction::Deny => 2,
                    VerdictAction::Mask => 3,
                };
                (v, code)
            }
        }
        Err(e) => fail_verdict(
            args.strict,
            &format!("vigil: unparseable event ({e}); failing open"),
        ),
    };
    println!("{}", verdict_json(&verdict));
    eprintln!("vigil: {} ({})", verdict.action, verdict.reason);
    Ok(code)
}

/// CLI entry: stdin → exit code.
pub fn intercept(args: InterceptArgs) -> Result<()> {
    let code = run(args, std::io::stdin())?;
    std::process::exit(code);
}
