//! Rule synchronization: effective ruleset assembly and remote updates.

use std::path::PathBuf;

use anyhow::Result;

use crate::config::Config;
use crate::rules::{builtin_rules, load_rules, Rule, Ruleset};

/// Load `*.toml` rules from `dirs` (later overrides earlier by id), with
/// the builtin core ruleset underneath. Each loaded rule is stamped with
/// its source label. Unreadable dirs are skipped (never fail boot).
fn load_rules_with_builtin(dirs: &[(String, std::path::PathBuf)]) -> Vec<Rule> {
    let mut merged: Vec<Rule> = builtin_rules();
    for r in &mut merged {
        r.source = "builtin".to_string();
    }
    // Load each dir separately so per-rule provenance can be stamped.
    for (source, dir) in dirs {
        let rules = match load_rules(std::slice::from_ref(dir)) {
            Ok(rules) => rules,
            Err(e) => {
                eprintln!(
                    "vigil: failed to load {source} rules from {}: {e}",
                    dir.display()
                );
                continue;
            }
        };
        for mut r in rules {
            r.source = source.clone();
            match merged.iter_mut().find(|m| m.id == r.id) {
                Some(m) => *m = r,
                None => merged.push(r),
            }
        }
    }
    merged
}

/// Remote rules tarball (GitHub source archive of vigil-rules main).
pub const REMOTE_RULES_URL: &str =
    "https://github.com/PotenFYR-Studios/vigil-rules/archive/refs/heads/main.tar.gz";

/// Rules directories in override order (later overrides earlier by id):
/// builtin → remote → user (~/.vigil/rules) → project (./.vigil/rules).
pub fn rules_dirs() -> Vec<(String, PathBuf)> {
    let home = crate::agents::home();
    vec![
        (
            "remote".to_string(),
            home.join(".vigil").join("rules").join("remote"),
        ),
        ("user".to_string(), home.join(".vigil").join("rules")),
        ("project".to_string(), PathBuf::from(".vigil").join("rules")),
    ]
}

/// Assemble the effective ruleset: builtin → remote → user → project,
/// later ids override earlier. Mode is applied per-evaluate, not here.
pub fn load_effective_rules(cfg: &Config) -> Ruleset {
    let mut dirs = rules_dirs();
    let _ = &cfg;
    let extension_rules =
        crate::ext::extension_rules(&crate::ext::load_extensions(crate::ext::extensions_root()));
    for (source, dir) in extension_rules {
        dirs.push((source, dir.parent().unwrap_or(&dir).to_path_buf()));
    }
    let _ = cfg;
    let rules = load_rules_with_builtin(&dirs);
    Ruleset::compile(rules)
}

fn real_fetch(url: &str) -> Result<Vec<u8>> {
    let resp = ureq::get(url).call()?;
    let mut buf = Vec::new();
    resp.into_reader().read_to_end(&mut buf)?;
    Ok(buf)
}

/// Download the remote rules tarball into `<remote dir>`, extracting only
/// `*.toml` files (tarball path prefixes stripped). On any network or
/// parse error the existing copy is kept and a note returned - never fail.
/// Gated by `rules.remote_update` when disabled skips the fetch entirely.
pub fn sync_remote_rules(cfg: &Config) -> Result<String> {
    if !cfg.rules.remote_update {
        return Ok("remote update disabled by rules.remote_update".to_string());
    }
    let dirs = rules_dirs();
    let remote_dir = dirs[0].1.clone();
    sync_remote_rules_into(&[("remote".to_string(), remote_dir.clone())], real_fetch)
}

