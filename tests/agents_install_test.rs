//! Integration tests for `vigil agents install|remove` (mutates HOME; serialized).

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

fn tmp_home(tag: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("vigil-agents-install-{tag}-{}", std::process::id()))
}

fn count_vigil(text: &str) -> usize {
    text.matches("vigil intercept").count()
}

#[test]
fn claude_hook_uses_native_pretooluse_shape() {
    let home = tmp_home("claude");
    std::fs::create_dir_all(home.join(".claude")).unwrap();
    std::fs::write(
        home.join(".claude/settings.json"),
        r#"{"model":"opus","hooks":{"PreToolUse":[{"matcher":"Bash","hooks":[{"type":"command","command":"other-hook"}]}],"Stop":[{"matcher":"*","hooks":[{"type":"command","command":"stop-hook"}]}]}}"#,
    )
    .unwrap();
    let _g = set_home(&home);

    let agents = vigil::agents::all_agents();
    let cc = agents.iter().find(|a| a.id == "claude-code").unwrap();
    let hook_path = &cc.hook_paths[0];

    (cc.install_hook)(hook_path, vigil::engine::Mode::Enforce).unwrap();

    let text = std::fs::read_to_string(hook_path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    // Native shape: hooks.PreToolUse[] with matcher + nested hooks[].command.
    let ptu = v["hooks"]["PreToolUse"]
        .as_array()
        .expect("PreToolUse array");
    assert_eq!(ptu.len(), 2, "existing hook kept, vigil added: {text}");
    let vigil_entries: Vec<&serde_json::Value> = ptu
        .iter()
        .filter(|e| {
            e["hooks"].as_array().is_some_and(|hs| {
                hs.iter().any(|h| {
                    h["command"]
                        .as_str()
                        .is_some_and(|c| c.starts_with("vigil "))
                })
            })
        })
        .collect();
    assert_eq!(vigil_entries.len(), 1, "exactly one vigil entry: {text}");
    assert_eq!(vigil_entries[0]["matcher"], "*");
    assert_eq!(
        vigil_entries[0]["hooks"][0]["command"],
        "vigil intercept --agent claude-code"
    );
    assert_eq!(vigil_entries[0]["hooks"][0]["type"], "command");
    // Unrelated content fully preserved.
    assert_eq!(v["model"], "opus");
    assert!(text.contains("other-hook") && text.contains("stop-hook"));

    // Re-run: update in place, never duplicate.
    (cc.install_hook)(hook_path, vigil::engine::Mode::Enforce).unwrap();
    let text2 = std::fs::read_to_string(hook_path).unwrap();
    assert_eq!(count_vigil(&text2), 1, "idempotent re-install:\n{text2}");
    let v2: serde_json::Value = serde_json::from_str(&text2).unwrap();
    assert_eq!(v2["hooks"]["PreToolUse"].as_array().unwrap().len(), 2);

    // Remove: only vigil entries go; everything else stays.
    (cc.remove_hook)(hook_path).unwrap();
    let text3 = std::fs::read_to_string(hook_path).unwrap();
    assert!(
        !text3.contains("vigil intercept"),
        "vigil removed:\n{text3}"
    );
    assert!(text3.contains("other-hook") && text3.contains("stop-hook"));
    assert!(text3.contains("model"));
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn claude_remove_drops_empty_hooks_key() {
    let home = tmp_home("claude-empty");
    std::fs::create_dir_all(home.join(".claude")).unwrap();
    std::fs::write(home.join(".claude/settings.json"), r#"{"model":"opus"}"#).unwrap();
    let _g = set_home(&home);

    let agents = vigil::agents::all_agents();
    let cc = agents.iter().find(|a| a.id == "claude-code").unwrap();
    let hook_path = &cc.hook_paths[0];
    (cc.install_hook)(hook_path, vigil::engine::Mode::Enforce).unwrap();
    (cc.remove_hook)(hook_path).unwrap();

    let text = std::fs::read_to_string(hook_path).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(v.get("hooks").is_none(), "empty hooks key cleaned:\n{text}");
    assert_eq!(v["model"], "opus");
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn cursor_hooks_json_install_idempotent() {
    let home = tmp_home("cursor");
    std::fs::create_dir_all(home.join(".cursor")).unwrap();
    let _g = set_home(&home);

    let agents = vigil::agents::all_agents();
    let cursor = agents.iter().find(|a| a.id == "cursor").unwrap();
    let hook_path = &cursor.hook_paths[0];
    assert!(
        hook_path.ends_with("hooks.json"),
        "cursor uses .cursor/hooks.json, got {}",
        hook_path.display()
    );

    (cursor.install_hook)(hook_path, vigil::engine::Mode::Enforce).unwrap();
    let text = std::fs::read_to_string(home.join(".cursor/hooks.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert!(v["hooks"]["beforeReadFile"].as_array().is_some(), "{text}");
    assert!(count_vigil(&text) >= 1);

    (cursor.install_hook)(hook_path, vigil::engine::Mode::Enforce).unwrap();
    let text2 = std::fs::read_to_string(home.join(".cursor/hooks.json")).unwrap();
    assert_eq!(
        count_vigil(&text2),
        count_vigil(&text),
        "idempotent:\n{text2}"
    );

    (cursor.remove_hook)(hook_path).unwrap();
    let text3 = std::fs::read_to_string(home.join(".cursor/hooks.json")).unwrap();
    assert!(!text3.contains("vigil intercept"), "cleaned:\n{text3}");
    std::fs::remove_dir_all(&home).ok();
}
