//! `vigil daemon` Unix socket server.

use anyhow::Result;
use clap::Parser;
use vigil::sync::sync_remote_rules;

#[derive(Parser)]
pub struct DaemonArgs {
    #[arg(long)]
    pub no_tray: bool,
}

pub async fn daemon(no_tray: bool) -> Result<()> {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let cfg = vigil::config::Config::load();
    let note = sync_remote_rules(&cfg)?;
    println!("vigil daemon: {note}");
    println!(
        "vigil daemon: detected {} agents",
        vigil::agents::detect_agents().len()
    );
    let (core, _sender) = vigil::daemon::DaemonCore::new(home.join(".vigil/events.jsonl"));
    if vigil::tray::tray_supported() && !no_tray {
        println!("vigil daemon: tray requested; headless daemon process remains active");
    } else {
        println!("vigil daemon: running headless");
    }
    println!("vigil daemon: listening on {}", vigil::daemon::SOCKET_PATH);
    core.watch_protected_paths()?;
    core.accept_events().await
}
