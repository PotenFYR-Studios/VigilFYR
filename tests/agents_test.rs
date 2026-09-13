//! Integration tests for agent detection (mutates HOME; serialized).

use std::sync::{Mutex, MutexGuard};

static HOME_LOCK: Mutex<()> = Mutex::new(());

struct HomeGuard {
    old: Option<std::ffi::OsString>,
    _guard: MutexGuard<'static, ()>,
}

impl Drop for HomeGuard {
    fn drop(&mut self) {
        match &self.old {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }
    }
}

fn set_home(home: &std::path::Path) -> HomeGuard {
    let guard = HOME_LOCK.lock().unwrap();
    let old = std::env::var_os("HOME");
    std::env::set_var("HOME", home);
    HomeGuard { old, _guard: guard }
}

#[test]
fn detects_claude_code_from_settings_json() {
    let home = std::env::temp_dir().join(format!("vigil-agents-claude-{}", std::process::id()));
    std::fs::create_dir_all(home.join(".claude")).unwrap();
    std::fs::write(home.join(".claude/settings.json"), "{}").unwrap();
    let _g = set_home(&home);

    let agents = vigil::agents::detect_agents();
    let cc = agents
        .iter()
        .find(|a| a.id == "claude-code")
        .expect("claude-code detected");
    assert_eq!(cc.display, "Claude Code");
    assert!(cc.supports_hooks);
    assert!(!cc.hook_paths.is_empty());
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn empty_home_detects_nothing() {
    let home = std::env::temp_dir().join(format!("vigil-agents-empty-{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    let _g = set_home(&home);

    let agents = vigil::agents::detect_agents();
    assert!(agents.is_empty(), "expected none, got: {agents:?}");
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn detects_codex_gemini_cursor_opencode_dirs() {
    let home = std::env::temp_dir().join(format!("vigil-agents-multi-{}", std::process::id()));
    for dir in [".codex", ".gemini", ".cursor", ".opencode"] {
        std::fs::create_dir_all(home.join(dir)).unwrap();
    }
    std::fs::write(home.join(".codex/config.toml"), "").unwrap();
    let _g = set_home(&home);

    let agents = vigil::agents::detect_agents();
    for id in ["codex", "gemini", "cursor", "opencode"] {
        assert!(
            agents.iter().any(|a| a.id == id),
            "missing {id} in {agents:?}"
        );
    }
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn claude_hook_install_is_idempotent() {
    let home = std::env::temp_dir().join(format!("vigil-agents-hook-{}", std::process::id()));
    std::fs::create_dir_all(home.join(".claude")).unwrap();
    std::fs::write(home.join(".claude/settings.json"), r#"{"model":"x"}"#).unwrap();
    let _g = set_home(&home);

    let agents = vigil::agents::detect_agents();
    let cc = agents.iter().find(|a| a.id == "claude-code").unwrap();
    let hook_path = &cc.hook_paths[0];

    (cc.install_hook)(hook_path, vigil::engine::Mode::Enforce).unwrap();
    let once = std::fs::read_to_string(&hook_path).unwrap();
    (cc.install_hook)(hook_path, vigil::engine::Mode::Enforce).unwrap();
    let twice = std::fs::read_to_string(&hook_path).unwrap();
    assert_eq!(once, twice, "second install must not duplicate");
    assert!(once.contains("vigil intercept"), "hook command present");
    assert!(once.contains("model"), "existing settings preserved");

    (cc.remove_hook)(hook_path).unwrap();
    let removed = std::fs::read_to_string(&hook_path).unwrap();
    assert!(!removed.contains("vigil intercept"), "hook removed");
    assert!(removed.contains("model"), "other settings survive removal");
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn non_hookable_agents_report_no_hooks() {
    // generic is the audit-only fallback; its install/remove are no-ops.
    let agents = vigil::agents::all_agents();
    let generic = agents.iter().find(|a| a.id == "generic").unwrap();
    assert!(!generic.supports_hooks);
    let p = std::path::Path::new("/nonexistent/vigil");
    (generic.install_hook)(p, vigil::engine::Mode::Audit).unwrap();
    (generic.remove_hook)(p).unwrap();
}
