# 项目架构

[English](./project-architecture.en.md) | 中文

返回：[README](../README.zh.md)

## 概览

WinPortKill 不是单一程序，而是一套共享的 Windows x64 检查后端加多种前端：

- workspace 根目录下的 Rust TUI
- `crates/winportkill-server` 中的 Rust HTTP sidecar
- `crates/winportkill-gui` 中的 Rust 原生 GUI
- `.vscode-extension` 中的 VS Code 扩展
- `jetbrains-plugin` 中的 JetBrains 插件

核心设计思路是：

- 把 Windows 特定逻辑集中在一个 Rust crate 中
- Rust 前端直接调用它
- IDE 前端通过 localhost HTTP 复用它

## 组件划分

### `winportkill-core`

- 通过 Windows API 扫描监听中的 TCP / UDP 绑定
- 通过 `sysinfo` 关联进程名和内存
- 提供面向端口和面向进程两种视图模型
- 计算统计信息
- 按 PID 结束进程

### `winportkill-server`

- 用 Axum 将 `winportkill-core` 包装成 HTTP 路由
- 提供 `/health`、`/ports`、`/processes`、统计接口、`/kill/{pid}` 和 `/ws`
- 作为 IDE 集成使用的传输层

### 根包 `winportkill`

- 解析 CLI 参数
- 三种模式择一运行：
  - 默认 TUI
  - `--json`
  - `--serve <port>`

### `winportkill-gui`

- 基于 `eframe` / `egui` 的原生桌面应用
- 直接调用 `winportkill-core`
- 自己维护刷新、过滤、选中和 kill 确认状态

### `.vscode-extension`

- TypeScript 扩展宿主
- webview sidebar UI
- 通过 Node `child_process.spawn` 启动打包 sidecar

### `jetbrains-plugin`

- IntelliJ Platform 插件
- Swing Tool Window UI
- Kotlin HTTP client 和 project 级 sidecar manager

## 数据流

### 直接调用 core 的流程

用于 TUI 和原生 GUI：

1. UI 触发刷新或 kill。
2. `winportkill-core` 扫描端口和进程。
3. 前端过滤并渲染当前视图。
4. 如需 kill，前端直接调用同一个 core crate。

### Sidecar 流程

用于 VS Code 和 JetBrains：

1. 集成为 sidecar 分配一个空闲 localhost 端口。
2. 启动 `winportkill.exe --serve <port>`。
3. 轮询 `/health`，直到返回 `ok`。
4. 前端请求 `/ports` 或 `/processes`。
5. kill 动作调用 `POST /kill/{pid}`，然后刷新当前视图。

## 共享 Sidecar 策略

当前仓库中，IDE 相关流程实际共享同一个重要二进制来源：

- `.vscode-extension/bin/win32-x64/winportkill.exe`

这很重要，因为：

- VS Code 扩展从这里发货
- JetBrains 的 `runIde` 开发流程复用这里的二进制
- JetBrains 的 `processResources` 在可用时也从这里复制 sidecar

## 当前运行边界

- 检查后端在实践中当前仅支持 Windows x64。
- kill 是结束整个进程，不是“只释放某一个端口”。
- 受保护目标可能需要管理员权限。
- 当前 WebSocket 流只推送端口视图快照。

## 推荐阅读顺序

- [README](../README.zh.md)
- [Server 模式指南](./server-mode-guide.zh.md)
- [VS Code 扩展指南](./vscode-extension-guide.zh.md)
- [JetBrains 插件指南](./jetbrains-plugin-guide.zh.md)