/// Inner sync with injectable fetcher. Returns a human note; existing
/// rules are kept whenever the fetch or extraction fails.
pub fn sync_remote_rules_into<F>(dirs: &[(String, std::path::PathBuf)], fetch: F) -> Result<String>
where
    F: Fn(&str) -> Result<Vec<u8>>,
{
    let remote_dir = &dirs[0].1;
    let bytes = match fetch(REMOTE_RULES_URL) {
        Ok(b) => b,
        Err(e) => {
            return Ok(format!(
                "remote update failed ({}); kept existing rules in {}",
                e,
                remote_dir.display()
            ))
        }
    };
    let gz = flate2::read::GzDecoder::new(bytes.as_slice());
    let mut archive = tar::Archive::new(gz);

    std::fs::create_dir_all(remote_dir)?;
    // Extract to a staging dir, then swap in, so a partial tarball cannot
    // clobber a good copy.
    let staging = remote_dir.with_extension("staging");
    if staging.exists() {
        std::fs::remove_dir_all(&staging)?;
    }
    std::fs::create_dir_all(&staging)?;

    let mut extracted = 0usize;
    let mut skipped = 0usize;
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();
        // Reject traversal attempts before any path join: ParentDir and
        // absolute/RootDir components must never reach the staging dir.
        let suspicious = path.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        });
        if suspicious {
            eprintln!(
                "vigil: skipping suspicious tarball entry {:?}",
                path.display()
            );
            skipped += 1;
            continue;
        }
        if path.extension().is_some_and(|e| e == "toml") {
            // Strip the tarball's top-level dir (e.g. vigil-rules-main/).
            let rel: PathBuf = path
                .components()
                .skip(1)
                .collect::<Vec<_>>()
                .into_iter()
                .collect();
            if rel.as_os_str().is_empty() {
                continue;
            }
            let dest = staging.join(&rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            entry.unpack(&dest)?;
            extracted += 1;
        }
    }
    if extracted == 0 {
        std::fs::remove_dir_all(&staging).ok();
        return Ok(format!(
            "remote tarball contained no rules; kept existing rules in {}",
            remote_dir.display()
        ));
    }
    if remote_dir.exists() {
        std::fs::remove_dir_all(remote_dir)?;
    }
    std::fs::rename(&staging, remote_dir)?;
    let skipped_note = if skipped > 0 {
        format!("; skipped {skipped} suspicious entries")
    } else {
        String::new()
    };
    Ok(format!(
        "updated {} rule files in {}{skipped_note}",
        extracted,
        remote_dir.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Action, VerdictAction};
    use crate::rules::Rule;
    use std::fs;

    fn tmp(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("vigil-sync-{}-{tag}", std::process::id()))
    }

    fn write_rule(dir: &std::path::Path, name: &str, id: &str, action: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(
            dir.join(name),
            format!("id = \"{id}\"\nscope = [\"read\"]\npaths = [\"**/marker-{id}\"]\naction = \"{action}\""),
        )
        .unwrap();
    }

    #[test]
    fn precedence_project_overrides_user_overrides_builtin() {
        let root = tmp("prec");
        let user = root.join("user");
        let project = root.join("project");
        write_rule(&user, "a.toml", "deny-env-files", "warn");
        write_rule(&project, "b.toml", "deny-env-files", "deny");

        let dirs = vec![
            ("user".to_string(), user.clone()),
            ("project".to_string(), project.clone()),
        ];
        let rules = load_rules_with_builtin(&dirs);
        let eff = rules.iter().find(|r| r.id == "deny-env-files").unwrap();
        assert_eq!(eff.action, VerdictAction::Deny, "project override wins");
        assert_eq!(eff.source, "project");
        // builtin-only rule survives
        assert!(rules.iter().any(|r| r.id == "deny-ssh-dir"));
        assert_eq!(
            rules
                .iter()
                .find(|r| r.id == "deny-ssh-dir")
                .unwrap()
                .source,
            "builtin"
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn remote_offline_uses_stale_copy() {
        let root = tmp("stale");
        let remote = root.join("remote");
        write_rule(&remote, "r.toml", "deny-key-files", "warn");

        // Fetcher that simulates being offline.
        let fetch = |_url: &str| -> Result<Vec<u8>> {
            anyhow::bail!("network unreachable");
        };
        let dirs = vec![("remote".to_string(), remote.clone())];
        let note = sync_remote_rules_into(&dirs, fetch).unwrap();
        assert!(
            note.contains("offline") || note.contains("kept"),
            "note: {note}"
        );
        // stale copy still there and used
        let rules = load_rules_with_builtin(&dirs);
        let eff = rules.iter().find(|r| r.id == "deny-key-files").unwrap();
        assert_eq!(eff.action, VerdictAction::Warn);
        assert_eq!(eff.source, "remote");
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn sync_extracts_toml_from_tarball() {
        let root = tmp("tar");
        let remote = root.join("remote");
        // Build a tarball in the layout GitHub serves: top-level dir with files.
        let inner = root.join("vigil-rules-main");
        write_rule(&inner, "core.toml", "deny-key-files", "deny");

        let tar_path = root.join("main.tar.gz");
        {
            let f = fs::File::create(&tar_path).unwrap();
            let gz = flate2::write::GzEncoder::new(f, flate2::Compression::fast());
            let mut tar = tar::Builder::new(gz);
            tar.append_dir_all("vigil-rules-main", &inner).unwrap();
            tar.into_inner().unwrap().finish().unwrap();
        }
        let bytes = fs::read(&tar_path).unwrap();

        let dirs = vec![("remote".to_string(), remote.clone())];
        let fetch = move |_url: &str| -> Result<Vec<u8>> { Ok(bytes.clone()) };
        let note = sync_remote_rules_into(&dirs, fetch).unwrap();
        assert!(note.contains("updated"), "note: {note}");
        assert!(remote.join("core.toml").is_file());
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn parse_toml_stamps_no_source() {
        let r =
            Rule::parse_toml("id = \"x\"\nscope = [\"read\"]\npaths = [\"p\"]\naction = \"deny\"")
                .unwrap()
                .pop()
                .unwrap();
        assert_eq!(r.source, "");
        let _ = Action::Read;
        let _ = builtin_rules().len();
    }

    #[test]
    fn sync_rejects_traversal_entries() {
        let root = tmp("traversal");
        let remote = root.join("remote");
        fs::create_dir_all(&root).unwrap();

        // Tarball with a legit file and a `../evil.toml` escape attempt.
        // The tar crate refuses to *write* `..` paths, so the malicious
        // entry is emitted as raw 512-byte ustar headers.
        let tar_path = root.join("evil.tar.gz");
        {
            let f = fs::File::create(&tar_path).unwrap();
            let gz = flate2::write::GzEncoder::new(f, flate2::Compression::fast());
            let mut tar = tar::Builder::new(gz);
            let good = b"id = \"good\"\nscope = [\"read\"]\npaths = [\"p\"]\naction = \"allow\"\n";
            let mut header = tar::Header::new_gnu();
            header.set_size(good.len() as u64);
            header.set_cksum();
            tar.append_data(&mut header, "vigil-rules-main/good.toml", &good[..])
                .unwrap();

            // Hand-rolled entry: name "vigil-rules-main/../evil.toml".
            let mut raw = [0u8; 512];
            let name = b"vigil-rules-main/../evil.toml";
            raw[..name.len()].copy_from_slice(name);
            raw[100..108].copy_from_slice(b"0000644\0"); // mode
            raw[108..116].copy_from_slice(b"0000000\0"); // uid
            raw[116..124].copy_from_slice(b"0000000\0"); // gid
            raw[124..136].copy_from_slice(b"00000000074\0"); // size = 60
            raw[136..148].copy_from_slice(b"00000000000\0"); // mtime
            raw[156] = b'0'; // regular file
            raw[257..262].copy_from_slice(b"ustar");
            raw[263..265].copy_from_slice(b"00");
            raw[148..156].copy_from_slice(b"        "); // spaces during calc
            let cksum: u32 = raw.iter().map(|b| *b as u32).sum();
            raw[148..156].copy_from_slice(format!("{:06o}\0 ", cksum).as_bytes());
            use std::io::Write as _;
            tar.get_mut().write_all(&raw).unwrap();
            tar.get_mut().write_all(good).unwrap();
            // Two zero blocks terminate the archive.
            tar.get_mut().write_all(&[0u8; 1024]).unwrap();
            tar.into_inner().unwrap().finish().unwrap();
        }
        let bytes = fs::read(&tar_path).unwrap();

        let dirs = vec![("remote".to_string(), remote.clone())];
        let fetch = move |_url: &str| -> Result<Vec<u8>> { Ok(bytes.clone()) };
        let note = sync_remote_rules_into(&dirs, fetch).unwrap();
        assert!(note.contains("updated"), "good file still lands: {note}");
        assert!(remote.join("good.toml").is_file());
        assert!(
            !root.join("evil.toml").exists(),
            "traversal entry must not escape the staging dir"
        );
        fs::remove_dir_all(&root).ok();
    }
}
