//! `vigil uninstall` core: detect current installation state, then tear it
//! down completely (hooks, autostart, state dir, optional binary removal),
//! optionally keeping `~/.vigil` so a later `vigil setup` can reuse it.

use anyhow::Result;

/// What the user chose to do with `~/.vigil`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigPolicy {
    /// Keep config/rules/events for reuse after reinstall.
    Keep,
    /// Wipe the entire state directory.
    Wipe,
}

/// Options for [`uninstall`].
#[derive(Debug, Clone)]
pub struct UninstallOptions {
    pub config: ConfigPolicy,
    /// Remove every agent's vigil hooks (default: true).
    pub remove_hooks: bool,
    /// Remove the autostart entry (default: true).
    pub remove_autostart: bool,
    /// Also try to delete the running binary itself (never done in tests).
    pub remove_binary: bool,
}

impl Default for UninstallOptions {
    fn default() -> Self {
        Self {
            config: ConfigPolicy::Wipe,
            remove_hooks: true,
            remove_autostart: true,
            remove_binary: false,
        }
    }
}

/// Human-readable summary of one detection row.
pub struct DetectRow {
    pub label: String,
    pub detail: String,
    pub present: bool,
}

/// Detect every artifact Vigil may have created on this machine.
pub fn detect_state() -> Vec<DetectRow> {
    let home = crate::agents::home();
    let state = home.join(".vigil");

    let mut rows = Vec::new();

    let config = state.join("config.toml");
    rows.push(DetectRow {
        label: "config".into(),
        detail: config.display().to_string(),
        present: config.is_file(),
    });

    let rules = state.join("rules");
    let rule_count = count_files(&rules);
    rows.push(DetectRow {
        label: "user rules".into(),
        detail: format!("{} file(s) in {}", rule_count, rules.display()),
        present: rule_count > 0,
    });

    let ext = state.join("extensions");
    let ext_count = count_files(&ext);
    rows.push(DetectRow {
        label: "extensions".into(),
        detail: format!("{} file(s) in {}", ext_count, ext.display()),
        present: ext_count > 0,
    });

    let events = state.join("events.jsonl");
    rows.push(DetectRow {
        label: "event log".into(),
        detail: size_or_missing(&events),
        present: events.exists(),
    });

    let mut hooked = Vec::new();
    for agent in crate::agents::all_agents() {
        for path in &agent.hook_paths {
            if path.exists()
                && std::fs::read_to_string(path)
                    .is_ok_and(|t| t.contains("vigil intercept") || t.contains("vigil "))
            {
                hooked.push(agent.id.to_string());
                break;
            }
        }
    }
    rows.push(DetectRow {
        label: "agent hooks".into(),
        detail: if hooked.is_empty() {
            "none installed".into()
        } else {
            hooked.join(", ")
        },
        present: !hooked.is_empty(),
    });

    rows.push(DetectRow {
        label: "autostart".into(),
        detail: crate::tray::autostart_path().display().to_string(),
        present: crate::tray::autostart_path().is_file(),
    });

    let sock = state.join("vigil.sock");
    rows.push(DetectRow {
        label: "daemon socket".into(),
        detail: sock.display().to_string(),
        present: sock.exists(),
    });

    rows
}

