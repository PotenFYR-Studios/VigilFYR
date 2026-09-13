import { DocShell } from "@/components/doc-shell";

export default function Daemon() {
  return (
    <DocShell title="Daemon & Live Report" active="/daemon">
      <p>
        The daemon owns event persistence and a local Unix-socket bus. Hook
        execution never waits on it; enforcement remains local and synchronous.
      </p>

      <h2>Run</h2>
      <pre>
        <code>{`vigil daemon
vigil daemon --no-tray`}</code>
      </pre>

      <h2>Live report</h2>
      <p>
        Use a snapshot in scripts, or the interactive TUI when a terminal is
        available.
      </p>
      <pre>
        <code>{`vigil tui --once
vigil tui`}</code>
      </pre>

      <h2>Updates</h2>
      <pre>
        <code>{`vigil update --check`}</code>
      </pre>
      <p>
        Checks use a short timeout and are never part of the enforcement path.
      </p>
    </DocShell>
  );
}
