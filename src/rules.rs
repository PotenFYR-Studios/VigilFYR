//! Rule definition, TOML parsing, and directory loading with id override.

use std::path::PathBuf;

use serde::Deserialize;

use crate::event::{Action, VerdictAction};

#[derive(Debug, Deserialize)]
struct RuleToml {
    id: String,
    #[serde(default)]
    description: String,
    scope: Vec<Action>,
    paths: Vec<String>,
    #[serde(default)]
    commands: Vec<String>,
    #[serde(default)]
    agents: Vec<String>,
    action: VerdictAction,
    #[serde(default)]
    severity: Severity,
    #[serde(default = "default_enabled")]
    enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

#[derive(Debug, PartialEq)]
pub struct Rule {
    pub id: String,
    pub description: String,
    pub scope: Vec<Action>,
    pub paths: Vec<String>,
    pub commands: Vec<String>,
    pub agents: Vec<String>,
    pub action: VerdictAction,
    pub severity: Severity,
    pub enabled: bool,
}

impl Rule {
    pub fn parse_toml(s: &str) -> anyhow::Result<Rule> {
        let t: RuleToml = toml::from_str(s)?;
        Ok(Rule {
            id: t.id,
            description: t.description,
            scope: t.scope,
            paths: t.paths,
            commands: t.commands,
            agents: t.agents,
            action: t.action,
            severity: t.severity,
            enabled: t.enabled,
        })
    }
}

/// Merge rules by id: later wins, keeps position of first occurrence.
fn merge_by_id(rules: Vec<Rule>) -> Vec<Rule> {
    let mut merged: Vec<Rule> = Vec::new();
    for r in rules {
        match merged.iter_mut().find(|m| m.id == r.id) {
            Some(m) => *m = r,
            None => merged.push(r),
        }
    }
    merged
}

/// Load `*.toml` rule files from each dir (sorted by filename); later dirs
/// override earlier by id.
pub fn load_rules(dirs: &[PathBuf]) -> anyhow::Result<Vec<Rule>> {
    let mut rules = Vec::new();
    for dir in dirs {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        entries.sort();
        for path in entries {
            let text = std::fs::read_to_string(&path)?;
            rules.push(Rule::parse_toml(&text)?);
        }
    }
    Ok(merge_by_id(rules))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rule_toml_with_defaults() {
        let r = Rule::parse_toml(
            "id = \"x\"\nscope = [\"read\"]\npaths = [\"**/.env\"]\naction = \"deny\"",
        )
        .unwrap();
        assert_eq!(r.severity, Severity::Medium); // default
        assert!(r.enabled);
        assert_eq!(r.id, "x");
        assert_eq!(r.scope, vec![Action::Read]);
        assert_eq!(r.paths, vec!["**/.env"]);
        assert_eq!(r.action, VerdictAction::Deny);
    }

    #[test]
    fn later_dir_overrides_same_id() {
        let merged = merge_by_id(vec![rule("x", "deny"), rule("x", "allow")]);
        assert_eq!(merged[0].action, VerdictAction::Allow);
        assert_eq!(merged.len(), 1);
    }

    fn rule(id: &str, action: &str) -> Rule {
        Rule::parse_toml(&format!(
            "id = \"{id}\"\nscope = [\"read\"]\npaths = [\"**/.env\"]\naction = \"{action}\""
        ))
        .unwrap()
    }
}
