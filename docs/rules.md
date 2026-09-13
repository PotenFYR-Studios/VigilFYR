# Rules Reference

Rules are TOML files. Vigil loads, in order:

1. **Built-in core ruleset** - embedded in the binary (`rules/core.toml` at build time), currently 53 policy rules.
2. **User rules** - `~/.vigil/rules/*.toml`
3. **Project rules** - `./.vigil/rules/*.toml` (highest precedence for shared, per-project policy)
4. **Remote feed** - community rules synced on boot and `vigil reload` (disable with `rules.remote_update = false`)

Later sources override earlier ones **by rule id**: a rule in `~/.vigil/rules/` with `id = "deny-env-files"` replaces the built-in rule of the same id. Within a single file, order is priority - **first matching rule wins**, so keep the most specific rules first. No match means **allow** (fail-open).

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

- **`scope`** - event actions: `read`, `write`, `search`, `exec`, `net`.
- **`paths`** - globset globs. `**` spans directories. A rule matches when *any* pattern matches *any* path on the event.
- **`commands`** - Rust `regex` crate syntax, matched against the full command line of `exec`/`net` events. A rule matches when *any* command regex matches.
- A rule fires if **either** a path hits **or** a command hits (both checked; at least one must match).
- **`agents`** - restricts a rule to specific agents. Empty list = every agent.

## Actions and exit codes

| `action` | Effect | Hook exit code |
| :--- | :--- | :---: |
| `deny` | Block the tool call | `2` |
| `warn` | Allow, but record and flag in the TUI | `0` |
| `allow` | Explicitly allow (escape hatch past earlier broad rules) | `0` |
| `mask` | Redact sensitive content before the agent sees it (requires masking enabled) | `0` |
| - no match | Default: allow, logged as rule `default` | `0` |
| *audit mode* | Every `deny` is demoted to `warn` - nothing is blocked | `0` |

Exit code `3` (**ask**) is reserved for interactive confirmation by the calling agent. On daemon or ruleset error the hook **fails open** (allow) - a broken guard must never break your agent.

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

`allow-internal-templates` must come **first** - first match wins.

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

Audit-mode canary - watch, don't block, while calibrating:

```toml
[[rule]]
id = "warn-build-artifacts"
description = "Notice when agents wander into target/"
scope = ["read", "search"]
paths = ["**/target/**", "**/node_modules/**"]
action = "warn"
severity = "low"
```

Or flip the whole daemon to audit: `vigil config set general.mode audit` (then `vigil reload`). Demotion applies per verdict, so mixing `warn` and `deny` rules is fine - in audit mode everything behaves as `warn`.

Disable a built-in rule without deleting it:

```toml
# ~/.vigil/rules/overrides.toml
[[rule]]
id = "warn-net-curl-pipe-shell"
enabled = false
```

(Keep the required fields - `scope`, `paths` - to satisfy the schema.)

## Built-in policy coverage

The built-in ruleset covers credential stores, CI/CD and deployment material,
package-manager registries, application configuration, containers and cluster
manifests, password managers, shell history, agent state, destructive system
commands, privilege escalation, persistence, network configuration, database
mutations, process and socket discovery, exfiltration channels, and internal
network access. Repository metadata emits a warning so agent traversal remains
visible without blocking normal Git workflows.

Representative groups:

| Coverage | Examples |
| :--- | :--- |
| Identity and secrets | `.ssh`, `.aws`, `.gnupg`, private keys, password databases, browser stores |
| Cloud and CI/CD | GitHub workflows, CircleCI, GitLab CI, Vault tokens, Terraform state, Vercel, Firebase |
| Package and service credentials | Cargo, Docker, Gradle, Composer, NuGet, `.npmrc`, `.netrc`, `.pypirc` |
| Application and infrastructure | `application*.yml`, `appsettings*.json`, Docker Compose, Kubernetes manifests |
| Execution attacks | exfiltration, secret hunting, recon, cracking, privilege escalation, persistence, database mutation, privileged containers |
| Network egress | metadata services, private networks, webhooks, paste and transfer services, plaintext HTTP |
| Masking entry points | JSON, YAML, TOML, INI, Python, JavaScript and TypeScript source reads |

| Rule id | Covers | Action / severity |
| :--- | :--- | :--- |
| `deny-env-files` | `.env`, `.env.*` | deny / high |
| `deny-ci-cd-deployment-credentials` | workflows, Vault, Terraform, hosting credentials | deny / critical |
| `deny-package-manager-credentials` | Cargo, Docker, Gradle, Composer, NuGet | deny / critical |
| `deny-message-queue-and-service-configs` | application YAML, appsettings, Django and WordPress settings | deny / high |
| `deny-container-and-kubernetes-secrets` | Compose, Kubernetes, Dockerfiles | deny / high |
| `deny-user-identity-and-shell-config-read` | shell and database command history | deny / high |
| `deny-password-databases` | 1Password, Bitwarden, KeePass, keychains | deny / critical |
| `deny-agent-state-and-mcp-config` | Claude, Codex, Cursor, Continue, Aider state | deny / high |
| `warn-source-control-metadata` | Git, Mercurial and SVN metadata | warn / medium |
| `deny-exec-remote-file-exfiltration` | uploads and Git/cloud pushes of secret material | deny / critical |
| `deny-exec-secret-search-and-archive` | recursive secret search and secret archives | deny / critical |
| `deny-exec-process-and-network-recon` | process, socket, interface and port discovery | deny / high |
| `deny-exec-password-and-hash-tools` | cracking and hash extraction tools | deny / critical |
| `deny-exec-privilege-escalation` | sudo shells, user creation, system permission changes | deny / critical |
| `deny-exec-scheduler-and-service-persistence` | cron, systemd, launchd, Windows scheduled tasks and registry | deny / critical |
| `deny-exec-network-configuration-and-vpn` | firewall, resolver and tunnel changes | deny / critical |
| `deny-exec-clipboard-and-screen-capture` | clipboard and screen capture | deny / high |
| `deny-exec-database-destructive-commands` | destructive SQL and interactive database commands | deny / critical |
| `deny-exec-container-control-and-mount` | privileged containers, mounts and kernel modules | deny / critical |
| `deny-exec-source-overwrite-system-shell` | shell startup files and global package installs | deny / critical |
| `deny-exec-agent-hook-and-config-mutation` | command-line mutation of agent settings and hooks | deny / critical |
| `warn-exec-package-install` | dependency installation | warn / medium |
| `deny-net-private-and-loopback-targets` | loopback, RFC1918 and cluster-internal targets | deny / critical |
| `deny-net-webhook-and-pastebin-exfil` | webhook, paste and transfer endpoints | deny / critical |
| `warn-net-public-ai-and-package-endpoints` | AI and registry endpoints | warn / low |
| `warn-net-non-https-url` | plaintext HTTP | warn / medium |
| `mask-source-config-secrets` | common source and config formats | mask / high |
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

Canonical source: [`rules/core.toml`](../rules/core.toml). Run `vigil rules list`
for the exact merged policy. Remote feed rules arrive via sync and appear in the
same list.
