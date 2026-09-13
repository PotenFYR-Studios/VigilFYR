//! Autostart and tray capability detection.

use std::path::PathBuf;

use anyhow::Result;

pub fn autostart_path() -> PathBuf {
    crate::agents::home().join(".config/autostart/vigil-daemon.desktop")
}

pub fn set_autostart(enabled: bool) -> Result<()> {
    let path = autostart_path();
    if enabled {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(
            &path,
            "[Desktop Entry]\nType=Application\nName=VigilFYR\nExec=vigil daemon\nX-GNOME-Autostart-enabled=true\n",
        )?;
    } else if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

pub fn tray_supported() -> bool {
    std::env::var_os("DISPLAY").is_some()
        || std::env::var_os("WAYLAND_DISPLAY").is_some()
        || std::env::var_os("SYSTEM_TRAY").is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_removes_linux_autostart_entry() {
        let _guard = crate::env_lock();
        let old = std::env::var_os("HOME");
        let home = std::env::temp_dir().join(format!("vigil-tray-{}", std::process::id()));
        std::env::set_var("HOME", &home);
        set_autostart(true).unwrap();
        assert!(autostart_path().is_file());
        set_autostart(false).unwrap();
        assert!(!autostart_path().exists());
        match old {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
    }
}
