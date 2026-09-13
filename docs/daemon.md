# Daemon, IPC & Live Report

Vigil's daemon owns event persistence and a local Unix-socket bus. Hook
execution never waits on it; enforcement remains local and synchronous.

## Run

```sh
vigil daemon
vigil daemon --no-tray
```

## Live report

```sh
vigil tui --once
vigil tui
```

## Updates

```sh
vigil update --check
```
