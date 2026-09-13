# Extensions

Extensions are drop-in folders that add rules, masking patterns, and event hooks without touching the core config. Vigil discovers them in `~/.vigil/extensions/<name>/`.

## Layout

```
~/.vigil/extensions/
└── my-team/
    ├── manifest.toml      # required
    ├── rules/
    │   └── internal.toml  # rule files, same schema as the core ruleset
    ├── patterns/
    │   └── tokens.toml    # reusable masking pattern families
    └── hooks/
        └── audit.toml     # event hooks
```

Only `manifest.toml` is required; any of the three subdirectories is optional.

## The manifest

```toml
[extension]
name = "my-team"                 # must match the folder name
version = "0.1.0"
description = "Company-internal guard policy"
```

## What each part does

| Directory | Contents | Effect |
| :--- | :--- | :--- |
| `rules/` | `[[rule]]` TOML files ([schema](rules.md#schema)) | Added to the ruleset like user rules |
| `patterns/` | Masking pattern families | Usable in `masking.patterns` and `mask` rules |
| `hooks/` | Event hook definitions | Run on rule verdicts / lifecycle events |

Rule ids from extensions participate in the same override chain: an extension rule with `id = "deny-env-files"` overrides the built-in of that id.

## Authoring walkthrough

1. Create the folder:

   ```sh
   mkdir -p ~/.vigil/extensions/my-team/rules
   ```

2. Write `manifest.toml`:

   ```toml
   [extension]
   name = "my-team"
   version = "0.1.0"
   description = "Block agent access to infra state"
   ```

3. Add `rules/infra.toml`:

   ```toml
   [[rule]]
   id = "deny-tfstate"
   description = "Terraform state may contain secrets"
   scope = ["read", "write", "search"]
   paths = ["**/*.tfstate", "**/*.tfstate.*"]
   action = "deny"
   severity = "high"
   ```

4. Load and verify:

   ```sh
   vigil reload
   vigil rules list    # deny-tfstate appears in priority order
   ```

## Sharing extensions

Share a Git repository containing the same folder layout. Install it with:

```sh
vigil extension install https://github.com/your-org/vigil-extension
vigil extension list
vigil extension remove your-org-vigil-extension
```

Extensions run hook scripts with your user privileges. Install only extensions
you trust; hooks have a 500 ms timeout and fail open.
An extension folder is self-contained — commit it to a repo, and teammates install it by copying into `~/.vigil/extensions/`. Version bumps in `manifest.toml` make updates visible in `vigil rules list` output and event reasons.

Keep extension rules narrow (specific paths, scoped `agents`) and ordered most-specific-first, same as the core ruleset.
