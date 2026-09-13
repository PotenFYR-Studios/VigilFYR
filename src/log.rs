//! Rotating IPC event log with bounded size, age, and record retention.

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::ipc::IpcRecord;

pub const DEFAULT_MAX_BYTES: u64 = 10 * 1024 * 1024;
pub const DEFAULT_ROTATIONS: u32 = 2;

#[derive(Debug)]
pub struct EventLog {
    path: PathBuf,
    max_bytes: u64,
    rotations: u32,
    max_age_days: Option<u32>,
    max_records: Option<usize>,
}

impl EventLog {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            max_bytes: DEFAULT_MAX_BYTES,
            rotations: DEFAULT_ROTATIONS,
            max_age_days: None,
            max_records: None,
        }
    }

    pub fn with_retention(mut self, max_age_days: Option<u32>, max_records: Option<usize>) -> Self {
        self.max_age_days = max_age_days.filter(|days| *days > 0);
        self.max_records = max_records.filter(|count| *count > 0);
        self
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(&self, record: &IpcRecord) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create log directory {}", parent.display()))?;
        }
        if self.path.metadata().map(|m| m.len()).unwrap_or(0) >= self.max_bytes {
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
        drop(writer);
        self.apply_retention()?;
        Ok(())
    }

    #[cfg(test)]
    fn raw_len(&self) -> usize {
        std::fs::read_to_string(&self.path)
            .map(|contents| contents.lines().count())
            .unwrap_or(0)
    }

    fn rotate(&self) -> Result<()> {
        for index in (1..=self.rotations).rev() {
            let source = self.rotated_path(index);
            if source.exists() {
                std::fs::rename(&source, self.rotated_path(index + 1))?;
            }
        }
        let oldest = self.rotated_path(self.rotations + 1);
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
        self.records().map(|mut records| {
            let start = records.len().saturating_sub(count);
            records.drain(..start);
            records
        })
    }

    pub fn records(&self) -> Result<Vec<IpcRecord>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        Ok(self.read_and_retain()?.0.into_iter().flatten().collect())
    }

    fn read_and_retain(&self) -> Result<(Vec<Option<IpcRecord>>, bool)> {
        let mut contents = String::new();
        File::open(&self.path)?.read_to_string(&mut contents)?;
        let cutoff = self.max_age_days.map(cutoff_timestamp);
        let mut parsed = contents
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str::<IpcRecord>(line).ok())
            .map(|record| {
                let expired = cutoff
                    .as_ref()
                    .map(|cutoff| {
                        let record_time =
                            chrono::DateTime::parse_from_rfc3339(&record.timestamp).ok();
                        let cutoff_time = chrono::DateTime::parse_from_rfc3339(cutoff).ok();
                        match (record_time, cutoff_time) {
                            (Some(record), Some(cutoff)) => record < cutoff,
                            _ => record.timestamp.as_str() < cutoff.as_str(),
                        }
                    })
                    .unwrap_or(false);
                if expired {
                    None
                } else {
                    Some(record)
                }
            })
            .collect::<Vec<_>>();
        let retained = parsed.iter().filter(|record| record.is_some()).count();
        if let Some(max_records) = self.max_records {
            let excess = retained.saturating_sub(max_records);
            if excess > 0 {
                let mut skipped = 0;
                parsed.iter_mut().for_each(|record| {
                    if record.is_some() && skipped < excess {
                        *record = None;
                        skipped += 1;
                    }
                });
            }
        }
        let changed = self
            .max_records
            .map(|limit| retained > limit)
            .unwrap_or(false)
            || parsed.iter().any(Option::is_none);
        Ok((parsed, changed))
    }

    fn apply_retention(&self) -> Result<()> {
        if !self.path.exists() || (self.max_age_days.is_none() && self.max_records.is_none()) {
            return Ok(());
        }
        let (records, changed) = self.read_and_retain()?;
        if !changed {
            return Ok(());
        }
        let mut output = String::new();
        for record in records.into_iter().flatten() {
            output.push_str(&serde_json::to_string(&record)?);
            output.push('\n');
        }
        std::fs::write(&self.path, output)?;
        Ok(())
    }
}

fn cutoff_timestamp(days: u32) -> String {
    (chrono::Utc::now() - chrono::Duration::days(i64::from(days)))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
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
            timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
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
        let path = std::env::temp_dir().join(format!("vigil-log-{}-{tag}", std::process::id()));
        path
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
        std::fs::write(&path, vec![b'x'; DEFAULT_MAX_BYTES as usize]).unwrap();
        let log = EventLog::new(&path);
        log.append(&record(9)).unwrap();

        assert!(PathBuf::from(format!("{}.1", path.display())).exists());
        assert!(!PathBuf::from(format!("{}.3", path.display())).exists());
        assert!(log.tail(1).unwrap().iter().any(|r| r.session == "s9"));
        for suffix in ["", ".1", ".2", ".3"] {
            std::fs::remove_file(format!("{}{suffix}", path.display())).ok();
        }
    }

    #[test]
    fn retention_drops_expired_and_enforces_record_limit() {
        let path = log_path("retention");
        std::fs::remove_file(&path).ok();
        let log = EventLog::new(&path).with_retention(Some(30), Some(2));
        let mut old = record(1);
        old.timestamp = (chrono::Utc::now() - chrono::Duration::days(31))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        log.append(&old).unwrap();
        assert!(path.exists(), "first append must create the log");
        assert_eq!(log.raw_len(), 0, "expired record must be rewritten away");
        log.append(&record(2)).unwrap();
        assert!(!log.records().unwrap().is_empty(), "valid record retained");
        log.append(&record(3)).unwrap();
        assert!(log.records().unwrap().len() <= 2);

        let records = log.records().unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].session, "s2");
        assert_eq!(records[1].session, "s3");
        std::fs::remove_file(&path).ok();
    }
}
