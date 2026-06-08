# JetBrains Plugin Guide

English | [中文](./jetbrains-plugin-guide.zh.md)

Back: [README](../README.md)

## Scope

This document covers the current WinPortKill JetBrains plugin MVP under [jetbrains-plugin](../jetbrains-plugin).

- target: JetBrains IDEs on Windows
- UI: Tool Window for port and process inspection
- backend: local sidecar process exposed over HTTP
- status: `runIde` works and the MVP can list, filter, and kill by PID

## Important Files

- [`build.gradle.kts`](../jetbrains-plugin/build.gradle.kts): IntelliJ Platform setup and `runIde`
- [`src/main/resources/META-INF/plugin.xml`](../jetbrains-plugin/src/main/resources/META-INF/plugin.xml): plugin registration
- [`toolwindow/WinPortKillToolWindowFactory.kt`](../jetbrains-plugin/src/main/kotlin/dev/winportkill/jetbrains/toolwindow/WinPortKillToolWindowFactory.kt): Tool Window entry
- [`ui/WinPortKillPanel.kt`](../jetbrains-plugin/src/main/kotlin/dev/winportkill/jetbrains/ui/WinPortKillPanel.kt): Swing UI and responsive tables
- [`sidecar/SidecarManager.kt`](../jetbrains-plugin/src/main/kotlin/dev/winportkill/jetbrains/sidecar/SidecarManager.kt): sidecar lifecycle and binary lookup
- [`api/ApiClient.kt`](../jetbrains-plugin/src/main/kotlin/dev/winportkill/jetbrains/api/ApiClient.kt): localhost HTTP client

## Local Development

Prerequisites:

- Windows
- JDK matching `jetbrains-plugin/gradle.properties`
- sidecar binary available at `.vscode-extension/bin/win32-x64/winportkill.exe`

Run the sandbox IDE:

```powershell
cd .\jetbrains-plugin
.\gradlew.bat runIde
```

`runIde` injects `-Dwinportkill.dev.root=<repo-root>` so the plugin can reuse the checked-out sidecar binary.

## Sidecar Flow

1. `WinPortKillPanel.refresh()` requests an `ApiClient`.
2. `SidecarManager.ensureStarted()` reuses or starts the sidecar.
3. It launches `winportkill.exe --serve <port>`.
4. It waits for `/health == ok`.
5. The panel fetches ports, processes, or sends kill requests.

## UI Layout Strategy

The plugin uses one table system with width-based bands instead of switching into a separate card mode.

- `WIDE`: full column set
- `MEDIUM`: reduced secondary columns
- `COMPACT`: smallest column set with footer details

Current thresholds:

- `width <= 500`: `COMPACT`
- `501..860`: `MEDIUM`
- `> 860`: `WIDE`

## Packaging Notes

- `processResources` copies `.vscode-extension/bin/win32-x64/winportkill.exe` into plugin resources when available
- development prefers the external checked-out binary
- the current MVP enforces Windows support in code

## Marketplace Notes

- primary distribution target: JetBrains Marketplace
- packaged artifact: `jetbrains-plugin/build/distributions/*.zip`
- current publisher metadata comes from `plugin.xml`
- release automation should attach the plugin zip to GitHub Releases even before Marketplace upload is automated
