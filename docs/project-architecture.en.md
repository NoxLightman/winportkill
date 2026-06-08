# Project Architecture

English | [中文](./project-architecture.zh.md)

Back: [README](../README.md)

## Overview

WinPortKill is a shared Windows inspection backend plus multiple frontends:

- Rust TUI in the workspace root
- Rust HTTP sidecar in `crates/winportkill-server`
- Rust native GUI in `crates/winportkill-gui`
- VS Code extension in `.vscode-extension`
- JetBrains plugin in `jetbrains-plugin`

The core idea is:

- keep Windows-specific logic in one Rust crate
- call it directly from Rust frontends
- expose it through localhost HTTP for IDE frontends

## Component Map

### `winportkill-core`

- scans listening TCP and UDP bindings from Windows APIs
- joins port rows with process name and memory via `sysinfo`
- exposes port-oriented and process-oriented view models
- computes stats
- terminates a process by PID

### `winportkill-server`

- wraps `winportkill-core` with Axum routes
- provides `/health`, `/ports`, `/processes`, stats endpoints, `/kill/{pid}`, and `/ws`
- is the transport used by IDE integrations

### Root package `winportkill`

- parses CLI flags
- runs one of three modes:
  - default TUI
  - `--json`
  - `--serve <port>`

### `winportkill-gui`

- native `eframe` / `egui` app
- calls `winportkill-core` directly
- keeps local refresh, filter, selection, and kill confirmation state

### `.vscode-extension`

- TypeScript extension host
- webview sidebar UI
- starts a bundled sidecar binary through Node `child_process.spawn`

### `jetbrains-plugin`

- IntelliJ Platform plugin
- Swing Tool Window UI
- Kotlin HTTP client and project-scoped sidecar manager

## Data Flows

### Direct-core flow

Used by the TUI and native GUI:

1. UI triggers refresh or kill.
2. `winportkill-core` scans ports and processes.
3. The frontend filters and renders the current view.
4. Optional kill requests call the same core crate directly.

### Sidecar flow

Used by VS Code and JetBrains:

1. The integration allocates a free localhost port.
2. It starts `winportkill.exe --serve <port>`.
3. It waits for `/health` to return `ok`.
4. The frontend requests `/ports` or `/processes`.
5. Kill actions call `POST /kill/{pid}` and then refresh the current view.

## Shared Sidecar Strategy

The current repository workflow shares one practical binary source for IDE work:

- `.vscode-extension/bin/win32-x64/winportkill.exe`

That matters because:

- the VS Code extension ships from that location
- JetBrains `runIde` development reuses that location
- JetBrains `processResources` copies from that location when available

## Operational Boundaries

- The inspection backend is currently Windows x64-only in practice.
- Kill semantics are process-wide, not “release only one port”.
- Protected targets may require elevation.
- The WebSocket stream currently publishes port snapshots only.

## Recommended Reading

- [README](../README.md)
- [Server Mode Guide](./server-mode-guide.en.md)
- [VS Code Extension Guide](./vscode-extension-guide.en.md)
- [JetBrains Plugin Guide](./jetbrains-plugin-guide.en.md)
