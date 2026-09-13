//! Integration test for `vigil shim` (spawns the built binary).

use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn shim_records_fs_event_and_propagates_exit_code() {
    let tmp = std::env::temp_dir().join(format!("vigil-shim-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let target = tmp.join("x.pem");

    let mut child = Command::new(env!("CARGO_BIN_EXE_vigil"))
        .args([
            "shim",
            "--cwd",
            tmp.to_str().unwrap(),
            "--",
            "sh",
            "-c",
            &format!("touch {}; echo done; exit 7", target.display()),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn vigil shim");
    child.stdin.take().unwrap().write_all(b"").ok();
    let out = child.wait_with_output().expect("wait for shim");

    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();

    // Child's exit code propagates.
    assert_eq!(out.status.code(), Some(7), "stderr: {stderr}");

    // An event record (JSON line) for the observed fs event must land on
    // stdout. The child touched a .pem inside the watched cwd.
    let event_line = stdout.lines().find(|l| {
        l.contains("x.pem")
            && l.contains("\"event\"")
            && serde_json::from_str::<serde_json::Value>(l).is_ok()
    });
    let line = event_line.unwrap_or_else(|| {
        panic!(
            "no event record for x.pem on stdout\ncwd: {}\nstdout: {stdout}\nstderr: {stderr}",
            tmp.display()
        )
    });
    let v: serde_json::Value = serde_json::from_str(line).unwrap();
    assert_eq!(v["schema"], 1);
    assert!(v["paths"].as_array().is_some_and(|p| !p.is_empty()));

    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn shim_help_is_honest_about_monitoring() {
    let out = Command::new(env!("CARGO_BIN_EXE_vigil"))
        .args(["shim", "--help"])
        .output()
        .unwrap();
    let text =
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
    assert!(
        text.contains("monitoring"),
        "--help must say monitoring wrapper: {text}"
    );
}
