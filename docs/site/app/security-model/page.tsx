import { DocShell } from "@/components/doc-shell";

const layers: [string, string, string][] = [
  ["Hooks", "blocks", "Hook-capable agents (Claude Code, Codex, Gemini CLI, Cursor, OpenCode, Hermes)"],
  ["Shim", "wraps", "Any command explicitly routed through vigil shim"],
  ["Audit", "warns only", "Any process touching watched areas — detection, not prevention"],
];

export default function SecurityModel() {
  return (
    <DocShell title="Security Model & Limitations" active="/security-model">
      <p>
        Vigil is a policy gate between an AI agent&apos;s tools and your
        filesystem. Every tool action — read, write, search, exec, net — is
        evaluated before (hooks), around (shim), or after the fact (audit).
      </p>

      <h2>Layer coverage</h2>
      <ul className="mt-4 space-y-2 text-sm">
        {layers.map(([l, cov, who]) => (
          <li key={l}>
            <strong className="text-zinc-100">{l}</strong> — {cov} — {who}.
          </li>
        ))}
      </ul>
      <p>
        Hooks cover hookable agents; audit and shim are for the rest. Nothing
        blocks an arbitrary, unhooked, unshimmed process — that is what OS
        permissions are for. Vigil complements them.
      </p>

      <h2>Fail-open, deliberately</h2>
      <p>
        On daemon error, ruleset load error, or hook malfunction, Vigil
        <strong> allows the action</strong> (exit code <code>0</code>). A guard
        sitting in every tool call&apos;s critical path must not be able to
        take your workflow down, and the threat model — an AI agent fumbling
        toward your secrets — is not a determined attacker who could be slowed
        by fail-closed behavior. Audit logging and TUI daemon health surface
        transient failures.
      </p>

      <h2>Verdict contract</h2>
      <pre>
        <code>{`allow  -> 0      permit (explicit or default)
deny   -> 2      blocked before execution
ask    -> 3      deferred for interactive confirmation
warn   -> 0      permitted, recorded, flagged
mask   -> 0      permitted, content redacted
audit mode      every deny demoted to warn`}</code>
      </pre>

      <h2>Known limitations</h2>
      <ul className="mt-4 list-disc space-y-2 pl-6 text-sm">
        <li>
          Rule matching is pattern-based; exotic path encodings or command
          obfuscation can evade globs/regexes. Accepted trade-offs are
          documented in <code>rules/core.toml</code>.
        </li>
        <li>
          Masking is best-effort content filtering, not a DLP product; novel
          secret formats pass through unless you add patterns.
        </li>
        <li>
          <code>net</code> scope works at the command-line level (metadata
          hosts, curl-pipe-shell), not as a firewall.
        </li>
        <li>
          Agents without hooks get audit + shim: record and wrap, no
          mid-flight blocking.
        </li>
        <li>
          Local trust boundary: Vigil runs as your user and does not defend
          against malware running as you.
        </li>
      </ul>
    </DocShell>
  );
}
