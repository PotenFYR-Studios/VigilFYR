import { DocShell } from "@/components/doc-shell";

const keys: [string, string, string][] = [
  ["general.enabled", "true", "Master switch"],
  ["general.mode", "enforce", "enforce | audit (audit never blocks)"],
  ["daemon.autostart", "true", "Daemon starts on login"],
  ["daemon.tray", "true", "Tray icon; update checks surface here"],
  ["masking.enabled", "false", "Opt-in sensitive-data masking"],
  [
    "masking.patterns",
    "all six",
    "aws_key, github_pat, openai_key, jwt, private_key, env_values",
  ],
  ["rules.remote_update", "true", "Remote rule sync on boot / vigil reload"],
  ["agents.<id>", "claude-code=enforce", "off | audit | enforce per agent"],
];

export default function Configuration() {
  return (
    <DocShell title="Configuration Reference" active="/configuration">
      <p>
        Config lives at <code>~/.vigil/config.toml</code>. A missing file means
        defaults — Vigil runs fine with no config.
      </p>
      <pre>
        <code>{`[general]
enabled = true
mode = "enforce"

[daemon]
autostart = true
tray = true

[masking]
enabled = false
patterns = ["aws_key", "github_pat", "openai_key", "jwt", "private_key", "env_values"]

[rules]
remote_update = true

[agents]
claude-code = "enforce"`}</code>
      </pre>

      <h2>CLI access</h2>
      <pre>
        <code>{`vigil config get general.mode               # enforce
vigil config set general.mode audit         # calibrate without blocking
vigil config set masking.enabled true
vigil config set agents.codex enforce       # unknown agent ids are created
vigil reload`}</code>
      </pre>
      <p>
        Unknown keys error with <code>unknown config key: &lt;key&gt;</code>;
        invalid values with <code>invalid value for &lt;key&gt;: &lt;value&gt;</code>.
      </p>

      <h2>Key reference</h2>
      <table className="mt-4 w-full text-sm">
        <thead>
          <tr className="text-left text-zinc-400">
            <th className="py-2">Key</th>
            <th className="py-2">Default</th>
            <th className="py-2">Notes</th>
          </tr>
        </thead>
        <tbody>
          {keys.map(([k, d, n]) => (
            <tr key={k} className="border-t border-zinc-800">
              <td className="py-2 font-mono text-sky-300">{k}</td>
              <td className="py-2 font-mono">{d}</td>
              <td className="py-2">{n}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <h2>File locations</h2>
      <pre>
        <code>{`~/.vigil/config.toml              # config
~/.vigil/rules/*.toml             # user rules (override built-ins by id)
./.vigil/rules/*.toml             # project rules (highest precedence by id)
~/.vigil/extensions/<name>/       # extensions
~/.vigil/                         # event log / daemon state`}</code>
      </pre>
      <p>
        Apply changes without a restart: <code>vigil reload</code>.
      </p>
    </DocShell>
  );
}
