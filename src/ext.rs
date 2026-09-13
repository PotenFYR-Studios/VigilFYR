//! Extension discovery, rule/pattern merging, and event hook execution.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ExtensionManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub provides: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Extension {
    pub manifest: ExtensionManifest,
    pub root: PathBuf,
}

pub fn extensions_root() -> PathBuf {
    crate::agents::home().join(".vigil/extensions")
}

pub fn load_extensions(root: impl AsRef<Path>) -> Vec<Extension> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return found;
    };
    for entry in entries.flatten() {
        let dir = entry.path();
        let manifest_path = dir.join("manifest.toml");
        if !dir.is_dir() || !manifest_path.is_file() {
            eprintln!(
                "vigil: skipping invalid extension directory {} (missing manifest.toml)",
                dir.display()
            );
            continue;
        }
        match std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("read {}", manifest_path.display()))
            .and_then(|text| toml::from_str::<ExtensionManifest>(&text).map_err(Into::into))
        {
            Ok(manifest) => found.push(Extension {
                manifest,
                root: dir,
            }),
            Err(error) => eprintln!("vigil: invalid extension manifest: {error:#}"),
        }
    }
    found.sort_by(|a, b| a.manifest.name.cmp(&b.manifest.name));
    found
}

pub fn extension_rules(extensions: &[Extension]) -> Vec<(String, PathBuf)> {
    extensions
        .iter()
        .flat_map(|ext| {
            ext.root
                .join("rules")
                .read_dir()
                .into_iter()
                .flatten()
                .flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    (path.extension() == Some("toml".as_ref())).then_some(path)
                })
                .map(|path| (format!("ext:{}", ext.manifest.name), path))
                .collect::<Vec<_>>()
        })
        .collect()
}

pub fn install_extension(source: &str, root: impl AsRef<Path>) -> Result<PathBuf> {
    let destination_root = root.as_ref();
    std::fs::create_dir_all(destination_root)?;
    if source.starts_with("http://") || source.starts_with("https://") || source.contains(':') {
        let name = source
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("extension")
            .trim_end_matches(".git");
        let destination = destination_root.join(name);
        if destination.exists() {
            anyhow::bail!("extension already exists: {name}");
        }
        Command::new("git")
            .args(["clone", "--depth", "1", source])
            .arg(&destination)
            .status()?;
        if !destination.join("manifest.toml").is_file() {
            let _ = std::fs::remove_dir_all(&destination);
            anyhow::bail!("repository has no manifest.toml");
        }
        Ok(destination)
    } else {
        let source_path = PathBuf::from(source);
        if !source_path.join("manifest.toml").is_file() {
            anyhow::bail!("source has no manifest.toml");
        }
        let name = source_path
            .file_name()
            .map(std::ffi::OsStr::to_string_lossy)
            .unwrap_or_else(|| "extension".into());
        let destination = destination_root.join(name.as_ref());
        if destination.exists() {
            anyhow::bail!("extension already exists: {name}");
        }
        copy_dir(&source_path, &destination)?;
        Ok(destination)
    }
}

pub fn remove_extension(name: &str, root: impl AsRef<Path>) -> Result<()> {
    let destination = root.as_ref().join(name);
    if !destination.starts_with(root.as_ref()) || destination == root.as_ref() {
        anyhow::bail!("invalid extension name");
    }
    std::fs::remove_dir_all(destination)?;
    Ok(())
}

fn copy_dir(source: &Path, destination: &Path) -> Result<()> {
    std::fs::create_dir_all(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

pub fn extension_patterns(extensions: &[Extension]) -> Result<Vec<crate::mask::MaskPattern>> {
    let mut patterns = Vec::new();
    for ext in extensions {
        let dir = ext.root.join("patterns");
        for entry in dir.read_dir().into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().map(|e| e != "toml").unwrap_or(true) {
                continue;
            }
            let text = std::fs::read_to_string(&path)?;
            let file: crate::mask::PatternsFile = toml::from_str(&text)?;
            for pattern in file.pattern {
                patterns.push(crate::mask::MaskPattern::new(
                    &pattern.name,
                    &pattern.regex,
                )?);
            }
        }
    }
    Ok(patterns)
}

pub fn apply_event_hooks(extensions: &[Extension], record_json: &str) -> Result<Option<String>> {
    for ext in extensions {
        let hook_scripts: &[&str] = if cfg!(windows) {
            &["on-event.exe", "on-event.ps1", "on-event.sh"]
        } else {
            &["on-event.sh"]
        };
        for script_name in hook_scripts {
            let script = ext.root.join("hooks").join(script_name);
            if !script.is_file() {
                continue;
            }
            let mut command = if cfg!(windows) && script_name.ends_with(".ps1") {
                let mut command = Command::new("powershell.exe");
                command.arg("-NoProfile").arg("-File").arg(&script);
                command
            } else if cfg!(windows) && script_name.ends_with(".sh") {
                let mut command = Command::new("bash");
                command.arg(&script);
                command
            } else {
                Command::new(&script)
            };
            let mut child = command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .with_context(|| format!("run extension hook {}", script.display()))?;
            if let Some(stdin) = child.stdin.as_mut() {
                let record_length = record_json.len();
                let mut written = 0;
                while written < record_length {
                    match stdin.write(&record_json.as_bytes()[written..]) {
                        Ok(0) => break,
                        Ok(count) => written += count,
                        Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => break,
                        Err(error) => return Err(error.into()),
                    }
                }
            }
            let output = wait_timeout(&mut child, Duration::from_millis(500))?;
            if output.status.code() == Some(2)
                && !String::from_utf8_lossy(&output.stdout).trim().is_empty()
            {
                return Ok(Some(String::from_utf8_lossy(&output.stdout).into_owned()));
            }
        }
    }
    Ok(None)
}

fn wait_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> Result<std::process::Output> {
    let start = std::time::Instant::now();
    let stdout_handle = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    if let Some(mut stdout) = child.stdout.take() {
        let writer = stdout_handle.clone();
        std::thread::spawn(move || {
            let mut buffer = Vec::new();
            let _ = std::io::Read::read_to_end(&mut stdout, &mut buffer);
            if let Ok(mut guard) = writer.lock() {
                *guard = buffer;
            }
        });
    }
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(std::process::Output {
                status,
                stdout: stdout_handle
                    .lock()
                    .map(|guard| guard.clone())
                    .unwrap_or_default(),
                stderr: Vec::new(),
            });
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            let status = child.wait()?;
            return Ok(std::process::Output {
                status,
                stdout: stdout_handle
                    .lock()
                    .map(|guard| guard.clone())
                    .unwrap_or_default(),
                stderr: Vec::new(),
            });
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_missing_manifest_and_loads_valid_extension() {
        let root = std::env::temp_dir().join(format!("vigil-ext-{}", std::process::id()));
        let valid = root.join("valid");
        let invalid = root.join("invalid");
        std::fs::create_dir_all(valid.join("rules")).unwrap();
        std::fs::create_dir_all(invalid).unwrap();
        std::fs::write(
            valid.join("manifest.toml"),
            "name = \"valid\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::write(
            valid.join("rules/ext.toml"),
            "id = \"ext-rule\"\nscope = [\"read\"]\npaths = [\"**/ext-secret\"]\naction = \"deny\"\n",
        )
        .unwrap();
        let extensions = load_extensions(&root);
        assert_eq!(extensions.len(), 1);
        assert_eq!(extension_rules(&extensions).len(), 1);
        std::fs::remove_dir_all(root).ok();
    }
}
