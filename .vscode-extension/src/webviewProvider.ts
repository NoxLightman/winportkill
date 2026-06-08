import * as vscode from "vscode";
import { ApiClient } from "./apiClient";
import { SidecarManager } from "./sidecarManager";
import { KillResult, ProcessEntry, ProcessResponse, PortEntry, PortResponse, ViewMode } from "./types";

type PanelState =
  | { type: "loading"; message: string }
  | {
      type: "ready";
      mode: ViewMode;
      filter: string;
      selectedRowId: string | null;
      selectedPid: number | null;
      payload: PortResponse | ProcessResponse;
      status: string;
      statusKind: "ok" | "error" | "info";
    }
  | { type: "error"; message: string };

type IncomingMessage =
  | { type: "ready" }
  | { type: "refresh"; mode: ViewMode; filter: string }
  | { type: "switchMode"; mode: ViewMode; filter: string }
  | { type: "select"; mode: ViewMode; filter: string; rowId: string; pid: number }
  | { type: "kill"; pid: number; mode: ViewMode; filter: string };

export class WinPortKillViewProvider
  implements vscode.WebviewViewProvider, vscode.Disposable
{
  private view: vscode.WebviewView | undefined;
  private requestVersion = 0;
  private refreshTimer: NodeJS.Timeout | undefined;
  private currentMode: ViewMode = "ports";
  private currentFilter = "";
  private selectedRowId: string | null = null;
  private selectedPid: number | null = null;
  private lastPayload: PortResponse | ProcessResponse | undefined;
  private readonly configurationSubscription: vscode.Disposable;

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly sidecarManager: SidecarManager
  ) {
    this.configurationSubscription = vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("winportkill.refreshIntervalSeconds")) {
        this.configureAutoRefresh();
      }
    });
  }

  dispose(): void {
    this.stopAutoRefresh();
    this.configurationSubscription.dispose();
  }

  resolveWebviewView(webviewView: vscode.WebviewView): void | Thenable<void> {
    this.view = webviewView;
    webviewView.webview.options = {
      enableScripts: true
    };
    webviewView.webview.html = getWebviewHtml(webviewView.webview);

    webviewView.webview.onDidReceiveMessage(async (message: IncomingMessage) => {
      await this.handleMessage(message);
    });
    webviewView.onDidChangeVisibility(() => {
      this.configureAutoRefresh();
    });
    webviewView.onDidDispose(() => {
      if (this.view === webviewView) {
        this.view = undefined;
      }
      this.stopAutoRefresh();
    });

    this.configureAutoRefresh();
  }

  async refresh(mode: ViewMode = "ports", filter = ""): Promise<void> {
    await this.pushData(mode, filter, "Refreshed", "info");
  }

  async killSelected(): Promise<void> {
    if (!this.view) {
      vscode.window.showInformationMessage("Open the WinPortKill panel and select a row first.");
      return;
    }

    if (!this.lastPayload) {
      await this.pushData(this.currentMode, this.currentFilter, "", "info");
    }

    const selection = resolveSelection(this.currentMode, this.lastPayload, this.selectedRowId);
    if (!selection.pid) {
      vscode.window.showInformationMessage("No WinPortKill row is selected.");
      return;
    }

    await this.killAndRefresh(selection.pid, this.currentMode, this.currentFilter);
  }

  private async handleMessage(message: IncomingMessage): Promise<void> {
    switch (message.type) {
      case "ready":
        await this.pushData("ports", "", "");
        return;
      case "refresh":
      case "switchMode":
        await this.pushData(message.mode, message.filter, "");
        return;
      case "select":
        this.currentMode = message.mode;
        this.currentFilter = message.filter;
        this.selectedRowId = message.rowId;
        this.selectedPid = message.pid;
        return;
      case "kill":
        await this.killAndRefresh(message.pid, message.mode, message.filter);
        return;
      default:
        return;
    }
  }

  private async killAndRefresh(pid: number, mode: ViewMode, filter: string): Promise<void> {
    const choice = await vscode.window.showWarningMessage(
      `Terminate PID ${pid}?`,
      { modal: true },
      "Kill"
    );
    if (choice !== "Kill") {
      await this.pushData(mode, filter, "Kill cancelled", "info", { showLoading: false });
      return;
    }

    try {
      const client = await this.sidecarManager.ensureStarted();
      const result: KillResult = await client.kill(pid);
      await this.pushData(mode, filter, result.message, result.success ? "ok" : "error");
    } catch (error) {
      this.postState({
        type: "error",
        message: `Kill failed: ${toMessage(error)}`
      });
    }
  }

  private async pushData(
    mode: ViewMode,
    filter: string,
    status: string,
    statusKind: "ok" | "error" | "info" = "info",
    options: { showLoading?: boolean } = {}
  ): Promise<void> {
    const requestVersion = ++this.requestVersion;
    const previousSelection = this.currentMode === mode ? this.selectedRowId : null;
    if (options.showLoading !== false) {
      this.postState({ type: "loading", message: "Loading WinPortKill..." });
    }

    try {
      const client: ApiClient = await this.sidecarManager.ensureStarted();
      const payload = await client.fetchView(mode, filter);
      if (requestVersion !== this.requestVersion) {
        return;
      }

      const selection = resolveSelection(mode, payload, previousSelection);
      this.currentMode = mode;
      this.currentFilter = filter;
      this.selectedRowId = selection.rowId;
      this.selectedPid = selection.pid;
      this.lastPayload = payload;

      this.postState({
        type: "ready",
        mode,
        filter,
        selectedRowId: selection.rowId,
        selectedPid: selection.pid,
        payload,
        status,
        statusKind
      });
    } catch (error) {
      if (requestVersion !== this.requestVersion) {
        return;
      }
      this.postState({
        type: "error",
        message: toMessage(error)
      });
    }
  }

  private postState(state: PanelState): void {
    this.view?.webview.postMessage(state);
  }

  private configureAutoRefresh(): void {
    this.stopAutoRefresh();
    if (!this.view?.visible) {
      return;
    }

    this.refreshTimer = setInterval(() => {
      void this.pushData(this.currentMode, this.currentFilter, "", "info", {
        showLoading: false
      });
    }, getRefreshIntervalMs());
  }

  private stopAutoRefresh(): void {
    if (this.refreshTimer) {
      clearInterval(this.refreshTimer);
      this.refreshTimer = undefined;
    }
  }
}

function toMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function getRefreshIntervalMs(): number {
  const seconds = vscode.workspace
    .getConfiguration("winportkill")
    .get<number>("refreshIntervalSeconds", 10);
  return Math.max(3, seconds) * 1000;
}

function resolveSelection(
  mode: ViewMode,
  payload: PortResponse | ProcessResponse | undefined,
  selectedRowId: string | null
): { rowId: string | null; pid: number | null } {
  if (!payload) {
    return { rowId: null, pid: null };
  }

  const rows = rowsForPayload(mode, payload);
  if (selectedRowId) {
    const matched = rows.find((row) => row.rowId === selectedRowId);
    if (matched) {
      return matched;
    }
  }

  return rows[0] ?? { rowId: null, pid: null };
}

function rowsForPayload(
  mode: ViewMode,
  payload: PortResponse | ProcessResponse
): Array<{ rowId: string; pid: number }> {
  if (mode === "ports") {
    return (payload as PortResponse).entries.map((entry) => ({
      rowId: portRowId(entry),
      pid: entry.pid
    }));
  }

  return (payload as ProcessResponse).entries.map((entry) => ({
    rowId: processRowId(entry),
    pid: entry.pid
  }));
}

function portRowId(entry: PortEntry): string {
  return `port:${entry.proto}:${entry.local_addr}:${entry.port}:${entry.pid}`;
}

function processRowId(entry: ProcessEntry): string {
  return `process:${entry.pid}`;
}

