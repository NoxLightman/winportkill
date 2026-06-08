# WinPortKill

[English](./README.md) | 中文

WinPortKill 是一个面向 Windows x64 的端口与进程检查工具集，核心是可复用的 Rust 后端。

当前仓库包含：

- 一个终端 TUI 可执行程序
- 一个供 IDE 集成使用的 HTTP sidecar 模式
- 一个基于 `egui` 的原生桌面 GUI 原型
- 一个 VS Code 扩展
- 一个 JetBrains 插件 MVP

## 当前范围

- 平台：实际端口/进程检查流程当前仅支持 Windows x64
- 核心能力：列出监听端口、按进程聚合、过滤结果、按 PID 结束进程
- 共享后端：位于 [crates](./crates) 下的 Rust crates
- IDE 集成：VS Code webview 扩展和 JetBrains Tool Window

## Workspace 结构

- [Cargo.toml](./Cargo.toml)：workspace 根配置和顶层 TUI 二进制入口
- [src](./src)：终端 UI 入口、状态管理和渲染代码
- [crates/winportkill-core](./crates/winportkill-core)：Windows 端口扫描、进程聚合、过滤与 kill 逻辑
- [crates/winportkill-server](./crates/winportkill-server)：Axum HTTP API 与 WebSocket 流
- [crates/winportkill-gui](./crates/winportkill-gui)：原生 `egui` 桌面 GUI
- [.vscode-extension](./.vscode-extension)：VS Code 扩展与 webview UI
- [jetbrains-plugin](./jetbrains-plugin)：JetBrains 插件 MVP
- [docs](./docs)：中英文项目与实现文档

## 快速开始

```powershell
cargo build
cargo run -p winportkill
```

其他常用入口：

```powershell
cargo run -p winportkill -- --json
cargo run -p winportkill -- --serve 3000
cargo run -p winportkill-gui
```

## 运行模式

### Terminal UI

根包 `winportkill` 默认启动 `ratatui` 终端界面。

- 每 10 秒刷新一次数据
- 支持过滤
- 可在 ports 视图和 processes 视图之间切换
- 可以结束当前选中的 PID

### JSON 模式

`--json` 会输出一次快照后退出。

### Server 模式

`--serve <port>` 会启动一个本地 HTTP 服务，供 IDE 集成使用。

### Native GUI

`winportkill-gui` crate 是一个基于 `winportkill-core` 的 `eframe` / `egui` 前端。

## IDE 集成

### VS Code 扩展

VS Code 扩展位于 [.vscode-extension](./.vscode-extension)。

- 以 webview sidebar 形式运行
- 从 `.vscode-extension/bin/...` 启动打包好的 Rust sidecar
- 通过 localhost HTTP 调用 sidecar

### JetBrains 插件

JetBrains 插件位于 [jetbrains-plugin](./jetbrains-plugin)。

- 提供 Tool Window UI
- 以 `winportkill.exe --serve <port>` 的形式启动 sidecar
- 通过 Kotlin HTTP client 调用本地 sidecar

## 文档索引

- [Project Architecture](./docs/project-architecture.en.md) | [项目架构](./docs/project-architecture.zh.md)
- [Server Mode Guide](./docs/server-mode-guide.en.md) | [Server 模式指南](./docs/server-mode-guide.zh.md)
- [VS Code Extension Guide](./docs/vscode-extension-guide.en.md) | [VS Code 扩展指南](./docs/vscode-extension-guide.zh.md)
- [JetBrains Plugin Guide](./docs/jetbrains-plugin-guide.en.md) | [JetBrains 插件指南](./docs/jetbrains-plugin-guide.zh.md)
- [egui GUI Guide](./docs/egui-gui-guide.en.md) | [egui GUI 指南](./docs/egui-gui-guide.zh.md)

## 当前边界

- 实际的检查与 kill 流程当前仅支持 Windows x64。
- 结束受保护进程可能需要管理员权限。
- VS Code 扩展和 JetBrains 插件都是围绕同一个 localhost sidecar 模型的 IDE 外壳。
- 原生 GUI 和 TUI 直接调用 `winportkill-core`，不经过 HTTP。
