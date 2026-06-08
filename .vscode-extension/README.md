# WinPortKill for VS Code

Inspect local Windows x64 listening ports and processes directly inside VS Code.

## Features

- Ports view and processes view in a sidebar webview
- Filter by PID, process name, protocol, address, or port
- Kill the selected process from the panel
- Command support for refresh and kill-selected
- Auto-refresh with configurable interval

## Requirements

- Windows x64
- Internet access on first launch to download the WinPortKill sidecar from GitHub Releases
- Or an explicit local sidecar path via `winportkill.sidecarPath`

## Commands

- `WinPortKill: Refresh`
- `WinPortKill: Kill Selected`

## Extension Settings

- `winportkill.refreshIntervalSeconds`
  - Refresh interval for the panel
  - Minimum: `3`
  - Default: `10`
- `winportkill.sidecarPath`
  - Optional absolute path to `winportkill.exe`
  - Overrides the cached GitHub Releases sidecar when set

## Notes

- Actual inspection and kill flow is Windows x64-only
- The extension caches the downloaded sidecar after the first successful launch
- Killing protected processes may require elevated privileges
- Kill actions terminate the whole process, not a single port binding

## Source

- Repository: https://github.com/NoxLightman/winportkill
