# Rules Reference

Rules are TOML files. Vigil loads, in order:

1. **Built-in core ruleset** — embedded in the binary (`rules/core.toml` at build time).
2. **User rules** — `~/.vigil/rules/*.toml`
3. **Project rules** — `./.vigil/rules/*.toml` (highest precedence for shared, per-project policy)
4. **Remote feed** — community rules synced on boot and `vigil reload` (disable with `rules.remote_update = false`)

Later sources override earlier ones **by rule id**: a rule in `~/.vigil/rules/` with `id = "deny-env-files"` replaces the built-in rule of the same id. Within a single file, order is priority — **first matching rule wins**, so keep the most specific rules first. No match means **allow** (fail-open).

## Schema

```toml
[[rule]]
id = "deny-my-secrets"            # required, unique; reuse a built-in id to override it
description = "Block the company secrets dir"
scope = ["read", "write", "search"]  # required: which actions this rule fires on
paths = ["**/company-secrets/**"]    # glob patterns (globset), matched against event paths
commands = []                        # regex patterns, matched against exec command lines
agents = []                          # empty = all agents; otherwise agent ids, e.g. ["claude-code"]
action = "deny"                      # deny | allow | warn | mask
severity = "critical"                # low | medium | high | critical (default: medium)
enabled = true                       # default: true
```

Field notes:

- **`scope`** — event actions: `read`, `write`, `search`, `exec`, `net`.
- **`paths`** — globset globs. `**` spans directories. A rule matches when *any* pattern matches *any* path on the event.
- **`commands`** — Rust `regex` crate syntax, matched against the full command line of `exec`/`net` events. A rule matches when *any* command regex matches.
- A rule fires if **either** a path hits **or** a command hits (both checked; at least one must match).
- **`agents`** — restricts a rule to specific agents. Empty list = every agent.

## Actions and exit codes

| `action` | Effect | Hook exit code |
| :--- | :--- | :---: |
| `deny` | Block the tool call | `2` |
| `warn` | Allow, but record and flag in the TUI | `0` |
| `allow` | Explicitly allow (escape hatch past earlier broad rules) | `0` |
| `mask` | Redact sensitive content before the agent sees it (requires masking enabled) | `0` |
| — no match | Default: allow, logged as rule `default` | `0` |
| *audit mode* | Every `deny` is demoted to `warn` — nothing is blocked | `0` |

Exit code `3` (**ask**) is reserved for interactive confirmation by the calling agent. On daemon or ruleset error the hook **fails open** (allow) — a broken guard must never break your agent.

## Cookbook

Deny a project's private directory, except a subfolder:

```toml
[[rule]]
id = "deny-internal-docs"
description = "Keep internal docs away from agents"
scope = ["read", "search", "write"]
paths = ["**/internal/**"]
action = "deny"
severity = "high"

[[rule]]
id = "allow-internal-templates"
description = "Templates under internal/ are fine"
scope = ["read", "search"]
paths = ["**/internal/templates/**"]
action = "allow"
severity = "low"
```

`allow-internal-templates` must come **first** — first match wins.

Block destructive commands (patterns match the full command line):

```toml
[[rule]]
id = "deny-git-force-push"
description = "No force pushes from agents"
scope = ["exec"]
commands = ["git\\s+push\\s+.*(--force|-f)\\b"]
action = "deny"
severity = "high"
```

Target one agent only:

```toml
[[rule]]
id = "deny-codex-package-publish"
description = "Only Codex is restricted from publishing"
scope = ["exec"]
agents = ["codex"]
commands = ["npm\\s+publish|cargo\\s+publish"]
action = "deny"
severity = "high"
```

Audit-mode canary — watch, don't block, while calibrating:

```toml
[[rule]]
id = "warn-build-artifacts"
description = "Notice when agents wander into target/"
scope = ["read", "search"]
paths = ["**/target/**", "**/node_modules/**"]
action = "warn"
severity = "low"
```

Or flip the whole daemon to audit: `vigil config set general.mode audit` (then `vigil reload`). Demotion applies per verdict, so mixing `warn` and `deny` rules is fine — in audit mode everything behaves as `warn`.

Disable a built-in rule without deleting it:

```toml
# ~/.vigil/rules/overrides.toml
[[rule]]
id = "warn-net-curl-pipe-shell"
enabled = false
```

(Keep the required fields — `scope`, `paths` — to satisfy the schema.)

## Built-in rules at a glance

| Rule id | Covers | Action / severity |
| :--- | :--- | :--- |
| `deny-env-files` | `.env`, `.env.*` | deny / high |
| `deny-secrets-dir` | `**/secrets/**` | deny / high |
| `deny-key-files` | `*.pem`, `*.key`, `id_rsa*`, `id_ed25519*`, `id_ecdsa*` | deny / critical |
| `deny-ssh-dir` | `.ssh/**` | deny / critical |
| `deny-aws-dir` | `.aws/**` | deny / critical |
| `deny-gnupg-dir` | `.gnupg/**` | deny / critical |
| `deny-wallet-dirs` | Electrum, Monero, generic wallet dirs | deny / critical |
| `deny-credentials-files` | `credentials*`, `.npmrc`, `.netrc`, `.pypirc` | deny / critical |
| `deny-etc-passwd-read` | `/etc/passwd`, `/etc/shadow`, `/etc/sudoers` | deny / high |
| `deny-exec-root-recursive-delete` | `rm -rf /`, `sudo rm … /` | deny / critical |
| `deny-exec-sudo-system-write` | `sudo` writes into `/etc`, `/usr`, `/boot`, Windows system dirs | deny / critical |
| `warn-exec-home-recursive-delete` | recursive deletes targeting home | warn / high |
| `warn-exec-windows-system-delete` | `rd`/`rmdir`/`del` on `C:\Windows` | warn / high |
| `deny-net-metadata-services` | `169.254.169.254`, `metadata.google.internal` | deny / critical |
| `warn-net-curl-pipe-shell` | `curl … \| sh` | warn / high |

Canonical source: [`rules/core.toml`](../rules/core.toml). Remote feed rules arrive via sync and show up in `vigil rules list`.
