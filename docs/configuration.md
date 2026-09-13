# Configuration Reference

Config lives at `~/.vigil/config.toml`. **A missing file means defaults** - Vigil runs fine with no config at all. Values below are the defaults.

```toml
[general]
enabled = true          # master switch
mode = "enforce"        # "enforce" | "audit"

[daemon]
autostart = true        # start the daemon on login
tray = true             # tray icon; update checks surface here

[masking]
enabled = false         # opt-in sensitive-data masking
patterns = ["aws_key", "github_pat", "openai_key", "jwt", "private_key", "env_values"]

[rules]
remote_update = true    # sync remote rules on boot / vigil reload

[agents]
claude-code = "enforce" # per-agent: "off" | "audit" | "enforce"
```

## CLI access

Read and write without editing the file:

```sh
vigil config get <key>          # value as display string
vigil config set <key> <value>
```

Examples:

```sh
vigil config get general.mode                  # enforce
vigil config set general.mode audit            # calibrate without blocking
vigil config set masking.enabled true
vigil config set rules.remote_update false
vigil config set agents.codex enforce          # unknown agent ids are created
```

Unknown keys error with `unknown config key: <key>`; invalid values error with `invalid value for <key>: <value>`.

## Key reference

| Key | Type | Default | Notes |
| :--- | :--- | :--- | :--- |
| `general.enabled` | bool | `true` | Master switch; `false` disables the guard |
| `general.mode` | `enforce` \| `audit` | `enforce` | `audit` demotes every `deny` to `warn` - nothing blocks |
| `daemon.autostart` | bool | `true` | Daemon starts on login |
| `daemon.tray` | bool | `true` | Tray icon; `vigil update` availability shows here |
| `masking.enabled` | bool | `false` | Opt-in; see [Masking](masking.md) |
| `masking.patterns` | list | all six | Subset to enable fewer families |
| `rules.remote_update` | bool | `true` | Remote rule sync on boot and `vigil reload` |
| `agents.<id>` | `off` \| `audit` \| `enforce` | `claude-code = "enforce"` | Per-agent level; any id accepted |

Agent ids match the supported agents: `claude-code`, `codex`, `gemini-cli`, `cursor`, `opencode`, `hermes`.

## File locations

| Path | Purpose |
| :--- | :--- |
| `~/.vigil/config.toml` | Config |
| `~/.vigil/rules/*.toml` | User rules (override built-ins by id) |
| `./.vigil/rules/*.toml` | Project rules (highest precedence by id) |
| `~/.vigil/extensions/<name>/` | Extensions ([authoring guide](extensions.md)) |
| `~/.vigil/` | Event log / daemon state |

Apply changes without restarting the daemon:

```sh
vigil reload
```

## Related

- [Getting Started](getting-started.md) - install and the setup wizard
- [Rules Reference](rules.md) - rule schema and precedence
- [Security model](security-model.md) - fail-open behavior and its trade-offs
