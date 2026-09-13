//! In-process daemon core and local IPC event fanout.

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use notify::{RecursiveMode, Watcher};
use tokio::io::AsyncWriteExt;
use tokio::sync::{broadcast, mpsc};

use crate::ipc::IpcRecord;
use crate::log::EventLog;

pub const SOCKET_PATH: &str = "/tmp/vigil.sock";

#[cfg(windows)]
pub const PIPE_PATH: &str = r"\\.\pipe\vigil-events";

#[derive(Debug)]
pub struct DaemonCore {
    pub log: EventLog,
    pub events: broadcast::Sender<IpcRecord>,
    inbox: mpsc::Receiver<IpcRecord>,
    sender: Arc<mpsc::Sender<IpcRecord>>,
}

impl DaemonCore {
    pub fn new(
        log_path: std::path::PathBuf,
        retention_days: u32,
        max_events: usize,
    ) -> (Self, tokio::sync::mpsc::Sender<IpcRecord>) {
        let (tx, rx) = tokio::sync::mpsc::channel::<IpcRecord>(1024);
        let (events, _) = broadcast::channel(1024);
        (
            Self {
                log: EventLog::new(log_path).with_retention(Some(retention_days), Some(max_events)),
                events,
                inbox: rx,
                sender: Arc::new(tx.clone()),
            },
            tx,
        )
    }

    pub async fn ingest(&self, event: &crate::event::Event, verdict: &crate::event::Verdict) {
        let record = IpcRecord::from_verdict(event, verdict);
        let _ = self.log.append(&record);
        let _ = self.events.send(record);
    }

    pub async fn accept_events(&self) -> Result<()> {
        #[cfg(unix)]
        let _ = tokio::fs::remove_file(SOCKET_PATH).await;
        #[cfg(unix)]
        let listener = tokio::net::UnixListener::bind(SOCKET_PATH)?;
        #[cfg(windows)]
        let listener = tokio::net::windows::named_pipe::ServerOptions::new()
            .first_pipe_instance(true)
            .create(PIPE_PATH)?;

        loop {
            #[cfg(unix)]
            let (mut stream, _) = listener.accept().await?;
            #[cfg(windows)]
            let mut stream = {
                let next_server = tokio::net::windows::named_pipe::ServerOptions::new()
                    .access_inbound(true)
                    .create(PIPE_PATH)?;
                listener.connect().await?;
                next_server
            };
            let sender = self.sender.clone();
            tokio::spawn(async move {
                let mut buffer = Vec::new();
                if tokio::io::AsyncReadExt::read_to_end(&mut stream, &mut buffer)
                    .await
                    .is_ok()
                {
                    for line in String::from_utf8_lossy(&buffer).lines() {
                        if let Ok(record) = serde_json::from_str::<IpcRecord>(line) {
                            let _ = sender.send(record).await.is_ok();
                        }
                    }
                }
                let _ = stream.shutdown().await;
            });
        }
    }

    pub fn watch_protected_paths(&self) -> Result<()> {
        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;
        for (source, dir) in crate::sync::rules_dirs() {
            if dir.is_dir() {
                let _ = watcher.watch(&dir, RecursiveMode::Recursive);
            }
            let _ = source;
        }
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                if let Ok(event) = event {
                    for path in event.paths.iter().filter(|p| p.is_file()) {
                        let record = IpcRecord {
                            schema: 1,
                            kind: "audit".to_string(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            agent: "audit".to_string(),
                            tool: "filesystem".to_string(),
                            action: crate::event::Action::Read,
                            paths: vec![path.clone()],
                            command: None,
                            cwd: path
                                .parent()
                                .map(Path::to_path_buf)
                                .unwrap_or_else(|| std::path::PathBuf::from(".")),
                            session: "audit".to_string(),
                            verdict: crate::event::VerdictAction::Warn,
                            rule: "audit-watcher".to_string(),
                            reason: "protected path activity observed".to_string(),
                            masked_paths: Vec::new(),
                        };
                        futures_block_on(sender.send(record));
                    }
                }
            }
        });
        Ok(())
    }

    pub async fn run(self) -> Result<()> {
        let Self {
            log,
            events,
            mut inbox,
            ..
        } = self;
        while let Some(record) = inbox.recv().await {
            let _ = log.append(&record);
            let _ = events.send(record);
        }
        Ok(())
    }
}

fn futures_block_on<T>(future: impl core::future::Future<Output = T>) -> Option<T> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok()
        .map(|runtime| runtime.block_on(future))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Action, Event, Verdict, VerdictAction};
    use std::path::PathBuf;

    #[tokio::test]
    async fn bus_broadcasts_record() {
        let event = Event {
            schema: 1,
            agent: "claude-code".to_string(),
            tool: "Read".to_string(),
            action: Action::Read,
            paths: vec![PathBuf::from("/repo/.env")],
            command: None,
            cwd: PathBuf::from("/repo"),
            session: "s1".to_string(),
        };
        let verdict = Verdict {
            action: VerdictAction::Deny,
            rule: "deny-env".to_string(),
            reason: "test".to_string(),
            masked_paths: Vec::new(),
        };
        let path = std::env::temp_dir().join(format!("vigil-daemon-{}.jsonl", std::process::id()));
        let (core, _tx) = DaemonCore::new(path, 30, 100_000);
        let mut rx = core.events.subscribe();
        core.ingest(&event, &verdict).await;
        let record = rx.recv().await.unwrap();
        assert_eq!(record.rule, "deny-env");
    }
}
