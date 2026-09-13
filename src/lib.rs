//! vigil library crate. Domain logic lands here in later tasks.

pub mod agents;
pub mod config;
pub mod daemon;
pub mod engine;
pub mod event;
pub mod ext;
pub mod ipc;
pub mod log;
pub mod mask;
pub mod rules;
pub mod sync;
pub mod tray;
pub mod tui;
pub mod uninstall;
pub mod update;

/// Test-only guard serializing tests that mutate process-global env
/// (HOME/USERPROFILE). Parallel tests swapping HOME race otherwise.
#[cfg(test)]
pub(crate) fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
