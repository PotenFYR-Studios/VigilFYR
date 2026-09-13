# Security Policy

Vigil is a security tool. Reports about anything below are important to
us, and we treat them seriously.

## Supported versions

| Version | Supported |
| --- | --- |
| 0.1.x | yes |

Older releases get no patches; please update to the latest 0.1.x.

## How to report

Please use GitHub's **private vulnerability reporting** for this
repository (Security tab -> "Report a vulnerability"). This keeps details
out of public view until a fix is ready. Do not open a public issue for
anything you believe is exploitable.

For non-security bugs, use the regular issue templates instead.

## What we need in a report

- What you observed, including how to recognize the failure
- Your platform: OS, architecture, Vigil version, which agents are
  installed (Claude Code / Codex / Gemini CLI / Cursor / OpenCode /
  Hermes / other), and per-agent mode (`enforce` / `audit` / `off`)
- Logs relevant to the issue (redacted as below), plus config and any
  custom rules where relevant

Please do NOT include in reports, issues, or any public channel:

- Secrets that Vigil would normally guard: API keys, tokens, SSH or
  cloud credentials, wallet data (redact them)
- Working exploit code or proof-of-concept payloads - a clear
  description of the impact is enough at this stage

## In scope

- Hook enforcement and the exit-code contract (`0` allow, `2` deny,
  `3` ask), including any way to bypass the hook path or spoof verdicts
- The rule engine: rule id overriding, precedence, layering of
  built-in -> user -> project rulesets, remote rule sync integrity
- The `vigil shim` command wrapper and the audit filesystem watcher
- Masking: patterns that should redact but leak, or leak more than the
  documented 4-character prefix
- The daemon and IPC surface (Unix socket permissions, message
  handling), extensions loading (`~/.vigil/extensions/`)
- Install path: install.sh / install.ps1 downloads and checksum
  verification

## Out of scope

- Vulnerabilities in supported agents themselves or in dependencies;
  report those upstream
- Bypasses that require modifying Vigil's own config or binaries on the
  victim's machine (a local attacker with those rights has already won)
- Reports from automated scanners without a demonstrated impact

## What to expect

We will acknowledge reports as quickly as we can and keep you updated
as we investigate. Fixes land on `master` and ship in the next patch
release; you will be credited in the changelog unless you prefer
otherwise.

## Guarding guarantees

The product's core promise is documented in
[docs/security-model.md](docs/security-model.md): tool actions are
evaluated against the ruleset before they execute, verdicts are
local and synchronous, and the hook fails open only on daemon or
ruleset error so a broken guard never breaks your agent. Reports that
break any of these guarantees are treated as high priority.
