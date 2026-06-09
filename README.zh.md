# WinPortKill

[English](./README.md) | 中文

WinPortKill 是一个面向 Windows x64 的端口与进程检查工具集，核心是共享的 Rust 后端。

当前对外提供四种使用形态：

- npm CLI：[`@noxlightman/winportkill`](https://www.npmjs.com/package/@noxlightman/winportkill)
- VS Code 扩展：Visual Studio Marketplace
- JetBrains 插件：JetBrains Marketplace 审核 / 上传流程
- 原生二进制：GitHub Releases

## 安装

### npm CLI

```powershell
npm install -g @noxlightman/winportkill
```

常用命令：

```powershell
winportkill list --json
winportkill who-uses 3000
winportkill kill --port 3000
winportkill --serve 3000
```

### VS Code

- 从 Visual Studio Marketplace 安装
- 或从 GitHub Releases 下载 `.vsix`

扩展会在首次使用时下载并缓存 Windows x64 sidecar。

### JetBrains IDE

- Marketplace 可用后直接安装
- 或使用 `jetbrains-plugin/build/distributions` 中的插件 zip 手动安装 / 上传

### 原生二进制

GitHub Releases 会发布 Windows x64 产物，例如：

- `winportkill-windows-x64.exe`
- `winportkill-gui-windows-x64.exe`
- `winportkill-vscode-<version>.vsix`
- `winportkill-jetbrains-plugin-<version>.zip`

## 功能

- 列出监听端口
- 按进程聚合端口
- 按 PID、进程名、协议、地址、端口过滤
- 结束 PID
- 为 IDE 集成提供 localhost HTTP sidecar

## 运行模式

### Terminal UI

根包 `winportkill` 默认启动 `ratatui` 终端界面。

- 每 10 秒刷新一次
- 支持过滤
- 支持 ports / processes 视图切换
- 可结束当前选中 PID

### JSON 模式

`--json` 输出一次快照后退出。

### 脚本 CLI 模式

- `list [--json]`
- `who-uses <port> [--json]`
- `kill --pid <pid> [--json]`
- `kill --port <port> [--json]`

### Server 模式

`--serve <port>` 启动 localhost HTTP 服务，供 IDE 集成使用。

### Native GUI

`winportkill-gui` 是基于 `winportkill-core` 的 `eframe` / `egui` 前端。

## 本地开发

```powershell
cargo build
cargo run -p winportkill
```

其他常用入口：

```powershell
cargo run -p winportkill -- --json
cargo run -p winportkill -- --serve 3000
cargo run -p winportkill-gui
cargo run -p winportkill -- list --json
cargo run -p winportkill -- who-uses 3000
cargo run -p winportkill -- kill --port 3000
```

## 仓库结构

- [Cargo.toml](./Cargo.toml)：workspace 根配置与顶层 TUI 二进制
- [src](./src)：终端 UI 入口与渲染
- [crates/winportkill-core](./crates/winportkill-core)：Windows 扫描、聚合、过滤、kill 逻辑
- [crates/winportkill-server](./crates/winportkill-server)：Axum HTTP API 与 WebSocket
- [crates/winportkill-gui](./crates/winportkill-gui)：原生桌面 GUI
- [.vscode-extension](./.vscode-extension)：VS Code 扩展
- [jetbrains-plugin](./jetbrains-plugin)：JetBrains 插件
- [packages/npm-cli](./packages/npm-cli)：npm 包装层
- [docs](./docs)：实现与架构文档

## 文档

- [Project Architecture](./docs/project-architecture.en.md) / [项目架构](./docs/project-architecture.zh.md)
- [Server Mode Guide](./docs/server-mode-guide.en.md) / [Server 模式指南](./docs/server-mode-guide.zh.md)
- [VS Code Extension Guide](./docs/vscode-extension-guide.en.md) / [VS Code 扩展指南](./docs/vscode-extension-guide.zh.md)
- [JetBrains Plugin Guide](./docs/jetbrains-plugin-guide.en.md) / [JetBrains 插件指南](./docs/jetbrains-plugin-guide.zh.md)
- [egui GUI Guide](./docs/egui-gui-guide.en.md) / [egui GUI 指南](./docs/egui-gui-guide.zh.md)

## 当前边界

- 实际检查与 kill 流程仅支持 Windows x64
- 结束受保护进程可能需要管理员权限
- IDE 集成围绕同一个 localhost sidecar 模型构建
- 原生 GUI 和 TUI 直接调用 `winportkill-core`，不经过 HTTP
