//! Rule definition, TOML parsing, and directory loading with id override.

use std::fmt;
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

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    /// Which rules source this rule came from ("builtin", "remote",
    /// "user", "project"); empty until stamped by the loader.
    pub source: String,
}

impl Rule {
    pub fn parse_toml(s: &str) -> anyhow::Result<Vec<Rule>> {
        let value: toml::Value = toml::from_str(s)?;
        if let Some(rules) = value.get("rule").and_then(|v| v.as_array()) {
            let parsed = rules
                .iter()
                .map(|value| {
                    value
                        .clone()
                        .try_into::<RuleToml>()
                        .map(RuleToml::into_rule)
                })
                .collect::<Result<Vec<_>, _>>()?;
            if parsed.is_empty() {
                return Err(anyhow::anyhow!("empty rule list"));
            }
            return Ok(parsed);
        }
        Ok(vec![value.try_into::<RuleToml>()?.into_rule()])
    }

    /// Parse every rule in a TOML file. Kept as a separate seam for
    /// callers that want all rules, even when the file contains one.
    pub fn parse_toml_all(s: &str) -> anyhow::Result<Vec<Rule>> {
        Self::parse_toml(s)
    }
}

impl RuleToml {
    fn into_rule(self) -> Rule {
        Rule {
            id: self.id,
            description: self.description,
            scope: self.scope,
            paths: self.paths,
            commands: self.commands,
            agents: self.agents,
            action: self.action,
            severity: self.severity,
            enabled: self.enabled,
            source: String::new(),
        }
    }
}

/// Built-in rules embedded at compile time from `rules/core.toml`.
pub fn builtin_rules() -> Vec<Rule> {
    #[derive(Deserialize)]
    struct CoreFile {
        #[serde(default)]
        rule: Vec<RuleToml>,
    }
    let file: CoreFile = toml::from_str(include_str!("../rules/core.toml"))
        .expect("embedded rules/core.toml must be valid");
    file.rule.into_iter().map(RuleToml::into_rule).collect()
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
/// override earlier by id. Missing dirs are treated as empty.
pub fn load_rules(dirs: &[PathBuf]) -> anyhow::Result<Vec<Rule>> {
    let mut rules = Vec::new();
    for dir in dirs {
        let read_dir = match std::fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        };
        let mut entries: Vec<PathBuf> = read_dir
            .collect::<std::io::Result<Vec<_>>>()?
            .into_iter()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        entries.sort();
        for path in entries {
            let text = std::fs::read_to_string(&path)?;
            rules.extend(Rule::parse_toml(&text)?);
        }
    }
    Ok(merge_by_id(rules))
}

/// Compiled rule set: precomputed matchers, first match wins.
#[derive(Debug, Default)]
pub struct Ruleset {
    pub rules: Vec<Rule>,
    pub compiled: Vec<CompiledRule>,
}

/// A rule with its matchers precompiled once at load time.
#[derive(Debug)]
pub struct CompiledRule {
    pub rule: Rule,
    pub paths: globset::GlobSet,
    pub commands: regex::RegexSet,
}

impl Ruleset {
    /// Compile the given rules into matchers. File order = priority.
    pub fn compile(rules: Vec<Rule>) -> Ruleset {
        let compiled = rules
            .iter()
            .map(|r| CompiledRule {
                rule: r.clone(),
                paths: compile_globs(&r.paths),
                commands: compile_regexes(&r.commands),
            })
            .collect();
        Ruleset { rules, compiled }
    }
}

fn compile_globs(patterns: &[String]) -> globset::GlobSet {
    let mut builder = globset::GlobSetBuilder::new();
    for p in patterns {
        if let Ok(glob) = globset::Glob::new(p) {
            builder.add(glob);
        }
    }
    builder
        .build()
        .unwrap_or_else(|_| globset::GlobSet::empty())
}

fn compile_regexes(patterns: &[String]) -> regex::RegexSet {
    regex::RegexSet::new(patterns.iter().map(String::as_str))
        .unwrap_or_else(|_| regex::RegexSet::empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rule_toml_with_defaults() {
        let r = Rule::parse_toml(
            "id = \"x\"\nscope = [\"read\"]\npaths = [\"**/.env\"]\naction = \"deny\"",
        )
        .unwrap()
        .pop()
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

    #[test]
    fn compile_preserves_rules() {
        let ruleset = Ruleset::compile(vec![rule("x", "deny"), rule("y", "warn")]);
        assert_eq!(ruleset.rules.len(), 2);
        assert_eq!(ruleset.rules[0].id, "x");
        assert_eq!(ruleset.rules[1].id, "y");
    }

    #[test]
    fn load_rules_skips_missing_dir_and_sorts_files() {
        let dir = std::env::temp_dir().join(format!("vigil-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("b.toml"), rule_toml("b", "deny")).unwrap();
        std::fs::write(dir.join("a.toml"), rule_toml("a", "warn")).unwrap();
        std::fs::write(dir.join("ignored.txt"), "not toml").unwrap();

        let missing = dir.join("does-not-exist");
        let rules = load_rules(&[missing, dir.clone()]).unwrap();

        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].id, "a"); // a.toml sorts before b.toml
        assert_eq!(rules[0].action, VerdictAction::Warn);
        assert_eq!(rules[1].id, "b");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    fn rule_toml(id: &str, action: &str) -> String {
        format!("id = \"{id}\"\nscope = [\"read\"]\npaths = [\"**/.env\"]\naction = \"{action}\"")
    }

    fn rule(id: &str, action: &str) -> Rule {
        Rule::parse_toml(&rule_toml(id, action))
            .unwrap()
            .pop()
            .unwrap()
    }
}
