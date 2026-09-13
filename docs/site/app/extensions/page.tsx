import { DocShell } from "@/components/doc-shell";

export default function Extensions() {
  return (
    <DocShell title="Extensions" active="/extensions">
      <p>
        Extensions are drop-in folders under{" "}
        <code>~/.vigil/extensions/&lt;name&gt;/</code> adding rules, masking
        patterns, and event hooks. Only <code>manifest.toml</code> is
        required.
      </p>

      <h2>Layout</h2>
      <pre>
        <code>{`~/.vigil/extensions/
└── my-team/
    ├── manifest.toml      # required
    ├── rules/*.toml       # same schema as the core ruleset
    ├── patterns/*.toml    # reusable masking pattern families
    └── hooks/*.toml       # event hooks`}</code>
      </pre>

      <h2>Manifest</h2>
      <pre>
        <code>{`[extension]
name = "my-team"       # must match the folder name
version = "0.1.0"
description = "Company-internal guard policy"`}</code>
      </pre>

      <h2>Walkthrough</h2>
      <pre>
        <code>{`mkdir -p ~/.vigil/extensions/my-team/rules

# rules/infra.toml
[[rule]]
id = "deny-tfstate"
description = "Terraform state may contain secrets"
scope = ["read", "write", "search"]
paths = ["**/*.tfstate", "**/*.tfstate.*"]
action = "deny"
severity = "high"

vigil reload
vigil rules list   # deny-tfstate appears in priority order`}</code>
      </pre>

      <h2>Sharing</h2>
      <p>
        An extension folder is self-contained — commit it to a repo; teammates
        install by copying into <code>~/.vigil/extensions/</code>. Rule ids
        from extensions join the same override chain: an extension rule with{" "}
        <code>id = &quot;deny-env-files&quot;</code> overrides the built-in of
        that id. Keep rules narrow and most-specific-first, same as the core
        ruleset.
      </p>
    </DocShell>
  );
}
