import { DocShell } from "@/components/doc-shell";

const patterns = [
  ["aws_key", "AWS access key IDs and secret keys (AKIA…)"],
  ["github_pat", "GitHub personal access tokens (ghp_, github_pat_, …)"],
  ["openai_key", "OpenAI API keys (sk-…)"],
  ["jwt", "JSON Web Tokens (three base64url segments)"],
  ["private_key", "PEM private key blocks"],
  ["env_values", "Values assigned in env-style files (SECRET=…)"],
];

export default function Masking() {
  return (
    <DocShell title="Masking" active="/masking">
      <p>
        Masking redacts sensitive values before an agent sees them, keeping the{" "}
        <strong>first 4 characters</strong> so you can still tell which secret
        it was. <strong>Off by default</strong> — opt in because it changes
        what agents see.
      </p>

      <h2>Enable</h2>
      <pre>
        <code>{`vigil config set masking.enabled true
vigil reload`}</code>
      </pre>

      <h2>Pattern families</h2>
      <ul className="mt-4 space-y-2">
        {patterns.map(([id, what]) => (
          <li key={id} className="text-sm">
            <code className="text-sky-300">{id}</code>
            <span className="text-zinc-400"> — {what}</span>
          </li>
        ))}
      </ul>

      <h2>Scope and interplay</h2>
      <p>
        Masking is a verdict action: rules with <code>action = &quot;mask&quot;</code>{" "}
        mark matched paths for redaction on <code>read</code>/<code>search</code>{" "}
        results flowing to the agent. Writes are unaffected. First match wins —
        put <code>mask</code> rules before broader <code>deny</code> rules to
        redact instead of block:
      </p>
      <pre>
        <code>{`[[rule]]
id = "mask-env-values"
scope = ["read"]
paths = ["**/.env"]
action = "mask"
severity = "high"`}</code>
      </pre>
      <p>
        Custom pattern families come from{" "}
        <a href="/extensions">extensions</a> (<code>patterns/*.toml</code> next
        to your manifest).
      </p>
    </DocShell>
  );
}
