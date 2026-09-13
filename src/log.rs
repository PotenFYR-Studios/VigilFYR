//! Append-only IPC event log with bounded rotation.

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::ipc::IpcRecord;

const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024;
const ROTATIONS_TO_KEEP: u32 = 2;

#[derive(Debug)]
pub struct EventLog {
    path: PathBuf,
}

impl EventLog {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(&self, record: &IpcRecord) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create log directory {}", parent.display()))?;
        }
        if self.path.metadata().map(|m| m.len()).unwrap_or(0) >= MAX_LOG_BYTES {
            self.rotate()?;
        }

        let mut line = serde_json::to_string(record)?;
        line.push('\n');
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let mut writer = BufWriter::new(file);
        writer.write_all(line.as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    fn rotate(&self) -> Result<()> {
        for index in (1..=ROTATIONS_TO_KEEP).rev() {
            let source = self.rotated_path(index);
            if source.exists() {
                std::fs::rename(&source, self.rotated_path(index + 1))?;
            }
        }
        let oldest = self.rotated_path(ROTATIONS_TO_KEEP + 1);
        if oldest.exists() {
            std::fs::remove_file(&oldest)?;
        }
        if self.path.exists() {
            std::fs::rename(&self.path, self.rotated_path(1))?;
        }
        Ok(())
    }

    fn rotated_path(&self, index: u32) -> PathBuf {
        PathBuf::from(format!("{}.{}", self.path.display(), index))
    }

    pub fn tail(&self, count: usize) -> Result<Vec<IpcRecord>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let mut contents = String::new();
        File::open(&self.path)?.read_to_string(&mut contents)?;
        let records = contents
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect::<Vec<_>>();
        let start = records.len().saturating_sub(count);
        Ok(records[start..].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Action, VerdictAction};
    use std::path::PathBuf;

    fn record(sequence: u8) -> IpcRecord {
        IpcRecord {
            schema: 1,
            kind: "event".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            agent: "claude-code".to_string(),
            tool: "Read".to_string(),
            action: Action::Read,
            paths: vec![PathBuf::from(format!("/tmp/{sequence}.env"))],
            command: None,
            cwd: PathBuf::from("/tmp"),
            session: format!("s{sequence}"),
            verdict: VerdictAction::Deny,
            rule: "deny-env".to_string(),
            reason: "test".to_string(),
            masked_paths: Vec::new(),
        }
    }

    fn log_path(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("vigil-log-{}-{tag}", std::process::id()))
    }

    #[test]
    fn appends_and_tails_records() {
        let path = log_path("roundtrip");
        std::fs::remove_file(&path).ok();
        let log = EventLog::new(&path);
        log.append(&record(1)).unwrap();
        log.append(&record(2)).unwrap();
        let tail = log.tail(1).unwrap();
        assert_eq!(tail.len(), 1);
        assert_eq!(tail[0].session, "s2");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn rotation_creates_one_and_drops_three() {
        let path = log_path("rotation");
        for suffix in ["", ".1", ".2", ".3"] {
            std::fs::remove_file(format!("{}{suffix}", path.display())).ok();
        }
        std::fs::write(format!("{}.3", path.display()), b"old").unwrap();
        std::fs::write(format!("{}.2", path.display()), b"old").unwrap();
        std::fs::write(&path, vec![b'x'; MAX_LOG_BYTES as usize]).unwrap();
        let log = EventLog::new(&path);
        log.append(&record(9)).unwrap();

        assert!(PathBuf::from(format!("{}.1", path.display())).exists());
        assert!(!PathBuf::from(format!("{}.3", path.display())).exists());
        assert!(log.tail(1).unwrap().iter().any(|r| r.session == "s9"));
        for suffix in ["", ".1", ".2", ".3"] {
            std::fs::remove_file(format!("{}{suffix}", path.display())).ok();
        }
    }
}
