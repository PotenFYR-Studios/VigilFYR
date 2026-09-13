import Link from "next/link";

import { AuroraText } from "@/components/magicui/aurora-text";
import { DocShell } from "@/components/doc-shell";
import { InstallCommand } from "@/components/install-command";

export default function GettingStarted() {
  return (
    <DocShell title="Getting Started" active="/getting-started">
      <p>
        Install Vigil, run the setup wizard once, and every supported AI coding
        agent on the machine is guarded.
      </p>

      <h2>Install</h2>
      <p>
        The install script detects your OS and architecture, downloads the
        matching release binary, and verifies its checksum. Supported:{" "}
        <strong>Linux, macOS, Windows</strong> on <strong>x86_64 and arm64</strong>.
      </p>
      <InstallCommand />

      <p>Prefer building from source:</p>
      <pre>
        <code>{`git clone https://github.com/PotenFYR-Studios/VigilFYR
cd VigilFYR
cargo install --path .`}</code>
      </pre>

      <h2>
        The <AuroraText>setup wizard</AuroraText>
      </h2>
      <p>
        <code>vigil setup</code> is interactive and covers everything:
      </p>
      <ol>
        <li>
          <strong>OS check</strong> - verifies your platform is supported.
        </li>
        <li>
          <strong>Agent auto-detect</strong> - scans for Claude Code, Codex,
          Gemini CLI, Cursor, OpenCode, and Hermes installs.
        </li>
        <li>
          <strong>Per-agent mode</strong> - <code>enforce</code>,{" "}
          <code>audit</code>, or <code>off</code> per detected agent; enforce is
          the default.
        </li>
        <li>
          <strong>Extra paths</strong> - project-specific or personal paths
          beyond the built-in sensitive areas.
        </li>
        <li>
          <strong>Masking opt-in</strong> - off by default; opt in here or later
          via config.
        </li>
        <li>
          <strong>Autostart + tray</strong> - daemon on login, tray icon with
          update checks.
        </li>
      </ol>
      <p>Zero-prompt variant:</p>
      <pre>
        <code>vigil setup --defaults</code>
      </pre>

      <h2>What runs where</h2>
      <ul>
        <li>
          <code>vigil</code> / <code>vigil tui</code> - live report: verdicts,
          agents, rules, event log, export.
        </li>
        <li>
          <code>vigil daemon</code> - the background guard: audit watcher plus
          rule sync.
        </li>
        <li>
          <code>vigil intercept</code> - the hook entry point agents call
          before each tool action.
        </li>
      </ul>

      <h2>First rules check</h2>
      <pre>
        <code>{`vigil rules list
vigil rules path
vigil reload`}</code>
      </pre>

      <p>
        Next:{" "}
        <Link href="/rules">Rules reference</Link> ·{" "}
        <Link href="/agents">Agents matrix</Link>
      </p>
    </DocShell>
  );
}
