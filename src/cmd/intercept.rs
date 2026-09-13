//! `vigil intercept` - the hot path agents call through their hooks.
//! Event JSON on stdin, verdict JSON on stdout, human note on stderr.

use std::sync::OnceLock;
use tokio::io::AsyncReadExt;

use anyhow::Result;
use clap::Parser;

use vigil::config::Config;
use vigil::engine::Mode;
use vigil::event::{Event, Verdict, VerdictAction};
use vigil::ext::{apply_event_hooks, load_extensions};
use vigil::ipc::IpcRecord;
use vigil::log::EventLog;
use vigil::sync::load_effective_rules;

#[derive(Parser)]
pub struct InterceptArgs {
    /// Override the event's agent id (e.g. when the hook knows better).
    #[arg(long)]
    agent: Option<String>,
    /// Fail closed: unparseable input denies instead of allowing.
    #[arg(long)]
    strict: bool,
    /// Protocol supports post-read content rewrite (shim/proxy paths).
    /// Hooks path leaves this off; Mask degrades to warn there.
    #[arg(long)]
    pub supports_rewrite: bool,
}

/// Memoized effective ruleset + mode; built once per process.
struct Effective {
    ruleset: vigil::rules::Ruleset,
    mode: Mode,
    enabled: bool,
    masking_enabled: bool,
}

static EFFECTIVE: OnceLock<Effective> = OnceLock::new();

fn effective() -> &'static Effective {
    EFFECTIVE.get_or_init(|| {
        let cfg = Config::load();
        Effective {
            ruleset: load_effective_rules(&cfg),
            mode: cfg.general.mode,
            enabled: cfg.general.enabled,
            masking_enabled: cfg.masking.enabled,
        }
    })
}

/// Post-process a Mask verdict for the calling protocol. Hooks can't
/// rewrite tool output; shim/proxy can. Returns the verdict to emit and
/// its exit code.
fn adapt_mask_verdict(v: &mut Verdict, supports_rewrite: bool, masking_enabled: bool) -> i32 {
    if v.action != VerdictAction::Mask {
        return match v.action {
            VerdictAction::Deny => 2,
            _ => 0,
        };
    }
    if !masking_enabled {
        v.action = VerdictAction::Warn;
        v.reason = format!("masking disabled by config; {reason}", reason = v.reason);
        return 0;
    }
    if supports_rewrite {
        // Allowed, content will be rewritten; paths tell the caller which
        // tool results to pass through the masking layer.
        0
    } else {
        v.action = VerdictAction::Warn;
        v.reason = format!(
            "masking unsupported by this agent; enable shim or proxy mode ({reason})",
            reason = v.reason
        );
        0
    }
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
pub async fn run(args: InterceptArgs, mut stdin: impl Unpin + tokio::io::AsyncRead) -> Result<i32> {
    let mut event_json = Vec::new();
    stdin.read_to_end(&mut event_json).await?;
    let parsed = serde_json::from_slice::<Event>(&event_json);
    let (verdict, code) = match parsed {
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
                let mut v = eff.ruleset.evaluate(&event, eff.mode);
                if v.action == VerdictAction::Mask {
                    v.masked_paths = event.paths.clone();
                }
                let extensions = load_extensions(vigil::ext::extensions_root());
                if !extensions.is_empty() {
                    let record = IpcRecord::from_verdict(&event, &v);
                    if let Some(override_json) = apply_event_hooks(&extensions, &record.to_line()?)?
                    {
                        #[derive(serde::Deserialize)]
                        struct HookOverride {
                            action: VerdictAction,
                            #[serde(default)]
                            rule: Option<String>,
                            #[serde(default)]
                            reason: Option<String>,
                        }
                        if let Ok(over) = serde_json::from_str::<HookOverride>(&override_json) {
                            v.action = over.action;
                            if let Some(rule) = over.rule {
                                v.rule = rule;
                            }
                            if let Some(reason) = over.reason {
                                v.reason = reason;
                            } else {
                                v.reason = "extension hook override".to_string();
                            }
                        }
                    }
                }
                let code = adapt_mask_verdict(&mut v, args.supports_rewrite, eff.masking_enabled);
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
    if let Ok(event) = serde_json::from_slice::<Event>(&event_json) {
        let record = IpcRecord::from_verdict(&event, &verdict);
        let log = EventLog::new(
            dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".vigil/events.jsonl"),
        );
        let _ = log.append(&record);
        publish(&record).await;
    }
    Ok(code)
}

async fn publish(record: &IpcRecord) {
    #[cfg(unix)]
    if let Ok(mut stream) = tokio::net::UnixStream::connect("/tmp/vigil.sock").await {
        use tokio::io::AsyncWriteExt;
        let _ = stream
            .write_all(record.to_line().unwrap_or_default().as_bytes())
            .await;
    }
    #[cfg(windows)]
    if let Ok(mut stream) =
        tokio::net::windows::named_pipe::ClientOptions::new().open(vigil::daemon::PIPE_PATH)
    {
        use tokio::io::AsyncWriteExt;
        let _ = stream
            .write_all(record.to_line().unwrap_or_default().as_bytes())
            .await;
    }
}

/// CLI entry: stdin → exit code.
pub fn intercept(args: InterceptArgs) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()?;
    let code = runtime.block_on(run(args, tokio::io::stdin()))?;
    std::process::exit(code);
}
