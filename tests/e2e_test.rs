//! Cross-layer end-to-end coverage for setup, enforcement, reporting, and cleanup.

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
    let output = child.wait_with_output().expect("wait for vigil");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn setup_enforce_log_snapshot_and_cleanup() {
    let home = std::env::temp_dir().join(format!("vigil-e2e-{}", std::process::id()));
    std::fs::create_dir_all(home.join(".claude")).unwrap();
    std::fs::write(home.join(".claude/settings.json"), "{}").unwrap();

    let (setup_code, _, setup_err) = run_vigil(&["setup", "--defaults"], "", &home);
    assert_eq!(setup_code, 0, "setup failed: {setup_err}");
    assert!(home.join(".vigil/config.toml").is_file());

    let (code, stdout, stderr) = run_vigil(
        &["intercept", "--agent", "claude-code"],
        r#"{"schema":1,"agent":"claude-code","tool":"Read","action":"read","paths":["/repo/.env"],"command":null,"cwd":"/repo","session":"e2e"}"#,
        &home,
    );
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stdout.contains("\"action\":\"deny\""), "stdout: {stdout}");
    assert!(home.join(".vigil/events.jsonl").is_file());

    let (tui_code, snapshot, tui_err) = run_vigil(&["tui", "--once"], "", &home);
    assert_eq!(tui_code, 0, "snapshot failed: {tui_err}");
    assert!(snapshot.contains("deny-env-files"), "snapshot: {snapshot}");

    let (remove_code, _, remove_err) = run_vigil(&["agents", "remove", "all"], "", &home);
    assert_eq!(remove_code, 0, "cleanup failed: {remove_err}");
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn extension_rule_participates_and_hook_can_override() {
    let home = std::env::temp_dir().join(format!("vigil-e2e-ext-{}", std::process::id()));
    let ext = home.join(".vigil/extensions/security");
    std::fs::create_dir_all(ext.join("rules")).unwrap();
    std::fs::write(
        ext.join("manifest.toml"),
        "name = \"security\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(
        ext.join("rules/ext.toml"),
        "id = \"ext-deny\"\nscope = [\"read\"]\npaths = [\"**/ext-secret\"]\naction = \"deny\"\n",
    )
    .unwrap();

    let (code, stdout, stderr) = run_vigil(
        &["intercept"],
        r#"{"schema":1,"agent":"claude-code","tool":"Read","action":"read","paths":["/repo/ext-secret"],"command":null,"cwd":"/repo","session":"ext"}"#,
        &home,
    );
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stdout.contains("ext-deny"), "stdout: {stdout}");
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn extension_hook_deny_overrides_verdict() {
    let home = std::env::temp_dir().join(format!("vigil-e2e-hook-{}", std::process::id()));
    let ext = home.join(".vigil/extensions/hook");
    std::fs::create_dir_all(ext.join("hooks")).unwrap();
    std::fs::write(
        ext.join("manifest.toml"),
        "name = \"hook\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    let hook_path = ext.join("hooks/on-event.sh");
    std::fs::write(
        hook_path,
        "#!/bin/sh\nprintf '%s' '{\"action\":\"deny\",\"rule\":\"hook-deny\",\"reason\":\"hook override\"}'\nexit 2\n",
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(ext.join("hooks/on-event.sh"))
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(ext.join("hooks/on-event.sh"), perms).unwrap();
    }
    #[cfg(windows)]
    std::fs::write(
        ext.join("hooks/on-event.cmd"),
        "@echo off\r\nmore\r\nexit /b 2\r\n",
    )
    .unwrap();

    let (code, stdout, stderr) = run_vigil(
        &["intercept"],
        r#"{"schema":1,"agent":"claude-code","tool":"Read","action":"read","paths":["/repo/normal"],"command":null,"cwd":"/repo","session":"hook"}"#,
        &home,
    );
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stdout.contains("hook-deny"), "stdout: {stdout}");
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn masking_opt_in_flips_env_read_to_mask() {
    let home = std::env::temp_dir().join(format!("vigil-e2e-mask-{}", std::process::id()));
    std::fs::create_dir_all(home.join(".vigil/rules")).unwrap();
    std::fs::write(
        home.join(".vigil/config.toml"),
        "[masking]\nenabled = true\npatterns = [\"aws_key\"]\n",
    )
    .unwrap();
    std::fs::write(
        home.join(".vigil/rules/mask.toml"),
        "rule = [{ id = \"mask-env\", scope = [\"read\"], paths = [\"**/*.env\"], action = \"mask\", severity = \"medium\" }]\n",
    )
    .unwrap();

    let (code, stdout, stderr) = run_vigil(
        &["intercept", "--supports-rewrite"],
        r#"{"schema":1,"agent":"claude-code","tool":"Read","action":"read","paths":["/repo/app.env"],"command":null,"cwd":"/repo","session":"mask"}"#,
        &home,
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("\"action\":\"mask\""), "stdout: {stdout}");
    assert!(stdout.contains("app.env"), "stdout: {stdout}");
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn installer_dry_run_prints_plan_without_writes() {
    let (program, script) = if cfg!(windows) {
        ("powershell.exe", "-NoProfile")
    } else {
        ("sh", "install.sh")
    };
    let mut command = Command::new(program);
    if cfg!(windows) {
        command
            .arg(script)
            .arg("-File")
            .arg("install.ps1")
            .arg("-DryRun");
    } else {
        command.arg(script);
    }
    let output = command
        .arg("--dry-run")
        .env("HOME", std::env::temp_dir())
        .output()
        .expect("run installer");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("install plan:"));
}

#[test]
fn piped_interactive_defaults_write_config_and_hooks() {
    let home = std::env::temp_dir().join(format!("vigil-setup-{}", std::process::id()));
    std::fs::create_dir_all(home.join(".claude")).unwrap();
    std::fs::write(home.join(".claude/settings.json"), "{}").unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_vigil"))
        .args(["setup"])
        .env("HOME", &home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"\n\n\n").unwrap();
    let status = child.wait().unwrap();
    assert!(status.success());
    assert!(home.join(".vigil/config.toml").is_file());
    std::fs::remove_dir_all(home).ok();
}
