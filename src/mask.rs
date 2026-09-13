//! Sensitive-data masker: named regex patterns, prefix-preserving
//! replacement. Patterns are embedded from `rules/patterns.toml` and can
//! be selected by name via `masking.patterns` config.

use serde::Deserialize;

/// A named masking pattern with its compiled regex.
#[derive(Debug, Clone)]
pub struct MaskPattern {
    pub name: String,
    regex: regex::Regex,
}

impl MaskPattern {
    pub fn new(name: &str, regex_src: &str) -> anyhow::Result<Self> {
        Ok(MaskPattern {
            name: name.to_string(),
            regex: regex::Regex::new(regex_src)?,
        })
    }
}

/// Pattern file shape (`rules/patterns.toml`, and user extension dirs).
#[derive(Debug, Deserialize)]
pub struct PatternsFile {
    #[serde(default)]
    pub pattern: Vec<PatternToml>,
}

#[derive(Debug, Deserialize)]
pub struct PatternToml {
    pub name: String,
    pub regex: String,
}

/// Built-in named patterns embedded from `rules/patterns.toml`.
pub fn builtin_patterns() -> Vec<MaskPattern> {
    let file: PatternsFile = toml::from_str(include_str!("../rules/patterns.toml"))
        .expect("embedded rules/patterns.toml must be valid");
    file.pattern
        .into_iter()
        .map(|p| {
            MaskPattern::new(&p.name, &p.regex)
                .unwrap_or_else(|e| panic!("pattern '{}' invalid: {e}", p.name))
        })
        .collect()
}

/// Select the named subset from the built-in library (unknown names are
/// ignored; empty selection means all patterns).
pub fn select_patterns(names: &[String]) -> Vec<MaskPattern> {
    let builtin = builtin_patterns();
    if names.is_empty() {
        return builtin;
    }
    builtin
        .into_iter()
        .filter(|p| names.iter().any(|n| n == &p.name))
        .collect()
}

/// Mask every pattern match in `text`, keeping the first 4 chars of each
/// match followed by bullets and a "(masked)" marker. Returns the masked
/// text and the number of replacements. Non-matching text is returned
/// unchanged (same allocation content, count 0).
pub fn mask_text(text: &str, patterns: &[MaskPattern]) -> (String, usize) {
    let mut out = text.to_string();
    let mut count = 0usize;
    for p in patterns {
        let replaced = p.regex.replace_all(&out, |caps: &regex::Captures| {
            count += 1;
            replace_match(caps.get(0).unwrap().as_str())
        });
        out = replaced.into_owned();
    }
    (out, count)
}

/// Prefix-preserving replacement: keep 4-char prefix, bullet the rest
/// (at least 4 bullets), append "(masked)".
fn replace_match(m: &str) -> String {
    let chars: Vec<char> = m.chars().collect();
    if chars.len() <= 4 {
        return "••••(masked)".to_string();
    }
    let prefix: String = chars[..4].iter().collect();
    let bullets = "•".repeat(chars.len() - 4);
    format!("{prefix}{bullets}(masked)")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aws() -> Vec<MaskPattern> {
        select_patterns(&["aws_key".to_string()])
    }

    fn all() -> Vec<MaskPattern> {
        builtin_patterns()
    }

    #[test]
    fn masks_aws_key_keeping_prefix() {
        let (out, n) = mask_text("key=AKIAIOSFODNN7EXAMPLE end", &aws());
        assert_eq!(out, "key=AKIA••••••••••••••••(masked) end");
        assert_eq!(n, 1);
    }

    #[test]
    fn masks_jwt_and_env_value_lines() {
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let text =
            format!("Authorization: Bearer {jwt}\nDB_PASSWORD=hunter2supersecret\nsafe line");
        let (out, n) = mask_text(&text, &all());
        assert_eq!(n, 2, "jwt + env value: {out}");
        assert!(
            out.starts_with("Authorization: Bearer eyJh"),
            "prefix kept: {out}"
        );
        assert!(out.contains("(masked)"));
        assert!(!out.contains(jwt), "jwt gone");
        assert!(!out.contains("hunter2supersecret"), "env value gone: {out}");
        assert!(out.contains("safe line"), "benign text untouched");
    }

    #[test]
    fn non_matching_text_untouched_zero_count() {
        let (out, n) = mask_text("hello", &aws());
        assert_eq!(out, "hello");
        assert_eq!(n, 0);
    }

    #[test]
    fn masks_github_pat_openai_key_and_private_key_block() {
        let text = "ghp_abcdefghijklmnopqrstuvwxyz0123456789 sk-proj-abcdefghijklmnopq\n-----BEGIN RSA PRIVATE KEY-----\nMIIB\n-----END RSA PRIVATE KEY-----";
        let (out, n) = mask_text(text, &all());
        assert!(out.starts_with("ghp_"), "prefix kept: {out}");
        assert!(out.contains("sk-p"), "openai prefix kept: {out}");
        assert!(!out.contains("abcdefghijklmnopqrstuvwxyz0123456789"));
        assert!(
            !out.contains("-----BEGIN RSA PRIVATE KEY-----"),
            "pem block masked: {out}"
        );
        assert!(n >= 3, "at least pat+key+pem: {n} in {out}");
    }

    #[test]
    fn builtin_library_has_all_named_patterns() {
        let all = all();
        for name in [
            "aws_key",
            "azure_storage_key",
            "azure_client_secret",
            "github_pat",
            "gitlab_pat",
            "openai_key",
            "google_api_key",
            "anthropic_key",
            "huggingface_token",
            "slack_token",
            "telegram_bot_token",
            "private_token",
            "npm_token",
            "pypi_token",
            "stripe_key",
            "sendgrid_key",
            "mailgun_key",
            "twilio_key",
            "cloudflare_api_token",
            "digitalocean_token",
            "heroku_api_key",
            "rubygems_key",
            "connection_uri",
            "sensitive_uri",
            "sensitive_assignment",
            "hex_secret",
            "base64_secret",
            "jwt",
            "private_key",
            "env_values",
        ] {
            assert!(all.iter().any(|p| p.name == name), "missing pattern {name}");
        }
    }

    #[test]
    fn select_patterns_filters_by_name() {
        let selected = select_patterns(&["jwt".to_string()]);
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].name, "jwt");
        // Unknown names select nothing.
        assert!(select_patterns(&["nope".to_string()]).is_empty());
    }
}
