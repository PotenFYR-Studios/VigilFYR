//! Integration tests for masking wired into intercept and shim.

use std::io::Write;
use std::process::{Command, Stdio};

fn run_vigil(args: &[&str], stdin: &str, home: &std::path::Path) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vigil"))
        .args(args)
        .env("HOME", home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn vigil");
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).ok();
    let out = child.wait_with_output().expect("wait for vigil");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn write_config(home: &std::path::Path, extra: &str) {
    let dir = home.join(".vigil");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("config.toml"), extra).unwrap();
}

/// A mask rule scoped to reads of *.env files.
const MASK_RULE: &str = r#"
rule = [
  {
    id = "mask-env",
    scope = ["read"],
    paths = ["**/*.env"],
    action = "mask",
    severity = "medium"
  }
]
"#;

#[test]
fn mask_verdict_degrades_to_warn_on_hook_path() {
    // Hooks protocol can't rewrite tool output → warn, not mask.
    let home = std::env::temp_dir().join(format!("vigil-mask-hook-{}", std::process::id()));
    write_config(
        &home,
        "[masking]\nenabled = true\npatterns = [\"aws_key\"]\n",
    );
    let rules_dir = home.join(".vigil/rules");
    std::fs::create_dir_all(&rules_dir).unwrap();
    std::fs::write(rules_dir.join("mask.toml"), MASK_RULE).unwrap();

    let (code, stdout, stderr) = run_vigil(
        &["intercept", "--agent", "claude-code"],
        r#"{"schema":1,"agent":"claude-code","tool":"Read","action":"read","paths":["/repo/prod.env"],"command":null,"cwd":"/repo","session":"s"}"#,
        &home,
    );
    assert_eq!(code, 0, "degraded warn exits 0: {stderr}");
    assert!(stdout.contains("\"action\":\"warn\""), "stdout: {stdout}");
    assert!(
        stderr.contains("masking unsupported by this agent"),
        "reason surfaced: {stderr}"
    );
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn mask_verdict_allows_and_fills_masked_paths_via_shim() {
    // Shim protocol supports content rewrite → allow + masked_paths filled.
    let home = std::env::temp_dir().join(format!("vigil-mask-shim-{}", std::process::id()));
    write_config(
        &home,
        "[masking]\nenabled = true\npatterns = [\"aws_key\"]\n",
    );
    let rules_dir = home.join(".vigil/rules");
    std::fs::create_dir_all(&rules_dir).unwrap();
    std::fs::write(rules_dir.join("mask.toml"), MASK_RULE).unwrap();

    // Read the fake .env via the shim: the file's AWS key must never
    // appear raw in the shim's stdout event records.
    let tmp = std::env::temp_dir().join(format!("vigil-mask-run-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    std::fs::write(tmp.join("app.env"), "KEY=AKIAIOSFODNN7EXAMPLE\n").unwrap();

    let (code, stdout, stderr) = run_vigil(
        &[
            "shim",
            "--cwd",
            tmp.to_str().unwrap(),
            "--",
            "sh",
            "-c",
            "cat app.env > /dev/null",
        ],
        "",
        &home,
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("\"verdict\":\"mask\""),
        "mask verdict in event record: {stdout}"
    );
    assert!(
        stdout.contains("masked_paths"),
        "masked_paths filled: {stdout}"
    );
    assert!(
        !stdout.contains("AKIAIOSFODNN7EXAMPLE"),
        "secret must not leak into records: {stdout}"
    );
    std::fs::remove_dir_all(&tmp).ok();
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn masking_disabled_leaves_mask_rules_deny_like_passthrough() {
    // masking.enabled = false (default): a mask rule still fires as Mask
    // in the engine, but the pipeline treats it as unsupported → warn.
    let home = std::env::temp_dir().join(format!("vigil-mask-off-{}", std::process::id()));
    let rules_dir = home.join(".vigil/rules");
    std::fs::create_dir_all(&rules_dir).unwrap();
    std::fs::write(rules_dir.join("mask.toml"), MASK_RULE).unwrap();

    let (code, stdout, stderr) = run_vigil(
        &["intercept", "--agent", "claude-code"],
        r#"{"schema":1,"agent":"claude-code","tool":"Read","action":"read","paths":["/repo/prod.env"],"command":null,"cwd":"/repo","session":"s"}"#,
        &home,
    );
    assert_eq!(code, 0, "warn exit: {stderr}");
    assert!(stdout.contains("\"action\":\"warn\""), "stdout: {stdout}");
    assert!(stderr.contains("masking disabled"), "reason: {stderr}");
    std::fs::remove_dir_all(&home).ok();
}
