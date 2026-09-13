import { DocShell } from "@/components/doc-shell";

const actions = [
  ["deny", "Block the tool call", "2"],
  ["warn", "Allow, record, flag in the TUI", "0"],
  ["allow", "Explicit allow (escape hatch past broad rules)", "0"],
  ["mask", "Redact sensitive content (masking enabled)", "0"],
  ["no match", "Default allow, logged as rule default", "0"],
  ["audit mode", "Every deny demoted to warn", "0"],
];

export default function Rules() {
  return (
    <DocShell title="Rules Reference" active="/rules">
      <p>
        Rules are TOML. Load order: built-in core ruleset →{" "}
        <code>~/.vigil/rules/*.toml</code> → <code>./.vigil/rules/*.toml</code>{" "}
        → remote feed. Later sources override earlier ones{" "}
        <strong>by rule id</strong>. Within a file, order is priority:{" "}
        <strong>first match wins</strong>. No match = allow (fail-open).
      </p>

      <h2>Schema</h2>
      <pre>
        <code>{`[[rule]]
id = "deny-my-secrets"              # required, unique; reuse a built-in id to override
description = "Block the company secrets dir"
scope = ["read", "write", "search"] # read | write | search | exec | net
paths = ["**/company-secrets/**"]   # globset globs
commands = []                       # regex patterns for exec/net command lines
agents = []                         # empty = all agents
action = "deny"                     # deny | allow | warn | mask
severity = "critical"               # low | medium | high | critical
enabled = true`}</code>
      </pre>
      <p>
        A rule fires if either a path glob or a command regex matches. Rules
        only apply to events whose action is in <code>scope</code>.
      </p>

      <h2>Verdict actions and exit codes</h2>
      <table className="mt-4 w-full text-sm">
        <thead>
          <tr className="text-left text-zinc-400">
            <th className="py-2">action</th>
            <th className="py-2">Effect</th>
            <th className="py-2">Hook exit</th>
          </tr>
        </thead>
        <tbody>
          {actions.map(([a, eff, code]) => (
            <tr key={a} className="border-t border-zinc-800">
              <td className="py-2 font-mono text-sky-300">{a}</td>
              <td className="py-2">{eff}</td>
              <td className="py-2 font-mono">{code}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <p>
        Exit code <code>3</code> (<code>ask</code>) defers to the calling agent
        for interactive confirmation. On daemon/ruleset error the hook fails
        open (allow).
      </p>

      <h2>Cookbook</h2>
      <p>Allow narrow before denying broad — first match wins:</p>
      <pre>
        <code>{`[[rule]]
id = "allow-internal-templates"
scope = ["read", "search"]
paths = ["**/internal/templates/**"]
action = "allow"
severity = "low"

[[rule]]
id = "deny-internal-docs"
scope = ["read", "search", "write"]
paths = ["**/internal/**"]
action = "deny"
severity = "high"`}</code>
      </pre>
      <p>Command-line regex on exec events:</p>
      <pre>
        <code>{`[[rule]]
id = "deny-git-force-push"
scope = ["exec"]
commands = ["git\\\\s+push\\\\s+.*(--force|-f)\\\\b"]
action = "deny"
severity = "high"`}</code>
      </pre>
      <p>Disable a built-in by id without deleting it:</p>
      <pre>
        <code>{`[[rule]]
id = "warn-net-curl-pipe-shell"
enabled = false`}</code>
      </pre>

      <h2>Built-in rules</h2>
      <p>
        Fifteen rules cover env files, secrets dirs, key files,{" "}
        <code>.ssh</code>, <code>.aws</code>, <code>.gnupg</code>, wallet dirs,
        credentials files, system account files, catastrophic exec (
        <code>rm -rf /</code>, sudo system writes), and net guards (cloud
        metadata, curl-pipe-shell). Full table with ids and severities:{" "}
        <a href="https://github.com/PotenFYR-Studios/VigilFYR/blob/master/docs/rules.md">
          docs/rules.md
        </a>{" "}
        and the canonical source{" "}
        <a href="https://github.com/PotenFYR-Studios/VigilFYR/blob/master/rules/core.toml">
          rules/core.toml
        </a>
        .
      </p>
    </DocShell>
  );
}
