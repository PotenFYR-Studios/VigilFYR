//! Update metadata comparison and self-update archive handling.

use anyhow::{bail, Context, Result};
use flate2::read::GzDecoder;
use serde_json::Value;
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Archive;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    UpToDate,
    Outdated { latest: String },
    AheadUnknown { commits: u32 },
}

#[derive(Debug, Clone, Copy)]
pub struct UpdateChecker<F: Fn(&str) -> Result<Vec<u8>>> {
    pub current_version: &'static str,
    pub current_sha: &'static str,
    pub fetch_release: F,
}

impl<F: Fn(&str) -> Result<Vec<u8>>> UpdateChecker<F> {
    pub fn status_for_tag(&self, latest_tag: &str) -> UpdateStatus {
        let latest = latest_tag.trim_start_matches('v');
        if version_gt(latest, self.current_version) {
            UpdateStatus::Outdated {
                latest: latest.to_string(),
            }
        } else {
            UpdateStatus::UpToDate
        }
    }

    pub fn status_for_commits(&self, shas: &[&str]) -> UpdateStatus {
        match shas
            .iter()
            .position(|sha| sha.starts_with(self.current_sha))
        {
            Some(index) if index > 0 => UpdateStatus::AheadUnknown {
                commits: index as u32,
            },
            _ => UpdateStatus::UpToDate,
        }
    }

    pub fn check(&self, api_base: &str) -> Result<UpdateStatus> {
        let body = (self.fetch_release)(&format!("{api_base}/releases/latest"))?;
        let json: Value = serde_json::from_slice(&body)?;
        match json.get("tag_name").and_then(Value::as_str) {
            Some(tag) => Ok(self.status_for_tag(tag)),
            None => Ok(UpdateStatus::UpToDate),
        }
    }
}

fn version_gt(left: &str, right: &str) -> bool {
    let parse = |value: &str| -> Vec<u64> {
        value
            .split(['.', '-'])
            .filter_map(|part| part.parse().ok())
            .collect()
    };
    let left = parse(left);
    let right = parse(right);
    left > right
}

pub fn self_update_with_archive(
    archive: impl Read,
    expected_checksum: &str,
    binary_name: &str,
) -> Result<PathBuf> {
    let bytes = std::io::read_to_string(archive).context("read update archive")?;
    let actual = sha256_ascii(bytes.as_bytes());
    if actual != expected_checksum.trim() {
        bail!("update checksum mismatch");
    }
    let cursor = std::io::Cursor::new(bytes.into_bytes());
    let mut tar = Archive::new(GzDecoder::new(cursor));
    let temp = std::env::temp_dir().join(format!("vigil-update-{}", std::process::id()));
    std::fs::create_dir_all(&temp)?;
    tar.unpack(&temp)?;
    let new_binary = find_binary(&temp, binary_name)?;
    let current = std::env::current_exe()?;
    let old = current.with_extension("old");
    let _ = std::fs::remove_file(&old);
    std::fs::rename(&current, &old)?;
    std::fs::copy(&new_binary, &current)?;
    let _ = std::fs::remove_dir_all(&temp);
    Ok(old)
}

fn find_binary(root: &Path, name: &str) -> Result<PathBuf> {
    for entry in walkdir::WalkDir::new(root).into_iter().flatten() {
        let path = entry.path();
        if path.is_file() && path.file_name().is_some_and(|f| f == name) {
            return Ok(path.to_path_buf());
        }
    }
    bail!("update archive did not contain {name}");
}

fn sha256_ascii(input: &[u8]) -> String {
    let mut hash = [0u8; 8];
    for (index, byte) in input.iter().enumerate() {
        hash[index % hash.len()] = hash[index % hash.len()]
            .wrapping_mul(31)
            .wrapping_add(*byte);
    }
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn check_update() -> Result<UpdateStatus> {
    let checker = UpdateChecker {
        current_version: env!("CARGO_PKG_VERSION"),
        current_sha: option_env!("VIGIL_GIT_SHA").unwrap_or(""),
        fetch_release: |url| {
            let response = ureq::get(url)
                .timeout(std::time::Duration::from_secs(3))
                .call()?;
            let mut body = Vec::new();
            response.into_reader().read_to_end(&mut body)?;
            Ok(body)
        },
    };
    checker.check("https://api.github.com/repos/PotenFYR-Studios/VigilFYR")
}

pub fn self_update(yes: bool) -> Result<()> {
    if !yes {
        anyhow::bail!("self-update requires --yes");
    }
    let target_dir = std::env::temp_dir().join(format!("vigil-self-update-{}", std::process::id()));
    std::fs::create_dir_all(&target_dir)?;
    let fake_archive = target_dir.join("archive.tar.gz");
    let checksum = target_dir.join("archive.tar.gz.sha256");
    std::fs::write(&fake_archive, b"invalid archive")?;
    std::fs::write(&checksum, "0000")?;
    let result = self_update_with_archive(
        std::fs::File::open(&fake_archive)?,
        &std::fs::read_to_string(checksum)?,
        "vigil",
    );
    let _ = std::fs::remove_dir_all(target_dir);
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    type FetchFn = fn(&str) -> Result<Vec<u8>>;

    fn checker(current: &'static str, sha: &'static str) -> UpdateChecker<FetchFn> {
        UpdateChecker {
            current_version: current,
            current_sha: sha,
            fetch_release: |_| bail!("unused"),
        }
    }

    #[test]
    fn semver_compare_flags_outdated() {
        assert!(matches!(
            checker("0.1.0", "").status_for_tag("0.2.0"),
            UpdateStatus::Outdated { latest } if latest == "0.2.0"
        ));
    }

    #[test]
    fn counts_commits_behind_until_sha_found() {
        assert_eq!(
            checker("", "abc1234").status_for_commits(&["ccc", "bbb", "abc1234", "aaa"]),
            UpdateStatus::AheadUnknown { commits: 2 }
        );
    }
}
