# Daemon, IPC & Live Report

Vigil's daemon owns event persistence, retention and a local IPC bus. Hook
execution never waits on it; enforcement remains local and synchronous.

## Run

```sh
vigil daemon
vigil daemon --no-tray
```

## Live report and SIEM handoff

```sh
vigil tui --once
vigil tui
vigil tui --report
```

`--report` prints denied/warned/masked totals, per-agent and per-action counts, and all retained events with rule severity. CSV and JSON export remain available in the TUI.

Health and policy checks:

```sh
vigil doctor
```

## Retention

Events are kept for 30 days and 100,000 records by default. Tune `rules.retention_days` and `rules.max_events` in `config.toml`; limits apply to the active event log and existing size-based rotation remains enabled.

## Updates

```sh
vigil update --check
```