function getWebviewHtml(webview: vscode.Webview): string {
  const nonce = String(Date.now());
  return `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; script-src 'nonce-${nonce}';" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>WinPortKill</title>
    <style>
      :root {
        --panel-border: var(--vscode-panel-border);
        --panel-bg: color-mix(in srgb, var(--vscode-sideBar-background) 86%, var(--vscode-editor-background));
        --panel-raised: color-mix(in srgb, var(--vscode-sideBar-background) 72%, var(--vscode-editor-background));
        --muted: var(--vscode-descriptionForeground);
      }
      body {
        font-family: var(--vscode-font-family);
        color: var(--vscode-foreground);
        background: var(--vscode-sideBar-background);
        margin: 0;
        padding: 10px;
      }
      * {
        box-sizing: border-box;
      }
      .shell {
        display: grid;
        gap: 10px;
      }
      .toolbar {
        display: grid;
        grid-template-columns: auto 1fr auto;
        gap: 8px;
        align-items: center;
        padding: 8px;
        border: 1px solid var(--panel-border);
        background: linear-gradient(180deg, color-mix(in srgb, var(--panel-bg) 75%, transparent), transparent);
      }
      .toolbar input {
        width: 100%;
        min-width: 0;
        background: var(--vscode-input-background);
        color: var(--vscode-input-foreground);
        border: 1px solid var(--vscode-input-border, transparent);
        padding: 8px 10px;
      }
      .segmented {
        display: inline-flex;
        gap: 6px;
      }
      button {
        background: var(--vscode-button-background);
        color: var(--vscode-button-foreground);
        border: none;
        padding: 8px 12px;
        cursor: pointer;
      }
      button.secondary {
        background: transparent;
        border: 1px solid var(--panel-border);
      }
      button.active {
        outline: 1px solid var(--vscode-focusBorder);
      }
      .status {
        min-height: 24px;
        display: flex;
        align-items: center;
        flex-wrap: wrap;
        gap: 8px;
      }
      .metric-strip {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
      }
      .metric {
        padding: 4px 8px;
        border: 1px solid var(--panel-border);
        background: var(--panel-raised);
        font-size: 11px;
        color: var(--muted);
      }
      .table-wrap {
        border: 1px solid var(--panel-border);
        overflow: hidden;
      }
      table {
        width: 100%;
        border-collapse: collapse;
        font-size: 12px;
      }
      th, td {
        padding: 6px 8px;
        border-bottom: 1px solid var(--panel-border);
        text-align: left;
        vertical-align: top;
      }
      th {
        font-size: 11px;
        color: var(--muted);
      }
      tr {
        cursor: pointer;
      }
      tr:hover {
        background: var(--vscode-list-hoverBackground);
      }
      tr.selected {
        background: color-mix(in srgb, var(--vscode-list-activeSelectionBackground) 82%, transparent);
      }
      .mono {
        font-variant-numeric: tabular-nums;
        font-family: var(--vscode-editor-font-family, var(--vscode-font-family));
      }
      .muted {
        color: var(--muted);
      }
      .error {
        color: var(--vscode-errorForeground);
      }
      .ok {
        color: var(--vscode-terminal-ansiGreen);
      }
      .info {
        color: var(--vscode-descriptionForeground);
      }
      .ports {
        font-size: 11px;
        line-height: 1.4;
      }
      .card-list {
        display: grid;
        gap: 8px;
      }
      .card {
        border: 1px solid var(--panel-border);
        background: linear-gradient(180deg, color-mix(in srgb, var(--panel-bg) 90%, transparent), transparent);
        padding: 10px;
        cursor: pointer;
      }
      .card.selected {
        border-color: var(--vscode-focusBorder);
        background: color-mix(in srgb, var(--vscode-list-activeSelectionBackground) 16%, var(--panel-bg));
      }
      .card-top {
        display: grid;
        grid-template-columns: auto 1fr auto;
        gap: 8px;
        align-items: start;
      }
      .card-main {
        min-width: 0;
      }
      .card-title {
        font-size: 13px;
        font-weight: 600;
        line-height: 1.3;
        overflow-wrap: anywhere;
      }
      .card-subtitle {
        margin-top: 2px;
        font-size: 11px;
        color: var(--muted);
      }
      .badge {
        min-width: 52px;
        padding: 4px 8px;
        border: 1px solid var(--panel-border);
        background: var(--panel-raised);
        font-size: 11px;
        font-weight: 600;
        text-align: center;
      }
      .badge.tcp {
        color: var(--vscode-terminal-ansiCyan);
      }
      .badge.udp {
        color: var(--vscode-terminal-ansiGreen);
      }
      .badge.pid {
        color: var(--vscode-textLink-foreground);
      }
      .meta-grid {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 6px 10px;
        margin-top: 8px;
      }
      .meta-label {
        font-size: 10px;
        text-transform: uppercase;
        letter-spacing: 0.04em;
        color: var(--muted);
      }
      .meta-value {
        margin-top: 2px;
        font-size: 12px;
        line-height: 1.35;
        overflow-wrap: anywhere;
      }
      .ports-preview {
        margin-top: 8px;
        font-size: 11px;
        line-height: 1.5;
        color: var(--muted);
      }
      .card-action {
        min-width: 56px;
      }
      @media (max-width: 760px) {
        .toolbar {
          grid-template-columns: 1fr auto;
        }
        .segmented {
          grid-column: 1 / -1;
        }
      }
      @media (max-width: 520px) {
        body {
          padding: 8px;
        }
        .toolbar {
          grid-template-columns: 1fr;
        }
        .segmented {
          width: 100%;
        }
        .segmented button,
        #refreshBtn {
          flex: 1;
        }
        .card-top {
          grid-template-columns: auto 1fr;
        }
        .card-action {
          grid-column: 1 / -1;
          width: 100%;
        }
        .meta-grid {
          grid-template-columns: 1fr;
        }
      }
    </style>
  </head>
  <body>
    <div class="shell">
      <div class="toolbar">
        <div class="segmented">
          <button id="portsBtn" class="active">Ports</button>
          <button id="processesBtn" class="secondary">Processes</button>
        </div>
        <input id="filterInput" placeholder="Filter by pid, port, proto, process..." />
        <button id="refreshBtn">Refresh</button>
      </div>
      <div id="status" class="status muted">Starting WinPortKill...</div>
      <div id="tableContainer"></div>
    </div>
    <script nonce="${nonce}">
      const vscode = acquireVsCodeApi();
      let mode = "ports";
      let filter = "";
      let selectedRowId = null;
      let selectedPid = null;
      let isComposing = false;
      let refreshTimer = undefined;
      let lastReadyState = undefined;

      const portsBtn = document.getElementById("portsBtn");
      const processesBtn = document.getElementById("processesBtn");
      const filterInput = document.getElementById("filterInput");
      const refreshBtn = document.getElementById("refreshBtn");
      const statusEl = document.getElementById("status");
      const tableContainer = document.getElementById("tableContainer");

      function syncButtons() {
        portsBtn.classList.toggle("active", mode === "ports");
        processesBtn.classList.toggle("active", mode === "processes");
        portsBtn.classList.toggle("secondary", mode !== "ports");
        processesBtn.classList.toggle("secondary", mode !== "processes");
      }

      function requestRefresh(kind) {
        vscode.postMessage({ type: kind, mode, filter });
      }

      function scheduleRefresh(delay = 180) {
        if (refreshTimer) {
          clearTimeout(refreshTimer);
        }
        refreshTimer = setTimeout(() => {
          refreshTimer = undefined;
          requestRefresh("refresh");
        }, delay);
      }

      portsBtn.addEventListener("click", () => {
        mode = "ports";
        syncButtons();
        requestRefresh("switchMode");
      });

      processesBtn.addEventListener("click", () => {
        mode = "processes";
        syncButtons();
        requestRefresh("switchMode");
      });

      filterInput.addEventListener("input", (event) => {
        filter = event.target.value;
        if (!isComposing) {
          scheduleRefresh();
        }
      });

      filterInput.addEventListener("compositionstart", () => {
        isComposing = true;
      });

      filterInput.addEventListener("compositionend", (event) => {
        isComposing = false;
        filter = event.target.value;
        scheduleRefresh(0);
      });

      refreshBtn.addEventListener("click", () => requestRefresh("refresh"));

      function isCompact() {
        return window.innerWidth < 760;
      }

      function escapeHtml(value) {
        return String(value)
          .replaceAll("&", "&amp;")
          .replaceAll("<", "&lt;")
          .replaceAll(">", "&gt;")
          .replaceAll('"', "&quot;")
          .replaceAll("'", "&#39;");
      }

      function metric(label, value) {
        return \`<span class="metric"><span class="muted">\${label}</span> <span class="mono">\${escapeHtml(value)}</span></span>\`;
      }

      function portRowId(entry) {
        return \`port:\${entry.proto}:\${entry.local_addr}:\${entry.port}:\${entry.pid}\`;
      }

      function processRowId(entry) {
        return \`process:\${entry.pid}\`;
      }

      function applySelection(rowId, pid) {
        selectedRowId = rowId;
        selectedPid = pid;
        if (lastReadyState?.type === "ready") {
          lastReadyState = { ...lastReadyState, selectedRowId, selectedPid };
          renderReadyState(lastReadyState);
        }
        vscode.postMessage({ type: "select", mode, filter, rowId, pid });
      }

      function renderStatus(state) {
        if (state.status) {
          statusEl.className = \`status \${state.statusKind}\`;
          statusEl.textContent = state.status;
          return;
        }

        statusEl.className = "status muted";
        const stats = state.payload.stats;
        statusEl.innerHTML = mode === "ports"
          ? \`<div class="metric-strip">\${metric("Rows", stats.total_rows)}\${metric("Proc", stats.total_procs)}\${metric("TCP", stats.tcp_count)}\${metric("UDP", stats.udp_count)}</div>\`
          : \`<div class="metric-strip">\${metric("Proc", stats.total_procs)}\${metric("Active", stats.procs_with_ports)}\${metric("TCP", stats.tcp_count)}\${metric("Bindings", stats.total_port_bindings)}</div>\`;
      }

      function renderPorts(payload) {
        if (isCompact()) {
          const cards = payload.entries.map((entry) => {
            const rowId = portRowId(entry);
            return \`
              <article class="card \${selectedRowId === rowId ? "selected" : ""}" data-row-id="\${escapeHtml(rowId)}" data-pid="\${entry.pid}">
                <div class="card-top">
                  <span class="badge \${entry.proto.startsWith("TCP") ? "tcp" : "udp"}">\${escapeHtml(entry.proto)}</span>
                  <div class="card-main">
                    <div class="card-title mono">\${escapeHtml(entry.local_addr)}:\${escapeHtml(entry.port)}</div>
                    <div class="card-subtitle">\${escapeHtml(entry.name)}</div>
                  </div>
                  <button class="card-action" data-pid="\${entry.pid}">Kill</button>
                </div>
                <div class="meta-grid">
                  <div>
                    <div class="meta-label">PID</div>
                    <div class="meta-value mono">\${entry.pid}</div>
                  </div>
                  <div>
                    <div class="meta-label">Memory</div>
                    <div class="meta-value mono">\${(entry.memory / 1024 / 1024).toFixed(1)} MB</div>
                  </div>
                </div>
              </article>
            \`;
          }).join("");

          tableContainer.innerHTML = \`<div class="card-list">\${cards}</div>\`;
          return;
        }

        const rows = payload.entries.map((entry) => {
          const rowId = portRowId(entry);
          return \`
            <tr class="\${selectedRowId === rowId ? "selected" : ""}" data-row-id="\${escapeHtml(rowId)}" data-pid="\${entry.pid}">
              <td>\${escapeHtml(entry.proto)}</td>
              <td class="mono">\${escapeHtml(entry.local_addr)}</td>
              <td class="mono">\${escapeHtml(entry.port)}</td>
              <td class="mono">\${entry.pid}</td>
              <td class="mono">\${(entry.memory / 1024 / 1024).toFixed(1)}</td>
              <td>\${escapeHtml(entry.name)}</td>
              <td><button data-pid="\${entry.pid}">Kill</button></td>
            </tr>
          \`;
        }).join("");

        tableContainer.innerHTML = \`
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Proto</th><th>Addr</th><th>Port</th><th>PID</th><th>Mem(MB)</th><th>Process</th><th></th>
                </tr>
              </thead>
              <tbody>\${rows}</tbody>
            </table>
          </div>
        \`;
      }

      function renderProcesses(payload) {
        if (isCompact()) {
          const cards = payload.entries.map((entry) => {
            const rowId = processRowId(entry);
            const ports = entry.ports.length
              ? entry.ports.slice(0, 3).map((port) => \`\${escapeHtml(port.proto)} \${escapeHtml(port.local_addr)}:\${escapeHtml(port.port)}\`).join("<br/>")
              : "No listening ports";
            return \`
              <article class="card \${selectedRowId === rowId ? "selected" : ""}" data-row-id="\${escapeHtml(rowId)}" data-pid="\${entry.pid}">
                <div class="card-top">
                  <span class="badge pid mono">\${entry.pid}</span>
                  <div class="card-main">
                    <div class="card-title">\${escapeHtml(entry.name)}</div>
                    <div class="card-subtitle">TCP \${entry.tcp_ports}  UDP \${entry.udp_ports}</div>
                  </div>
                  <button class="card-action" data-pid="\${entry.pid}">Kill</button>
                </div>
                <div class="meta-grid">
                  <div>
                    <div class="meta-label">Memory</div>
                    <div class="meta-value mono">\${(entry.memory / 1024 / 1024).toFixed(1)} MB</div>
                  </div>
                  <div>
                    <div class="meta-label">Bindings</div>
                    <div class="meta-value mono">\${entry.ports.length}</div>
                  </div>
                </div>
                <div class="ports-preview">\${ports}</div>
              </article>
            \`;
          }).join("");

          tableContainer.innerHTML = \`<div class="card-list">\${cards}</div>\`;
          return;
        }

        const rows = payload.entries.map((entry) => {
          const rowId = processRowId(entry);
          const ports = entry.ports.length
            ? entry.ports.slice(0, 3).map((port) => \`\${escapeHtml(port.proto)} \${escapeHtml(port.local_addr)}:\${escapeHtml(port.port)}\`).join("<br/>")
            : "No listening ports";
          return \`
            <tr class="\${selectedRowId === rowId ? "selected" : ""}" data-row-id="\${escapeHtml(rowId)}" data-pid="\${entry.pid}">
              <td class="mono">\${entry.pid}</td>
              <td class="mono">\${entry.tcp_ports}</td>
              <td class="mono">\${entry.udp_ports}</td>
              <td class="mono">\${(entry.memory / 1024 / 1024).toFixed(1)}</td>
              <td><strong>\${escapeHtml(entry.name)}</strong><div class="ports muted">\${ports}</div></td>
              <td><button data-pid="\${entry.pid}">Kill</button></td>
            </tr>
          \`;
        }).join("");

        tableContainer.innerHTML = \`
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>PID</th><th>TCP</th><th>UDP</th><th>Mem(MB)</th><th>Process / Ports</th><th></th>
                </tr>
              </thead>
              <tbody>\${rows}</tbody>
            </table>
          </div>
        \`;
      }

      function renderReadyState(state) {
        mode = state.mode;
        filter = state.filter;
        selectedRowId = state.selectedRowId ?? null;
        selectedPid = state.selectedPid ?? null;
        if (!isComposing && document.activeElement !== filterInput && filterInput.value !== filter) {
          filterInput.value = filter;
        }
        syncButtons();
        renderStatus(state);
        if (mode === "ports") {
          renderPorts(state.payload);
        } else {
          renderProcesses(state.payload);
        }
      }

      tableContainer.addEventListener("click", (event) => {
        const target = event.target;
        if (!(target instanceof Element)) {
          return;
        }

        const button = target.closest("button[data-pid]");
        const row = target.closest("[data-row-id]");
        const rowId = row?.getAttribute("data-row-id");
        const rowPid = Number(row?.getAttribute("data-pid"));
        if (rowId && rowPid) {
          applySelection(rowId, rowPid);
        }

        if (!(button instanceof HTMLButtonElement)) {
          return;
        }
        const pid = Number(button.dataset.pid);
        if (!pid) {
          return;
        }
        vscode.postMessage({ type: "kill", pid, mode, filter });
      });

      window.addEventListener("message", (event) => {
        const state = event.data;
        if (state.type === "loading") {
          statusEl.className = "status muted";
          statusEl.textContent = state.message;
          return;
        }
        if (state.type === "error") {
          statusEl.className = "status error";
          statusEl.textContent = state.message;
          return;
        }
        if (state.type === "ready") {
          lastReadyState = state;
          renderReadyState(state);
        }
      });

      window.addEventListener("resize", () => {
        if (lastReadyState) {
          renderReadyState(lastReadyState);
        }
      });

      vscode.postMessage({ type: "ready" });
    </script>
  </body>
</html>`;
}
