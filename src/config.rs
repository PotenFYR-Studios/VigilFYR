//! Config file load/save and dotted-key access for the CLI.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::engine::Mode;

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Mode::Enforce => "enforce",
            Mode::Audit => "audit",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for Mode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "enforce" => Ok(Mode::Enforce),
            "audit" => Ok(Mode::Audit),
            _ => Err(format!("invalid mode: {s}")),
        }
    }
}

/// Per-agent enforcement level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentMode {
    Off,
    Audit,
    Enforce,
}

impl std::fmt::Display for AgentMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AgentMode::Off => "off",
            AgentMode::Audit => "audit",
            AgentMode::Enforce => "enforce",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for AgentMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "off" => Ok(AgentMode::Off),
            "audit" => Ok(AgentMode::Audit),
            "enforce" => Ok(AgentMode::Enforce),
            _ => Err(format!("invalid agent mode: {s}")),
        }
    }
}

/// Configuration root, mirroring the locked schema. A missing file yields
/// these defaults.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub general: General,
    pub daemon: Daemon,
    pub masking: Masking,
    pub rules: Rules,
    pub agents: HashMap<String, AgentMode>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            general: General::default(),
            daemon: Daemon::default(),
            masking: Masking::default(),
            rules: Rules::default(),
            agents: default_agents(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct General {
    pub enabled: bool,
    pub mode: Mode,
    #[serde(default = "default_true")]
    pub check_updates: bool,
}

impl Default for General {
    fn default() -> Self {
        General {
            enabled: true,
            mode: Mode::Enforce,
            check_updates: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Daemon {
    pub autostart: bool,
    pub tray: bool,
    pub autostart_enabled: bool,
}

impl Default for Daemon {
    fn default() -> Self {
        Daemon {
            autostart: true,
            tray: true,
            autostart_enabled: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Masking {
    pub enabled: bool,
    pub patterns: Vec<String>,
}

impl Default for Masking {
    fn default() -> Self {
        Masking {
            enabled: false,
            patterns: default_masking_patterns(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Rules {
    pub remote_update: bool,
}

impl Default for Rules {
    fn default() -> Self {
        Rules {
            remote_update: true,
        }
    }
}

fn default_agents() -> HashMap<String, AgentMode> {
    HashMap::from([("claude-code".to_string(), AgentMode::Enforce)])
}

fn default_true() -> bool {
    true
}

fn default_masking_patterns() -> Vec<String> {
    [
        "aws_key",
        "github_pat",
        "openai_key",
        "google_api_key",
        "anthropic_key",
        "slack_token",
        "private_token",
        "stripe_key",
        "sendgrid_key",
        "twilio_key",
        "connection_uri",
        "jwt",
        "private_key",
        "env_values",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("unknown config key: {0}")]
    UnknownKey(String),
    #[error("invalid value for {key}: {value}")]
    InvalidValue { key: String, value: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),
    #[error(transparent)]
    TomlSer(#[from] toml::ser::Error),
}

impl Config {
    /// Load from the default location (~/.vigil/config.toml); a missing
    /// file yields the defaults.
    pub fn load() -> Self {
        let path = config_path();
        Self::load_from(&path).unwrap_or_default()
    }

    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(s) => Ok(toml::from_str(&s)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }

    /// Save to the default location (~/.vigil/config.toml), creating
    /// parent dirs as needed.
    pub fn save(&self) -> Result<(), ConfigError> {
        self.save_to(&config_path())
    }

    pub fn save_to(&self, path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Set a dotted key like "masking.enabled" or "agents.<id>". Unknown
    /// agent ids are created; other unknown keys are errors.
    pub fn set(&mut self, dotted_key: &str, value: &str) -> Result<(), ConfigError> {
        match dotted_key {
            "general.enabled" => self.general.enabled = parse_bool(dotted_key, value)?,
            "general.mode" => {
                self.general.mode = value.parse().map_err(|_| ConfigError::InvalidValue {
                    key: dotted_key.to_string(),
                    value: value.to_string(),
                })?;
            }
            "general.check_updates" => self.general.check_updates = parse_bool(dotted_key, value)?,
            "daemon.autostart" => self.daemon.autostart = parse_bool(dotted_key, value)?,
            "daemon.tray" => self.daemon.tray = parse_bool(dotted_key, value)?,
            "daemon.autostart_enabled" => {
                self.daemon.autostart_enabled = parse_bool(dotted_key, value)?
            }
            "masking.enabled" => self.masking.enabled = parse_bool(dotted_key, value)?,
            "rules.remote_update" => self.rules.remote_update = parse_bool(dotted_key, value)?,
            other if other.starts_with("agents.") => {
                let id = other.trim_start_matches("agents.");
                if id.is_empty() {
                    return Err(ConfigError::UnknownKey(other.to_string()));
                }
                let mode: AgentMode = value.parse().map_err(|_| ConfigError::InvalidValue {
                    key: dotted_key.to_string(),
                    value: value.to_string(),
                })?;
                self.agents.insert(id.to_string(), mode);
            }
            other => return Err(ConfigError::UnknownKey(other.to_string())),
        }
        Ok(())
    }

    /// Get a dotted key as a display string, e.g. "true", "enforce".
    pub fn get(&self, dotted_key: &str) -> Result<String, ConfigError> {
        match dotted_key {
            "general.enabled" => Ok(self.general.enabled.to_string()),
            "general.mode" => Ok(self.general.mode.to_string()),
            "general.check_updates" => Ok(self.general.check_updates.to_string()),
            "daemon.autostart" => Ok(self.daemon.autostart.to_string()),
            "daemon.tray" => Ok(self.daemon.tray.to_string()),
            "daemon.autostart_enabled" => Ok(self.daemon.autostart_enabled.to_string()),
            "masking.enabled" => Ok(self.masking.enabled.to_string()),
            "rules.remote_update" => Ok(self.rules.remote_update.to_string()),
            other if other.starts_with("agents.") => {
                let id = other.trim_start_matches("agents.");
                match self.agents.get(id) {
                    Some(m) => Ok(m.to_string()),
                    None => Err(ConfigError::UnknownKey(other.to_string())),
                }
            }
            other => Err(ConfigError::UnknownKey(other.to_string())),
        }
    }
}

fn parse_bool(key: &str, value: &str) -> Result<bool, ConfigError> {
    value
        .parse::<bool>()
        .map_err(|_| ConfigError::InvalidValue {
            key: key.to_string(),
            value: value.to_string(),
        })
}

fn config_path() -> PathBuf {
    crate::agents::home().join(".vigil").join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("vigil-config-{}-{tag}", std::process::id()))
    }

    #[test]
    fn load_missing_file_gives_defaults() {
        let c = Config::load_from(&tmp_path("missing")).unwrap();
        assert!(c.general.enabled);
        assert_eq!(c.general.mode, Mode::Enforce);
        assert!(c.daemon.autostart && c.daemon.tray);
        assert!(!c.masking.enabled);
        assert_eq!(
            c.masking.patterns.first().map(String::as_str),
            Some("aws_key")
        );
        assert!(c.rules.remote_update);
        assert_eq!(c.agents.get("claude-code"), Some(&AgentMode::Enforce));
    }

    #[test]
    fn set_save_load_round_trip_preserves_change() {
        let path = tmp_path("roundtrip");
        let mut c = Config::load_from(&path).unwrap();
        c.set("masking.enabled", "true").unwrap();
        c.save_to(&path).unwrap();

        let reloaded = Config::load_from(&path).unwrap();
        assert!(reloaded.masking.enabled);
        assert_eq!(
            reloaded.agents.get("claude-code"),
            Some(&AgentMode::Enforce)
        );

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn set_bad_value_is_error() {
        let mut c = Config::default();
        assert!(c.set("general.mode", "yolo").is_err());
        assert!(c.set("masking.enabled", "maybe").is_err());
        assert!(c.set("nope.nope", "true").is_err());
        assert!(c.set("general.enabled", "true").is_ok());
        assert!(c.get("nope.nope").is_err());
    }

    #[test]
    fn agents_map_extends_with_unknown_id() {
        let path = tmp_path("agents");
        let mut c = Config::load_from(&path).unwrap();
        c.set("agents.codex-cli", "audit").unwrap();
        c.save_to(&path).unwrap();

        let reloaded = Config::load_from(&path).unwrap();
        assert_eq!(reloaded.agents.get("codex-cli"), Some(&AgentMode::Audit));
        assert_eq!(
            reloaded.agents.get("claude-code"),
            Some(&AgentMode::Enforce)
        );
        assert_eq!(c.get("agents.codex-cli").unwrap(), "audit");

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn get_reads_defaults() {
        let c = Config::default();
        assert_eq!(c.get("general.mode").unwrap(), "enforce");
        assert_eq!(c.get("general.enabled").unwrap(), "true");
        assert_eq!(c.get("masking.enabled").unwrap(), "false");
        assert_eq!(c.get("daemon.tray").unwrap(), "true");
        assert_eq!(c.get("rules.remote_update").unwrap(), "true");
    }
}
