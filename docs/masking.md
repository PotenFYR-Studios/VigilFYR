# Masking

Masking redacts sensitive values out of content before an AI agent sees it, while keeping enough context to work with: each masked value keeps its **first 4 characters**, everything after is replaced.

**Off by default.** Masking is opt-in because it changes what agents see, and changed content can surprise tools that diff or checksum files.

## Enable

```sh
vigil config set masking.enabled true
vigil reload
```

Or accept it in `vigil setup` (masking step of the wizard).

## What gets masked

Thirty-three pattern families ship enabled by default, grouped by provider,
token, URI, assignment, and entropy shape:

| Group | Patterns |
| :--- | :--- |
| Cloud providers | `aws_key`, `aws_session_token`, `azure_storage_key`, `azure_client_secret`, `google_api_key`, `digitalocean_token`, `cloudflare_api_token`, `heroku_api_key` |
| AI services | `openai_key`, `anthropic_key`, `huggingface_token` |
| Developer platforms | `github_pat`, `gitlab_pat`, `private_token`, `npm_token`, `pypi_token`, `rubygems_key` |
| Messaging and billing | `slack_token`, `telegram_bot_token`, `stripe_key`, `sendgrid_key`, `mailgun_key`, `twilio_key`, `line_token`, `plaid_secret` |
| Identity and transport | `jwt`, `private_key`, `connection_uri`, `sensitive_uri`, `sensitive_assignment` |
| Generic secret shapes | `hex_secret`, `base64_secret`, `env_values` |

Example - `AKIAIOSFODNN7EXAMPLE` becomes `AKIA••••••••••••••••` (4-char prefix preserved); the exact replacement form is applied at the content layer before the agent's tool result is returned.

## Scope

Masking is a **verdict action**: rules with `action = "mask"` mark matched paths for redaction. It applies on `read`/`search` results flowing to the agent. Writes are unaffected - masking protects what leaves your disk toward the model, not what gets written to it.

Write your own masking rule:

```toml
[[rule]]
id = "mask-stripe-keys"
description = "Mask Stripe keys in reads"
scope = ["read", "search"]
paths = ["**/config/**", "**/.env*"]
action = "mask"
severity = "high"
```

Canonical pattern source: [`rules/patterns.toml`](../rules/patterns.toml).
Custom pattern families are added via [Extensions](extensions.md)
(`patterns/*.toml` next to your `manifest.toml`).

## Interplay with other rules

First match wins. If a `deny` rule matches first, the event never reaches the masking stage. Put `mask` rules **before** broader `deny` rules when you want redaction instead of a hard block for a subset of files:

```toml
# ~/.vigil/rules/mask-first.toml
[[rule]]
id = "mask-env-values"
scope = ["read"]
paths = ["**/.env"]
action = "mask"
severity = "high"

# the built-in deny-env-files (also matching **/.env) now only fires
# for write/search - reads of .env get masked values instead of a block
```

Check current state:

```sh
vigil config get masking.enabled
vigil config get masking.patterns
```
