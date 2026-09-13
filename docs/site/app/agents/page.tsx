import { DocShell } from "@/components/doc-shell";

const agents = [
  "Claude Code",
  "Codex",
  "Gemini CLI",
  "Cursor",
  "OpenCode",
  "Hermes",
];

export default function Agents() {
  return (
    <DocShell title="Agents Matrix" active="/agents">
      <p>
        Vigil guards each agent at the highest level it supports, with two
        universal fallbacks. <code>vigil setup</code> auto-detects installed
        agents and picks the mode.
      </p>

      <table className="mt-4 w-full text-sm">
        <thead>
          <tr className="text-left text-zinc-400">
            <th className="py-2">Agent</th>
            <th className="py-2">Hooks (enforce)</th>
            <th className="py-2">Audit</th>
            <th className="py-2">Shim</th>
          </tr>
        </thead>
        <tbody>
          {agents.map((a) => (
            <tr key={a} className="border-t border-zinc-800">
              <td className="py-2">{a}</td>
              <td className="py-2">yes (auto-installed)</td>
              <td className="py-2">yes</td>
              <td className="py-2">yes</td>
            </tr>
          ))}
          <tr className="border-t border-zinc-800">
            <td className="py-2">anything else</td>
            <td className="py-2">-</td>
            <td className="py-2">yes</td>
            <td className="py-2">yes</td>
          </tr>
        </tbody>
      </table>

      <h2>The three layers</h2>
      <ul className="mt-4 space-y-3 text-sm">
        <li>
          <strong className="text-zinc-100">Hooks</strong> - for hook-capable
          agents, installed automatically by <code>vigil setup</code>. Every
          tool call (read, write, search, exec, net) is evaluated before it
          runs. Exit codes: <code>0</code> allow, <code>2</code> deny,{" "}
          <code>3</code> ask. The only layer that blocks.
        </li>
        <li>
          <strong className="text-zinc-100">Audit</strong> - filesystem
          watcher; records sensitive-area access by any process and warns.
          Never blocks: every <code>deny</code> demotes to <code>warn</code>.
        </li>
        <li>
          <strong className="text-zinc-100">Shim</strong> -{" "}
          <code>vigil shim &lt;command&gt;</code> wraps arbitrary commands with
          the same rule evaluation, for agents without hook support.
        </li>
      </ul>

      <h2>Per-agent modes</h2>
      <pre>
        <code>{`vigil config get agents.codex
vigil config set agents.codex enforce
vigil config set agents.gemini-cli audit
vigil config set agents.cursor off
vigil reload`}</code>
      </pre>
      <p>
        Modes: <code>enforce</code> (hooks active, denies block),{" "}
        <code>audit</code> (record + warn only), <code>off</code> (no hooks;
        audit watcher still sees filesystem activity).
      </p>

      <h2>Proxy-proofing</h2>
      <p>
        Vigil evaluates local tool actions, not LLM network traffic. Which
        API, proxy, gateway, or router serves the model (9router, LiteLLM, a
        corporate endpoint) has zero effect on verdicts.
      </p>

      <h2>Honest limits</h2>
      <p>
        Hooks cover hookable agents; audit and shim provide visibility and
        wrapping, not interception. Full discussion:{" "}
        <a href="/security-model">Security model &amp; limitations</a>.
      </p>
    </DocShell>
  );
}