/// Tear down everything Vigil installed. Returns the report lines that the
/// CLI should print (kept side-effect-free for testability).
pub fn uninstall(opts: &UninstallOptions) -> Result<Vec<String>> {
    let mut report = Vec::new();
    let home = crate::agents::home();
    let state = home.join(".vigil");

    if sock_alive(&state) {
        report.push(
            "warning: daemon socket found; stop `vigil daemon` before uninstalling if it is running"
                .into(),
        );
    }

    if opts.remove_hooks {
        for agent in crate::agents::all_agents() {
            if !agent.supports_hooks {
                continue;
            }
            for path in &agent.hook_paths {
                if !path.exists() {
                    continue;
                }
                match (agent.remove_hook)(path) {
                    Ok(()) => report.push(format!(
                        "removed {} hooks from {}",
                        agent.id,
                        path.display()
                    )),
                    Err(e) => report.push(format!(
                        "could not remove {} hooks from {}: {e}",
                        agent.id,
                        path.display()
                    )),
                }
            }
        }
    }

    if opts.remove_autostart && crate::tray::autostart_path().is_file() {
        match crate::tray::set_autostart(false) {
            Ok(()) => report.push("removed autostart entry".into()),
            Err(e) => report.push(format!("could not remove autostart entry: {e}")),
        }
    }

    match opts.config {
        ConfigPolicy::Keep => {
            report.push(format!(
                "kept {} (config, rules, events reusable by a future setup)",
                state.display()
            ));
        }
        ConfigPolicy::Wipe => {
            if state.exists() {
                match std::fs::remove_dir_all(&state) {
                    Ok(()) => report.push(format!("removed {}", state.display())),
                    Err(e) => report.push(format!("could not remove {}: {e}", state.display())),
                }
            }
        }
    }

    if opts.remove_binary {
        if let Ok(exe) = std::env::current_exe() {
            let old = exe.with_extension("vigil-uninstalled.old");
            match std::fs::rename(&exe, &old) {
                Ok(()) => {
                    match std::fs::remove_file(&old) {
                        Ok(()) => report.push(format!("removed binary {}", exe.display())),
                        // Windows cannot delete a running exe; the .old file
                        // is removable manually after this process exits.
                        Err(_) => report.push(format!(
                            "binary moved to {} (delete it after this process exits)",
                            old.display()
                        )),
                    }
                }
                Err(e) => report.push(format!("could not remove binary {}: {e}", exe.display())),
            }
        }
    }

    Ok(report)
}

fn sock_alive(state: &std::path::Path) -> bool {
    state.join("vigil.sock").exists() || state.join("vigil.token").exists()
}

fn count_files(dir: &std::path::Path) -> usize {
    if !dir.is_dir() {
        return 0;
    }
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                .count()
        })
        .unwrap_or(0)
}

fn size_or_missing(path: &std::path::Path) -> String {
    match std::fs::metadata(path) {
        Ok(m) => format!("{} bytes ({})", m.len(), path.display()),
        Err(_) => format!("missing ({})", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_home(tag: &str) -> std::path::PathBuf {
        let home =
            std::env::temp_dir().join(format!("vigil-uninstall-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).unwrap();
        home
    }

    fn set_home(home: &std::path::Path) {
        std::env::set_var("HOME", home);
        std::env::set_var("USERPROFILE", home);
    }

    fn restore_home(old: Option<std::ffi::OsString>) {
        match old {
            Some(v) => {
                std::env::set_var("HOME", v);
            }
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn detect_reports_config_only_when_present() {
        let _guard = crate::env_lock();
        let old = std::env::var_os("HOME");
        let home = temp_home("detect");
        set_home(&home);

        let none = detect_state();
        assert!(!none.iter().any(|r| r.label == "config" && r.present));

        let state = home.join(".vigil");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::write(state.join("config.toml"), "[general]\nenabled = true\n").unwrap();

        let some = detect_state();
        let config = some.iter().find(|r| r.label == "config").unwrap();
        assert!(config.present);
        assert!(config.detail.contains("config.toml"));

        restore_home(old);
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn wipe_removes_state_dir_and_hooks() {
        let _guard = crate::env_lock();
        let old = std::env::var_os("HOME");
        let home = temp_home("wipe");
        set_home(&home);

        let state = home.join(".vigil");
        std::fs::create_dir_all(state.join("rules")).unwrap();
        std::fs::write(state.join("config.toml"), "[general]\nenabled = true\n").unwrap();

        let opts = UninstallOptions::default();
        let report = uninstall(&opts).unwrap();
        assert!(
            !state.exists(),
            "state dir must be gone, report: {report:?}"
        );
        assert!(report.iter().any(|l| l.contains("removed")));

        restore_home(old);
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn keep_preserves_state_but_removes_autostart() {
        let _guard = crate::env_lock();
        let old = std::env::var_os("HOME");
        let home = temp_home("keep");
        set_home(&home);

        let state = home.join(".vigil");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::write(state.join("config.toml"), "[general]\nenabled = true\n").unwrap();
        crate::tray::set_autostart(true).unwrap();
        assert!(crate::tray::autostart_path().is_file());

        let opts = UninstallOptions {
            config: ConfigPolicy::Keep,
            ..UninstallOptions::default()
        };
        let report = uninstall(&opts).unwrap();
        assert!(state.exists(), "state dir must survive: {report:?}");
        assert!(state.join("config.toml").is_file());
        assert!(!crate::tray::autostart_path().exists());
        assert!(report.iter().any(|l| l.contains("kept")));

        restore_home(old);
        let _ = std::fs::remove_dir_all(&home);
    }
}
