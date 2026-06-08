# WinPortKill

English | [中文](./README.zh.md)

WinPortKill is a Windows x64-first port and process inspection toolset built around a shared Rust core.

The repository currently contains:

- a terminal UI binary
- an HTTP sidecar mode for IDE integrations
- an `egui` desktop GUI prototype
- a VS Code extension
- a JetBrains plugin MVP

## Current Scope

- Platform: Windows x64 for the actual port/process inspection flow
- Core capabilities: list listening ports, aggregate by process, filter results, kill a PID
- Shared backend: Rust crates under [crates](./crates)
- IDE integrations: VS Code webview extension and JetBrains Tool Window

## Workspace Layout

- [Cargo.toml](./Cargo.toml): workspace root and top-level TUI binary
- [src](./src): terminal UI entry and TUI state/rendering
- [crates/winportkill-core](./crates/winportkill-core): Windows port scan, process aggregation, filtering, kill logic
- [crates/winportkill-server](./crates/winportkill-server): Axum HTTP API and WebSocket stream
- [crates/winportkill-gui](./crates/winportkill-gui): native `egui` desktop GUI
- [.vscode-extension](./.vscode-extension): VS Code extension and webview UI
- [jetbrains-plugin](./jetbrains-plugin): JetBrains plugin MVP
- [docs](./docs): bilingual project and implementation guides

## Quick Start

```powershell
cargo build
cargo run -p winportkill
```

Other common entrypoints:

```powershell
cargo run -p winportkill -- --json
cargo run -p winportkill -- --serve 3000
cargo run -p winportkill-gui
cargo run -p winportkill -- list --json
cargo run -p winportkill -- who-uses 3000
cargo run -p winportkill -- kill --port 3000
```

## Runtime Modes

### Terminal UI

The root package `winportkill` launches a `ratatui` interface by default.

- refreshes data every 10 seconds
- supports filtering
- can switch between ports view and processes view
- can kill the selected PID

### JSON mode

`--json` prints one snapshot and exits.

### Scriptable CLI mode

The root package also exposes script-friendly subcommands:

- `list [--json]`
- `who-uses <port> [--json]`
- `kill --pid <pid> [--json]`
- `kill --port <port> [--json]`

### Server mode

`--serve <port>` starts a localhost HTTP service for IDE integrations.

### Native GUI

The `winportkill-gui` crate is an `eframe`/`egui` frontend over `winportkill-core`.

## IDE Integrations

### VS Code extension

The VS Code extension lives under [.vscode-extension](./.vscode-extension).

- runs as a webview sidebar
- starts the bundled Rust sidecar binary from `.vscode-extension/bin/...`
- talks to the sidecar over localhost HTTP

### JetBrains plugin

The JetBrains plugin lives under [jetbrains-plugin](./jetbrains-plugin).

- provides a Tool Window UI
- starts the same sidecar binary shape with `winportkill.exe --serve <port>`
- uses a Kotlin HTTP client against the local sidecar

## Releases

Tag-based releases publish Windows x64 artifacts to GitHub Releases:

- `winportkill-windows-x64.exe`
- `winportkill-gui-windows-x64.exe`
- `@noxlightman/winportkill` npm tarball

## Documentation Index

- [Project Architecture](./docs/project-architecture.en.md) | [项目架构](./docs/project-architecture.zh.md)
- [Server Mode Guide](./docs/server-mode-guide.en.md) | [Server 模式指南](./docs/server-mode-guide.zh.md)
- [VS Code Extension Guide](./docs/vscode-extension-guide.en.md) | [VS Code 扩展指南](./docs/vscode-extension-guide.zh.md)
- [JetBrains Plugin Guide](./docs/jetbrains-plugin-guide.en.md) | [JetBrains 插件指南](./docs/jetbrains-plugin-guide.zh.md)
- [egui GUI Guide](./docs/egui-gui-guide.en.md) | [egui GUI 指南](./docs/egui-gui-guide.zh.md)

## Known Boundaries

- Actual inspection and kill flow is currently Windows x64-only.
- Killing protected processes may require elevated privileges.
- The VS Code extension and JetBrains plugin are IDE-specific shells around the same localhost sidecar model.
- The native GUI and TUI call `winportkill-core` directly instead of going through HTTP.
