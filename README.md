# WinPortKill

English | [中文](./README.zh.md)

WinPortKill is a Windows x64 port and process inspection toolset built around a shared Rust core.

It currently ships in four user-facing forms:

- npm CLI: [`@noxlightman/winportkill`](https://www.npmjs.com/package/@noxlightman/winportkill)
- VS Code extension: Visual Studio Marketplace
- JetBrains plugin: JetBrains Marketplace review/upload flow
- native binaries: GitHub Releases

## Install

### npm CLI

```powershell
npm install -g @noxlightman/winportkill
```

Common commands:

```powershell
winportkill list --json
winportkill who-uses 3000
winportkill kill --port 3000
winportkill --serve 3000
```

### VS Code

- Install from the Visual Studio Marketplace
- Or use the packaged `.vsix` from GitHub Releases

The extension downloads and caches the Windows x64 sidecar on first use.

### JetBrains IDEs

- Install from JetBrains Marketplace when available
- Or upload / install the plugin zip produced from `jetbrains-plugin/build/distributions`

### Native binaries

GitHub Releases publish Windows x64 artifacts such as:

- `winportkill-windows-x64.exe`
- `winportkill-gui-windows-x64.exe`
- `winportkill-vscode-<version>.vsix`
- `winportkill-jetbrains-plugin-<version>.zip`

## What It Does

- lists listening ports
- aggregates ports by process
- filters by PID, process name, protocol, address, or port
- kills a PID
- exposes a localhost HTTP sidecar for IDE integrations

## Runtime Modes

### Terminal UI

The root package `winportkill` launches a `ratatui` interface by default.

- refreshes data every 10 seconds
- supports filtering
- switches between ports and processes views
- kills the selected PID

### JSON mode

`--json` prints one snapshot and exits.

### Scriptable CLI mode

- `list [--json]`
- `who-uses <port> [--json]`
- `kill --pid <pid> [--json]`
- `kill --port <port> [--json]`

### Server mode

`--serve <port>` starts a localhost HTTP service for IDE integrations.

### Native GUI

`winportkill-gui` is an `eframe` / `egui` frontend over `winportkill-core`.

## Local Development

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

## Repository Layout

- [Cargo.toml](./Cargo.toml): workspace root and top-level TUI binary
- [src](./src): terminal UI entry and rendering
- [crates/winportkill-core](./crates/winportkill-core): Windows scan, aggregation, filtering, kill logic
- [crates/winportkill-server](./crates/winportkill-server): Axum HTTP API and WebSocket stream
- [crates/winportkill-gui](./crates/winportkill-gui): native desktop GUI
- [.vscode-extension](./.vscode-extension): VS Code extension
- [jetbrains-plugin](./jetbrains-plugin): JetBrains plugin
- [packages/npm-cli](./packages/npm-cli): npm wrapper package
- [docs](./docs): implementation and architecture guides

## Documentation

- [Project Architecture](./docs/project-architecture.en.md)
- [Server Mode Guide](./docs/server-mode-guide.en.md)
- [VS Code Extension Guide](./docs/vscode-extension-guide.en.md)
- [JetBrains Plugin Guide](./docs/jetbrains-plugin-guide.en.md)
- [egui GUI Guide](./docs/egui-gui-guide.en.md)

## Boundaries

- actual inspection and kill flow is Windows x64-only
- killing protected processes may require elevated privileges
- the IDE integrations are shells around the same localhost sidecar model
- the native GUI and TUI call `winportkill-core` directly instead of going through HTTP
