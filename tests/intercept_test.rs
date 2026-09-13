//! Integration tests for `vigil intercept` (spawns the built binary).

use std::io::Write;
use std::process::{Command, Stdio};

struct Out {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}

fn run_vigil(args: &[&str], stdin: &str) -> Out {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vigil"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn vigil");
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).ok();
    let output = child.wait_with_output().expect("wait for vigil");
    Out {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

#[test]
fn intercept_denies_env_read_via_stdin() {
    let out = run_vigil(
        &["intercept", "--agent", "claude-code"],
        r#"{"schema":1,"agent":"claude-code","tool":"Read","action":"read","paths":["/repo/.env"],"command":null,"cwd":"/repo","session":"s"}"#,
    );
    assert_eq!(out.status.code(), Some(2), "stderr: {}", out.stderr);
    assert!(
        out.stdout.contains("\"action\":\"deny\""),
        "stdout: {}",
        out.stdout
    );
}

#[test]
fn intercept_fails_open_on_garbage() {
    let out = run_vigil(&["intercept"], "not json");
    assert_eq!(out.status.code(), Some(0), "stderr: {}", out.stderr);
    assert!(out.stderr.contains("vigil"), "stderr: {}", out.stderr);
    assert!(
        out.stdout.contains("\"action\":\"allow\""),
        "stdout: {}",
        out.stdout
    );
}

#[test]
fn intercept_strict_fails_closed_on_garbage() {
    let out = run_vigil(&["intercept", "--strict"], "not json");
    assert_eq!(out.status.code(), Some(2), "stderr: {}", out.stderr);
    assert!(
        out.stdout.contains("\"action\":\"deny\""),
        "stdout: {}",
        out.stdout
    );
}

#[test]
fn intercept_allows_benign_read() {
    let out = run_vigil(
        &["intercept"],
        r#"{"schema":1,"agent":"claude-code","tool":"Read","action":"read","paths":["/repo/src/main.rs"],"command":null,"cwd":"/repo","session":"s"}"#,
    );
    assert_eq!(out.status.code(), Some(0), "stderr: {}", out.stderr);
    assert!(
        out.stdout.contains("\"action\":\"allow\""),
        "stdout: {}",
        out.stdout
    );
}

#[test]
fn intercept_audit_mode_demotes_deny_to_warn() {
    // Point HOME at a temp config with mode=audit; deny .env read becomes warn.
    let home = std::env::temp_dir().join(format!("vigil-intercept-audit-{}", std::process::id()));
    let cfg_dir = home.join(".vigil");
    std::fs::create_dir_all(&cfg_dir).unwrap();
    std::fs::write(
        cfg_dir.join("config.toml"),
        "[general]\nenabled = true\nmode = \"audit\"\n",
    )
    .unwrap();
    let out = {
        let mut child = Command::new(env!("CARGO_BIN_EXE_vigil"))
            .args(["intercept", "--agent", "claude-code"])
            .env("HOME", &home)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(
                br#"{"schema":1,"agent":"claude-code","tool":"Read","action":"read","paths":["/repo/.env"],"command":null,"cwd":"/repo","session":"s"}"#,
            )
            .ok();
        child.wait_with_output().unwrap()
    };
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("\"action\":\"warn\""),
        "stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn intercept_under_10ms_p95() {
    // In-process: 200 evaluates against the compiled ruleset, p95 < 10ms.
    use std::time::Instant;
    let mut vigil = vigil::sync::load_effective_rules(&vigil::config::Config::load());
    let event: vigil::event::Event = serde_json::from_str(
        r#"{"schema":1,"agent":"claude-code","tool":"Read","action":"read","paths":["/repo/.env"],"command":null,"cwd":"/repo","session":"s"}"#,
    )
    .unwrap();
    let mut samples: Vec<u128> = Vec::new();
    for _ in 0..200 {
        let start = Instant::now();
        let v = vigil.evaluate(&event, vigil::engine::Mode::Enforce);
        samples.push(start.elapsed().as_micros());
        assert_eq!(v.action, vigil::event::VerdictAction::Deny);
    }
    samples.sort();
    let p95 = samples[189];
    assert!(p95 < 10_000, "p95 {}µs exceeds 10ms budget", p95);
    // silence unused-mut if evaluate ever takes &self by value
    let _ = &mut vigil;
}
