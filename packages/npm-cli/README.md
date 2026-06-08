# `@noxlightman/winportkill`

Windows x64 CLI launcher for WinPortKill.

## Current Behavior

This package is a thin Node wrapper around the Rust binary. Published tarballs are
expected to include `vendor/winportkill.exe`.

At runtime, it resolves `winportkill.exe` in the following order:

1. `WINPORTKILL_BIN`
2. `target/debug/winportkill.exe` in the workspace
3. `target/release/winportkill.exe` in the workspace
4. `.vscode-extension/bin/win32-x64/winportkill.exe` in the workspace
5. `vendor/winportkill.exe` inside this package

## Local Development

From the repository root:

```powershell
cargo build --release -p winportkill
node .\packages\npm-cli\bin\winportkill.js list --json
node .\packages\npm-cli\bin\winportkill.js who-uses 3000
```

Or from this package directory:

```powershell
npm.cmd run test:local
npm.cmd pack
```

## Notes

- Supported platform: Windows x64 only
- The npm wrapper forwards all arguments directly to the Rust CLI
- `npm pack` / `npm publish` runs `prepack`, which copies a release binary into `vendor/`
- A later publishing step can replace the local binary copy with a GitHub Releases download flow

## Commands

```powershell
winportkill list --json
winportkill who-uses 3000
winportkill kill --pid 1234
winportkill kill --port 3000
winportkill --serve 3000
```
