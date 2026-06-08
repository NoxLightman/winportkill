import * as vscode from "vscode";
import * as cp from "node:child_process";
import * as fs from "node:fs";
import * as net from "node:net";
import * as path from "node:path";
import { ApiClient } from "./apiClient";

export class SidecarManager implements vscode.Disposable {
  private process: cp.ChildProcess | undefined;
  private apiClient: ApiClient | undefined;
  private startPromise: Promise<ApiClient> | undefined;
  private readonly output = vscode.window.createOutputChannel("WinPortKill");

  constructor(private readonly context: vscode.ExtensionContext) {}

  async ensureStarted(): Promise<ApiClient> {
    if (this.apiClient) {
      return this.apiClient;
    }
    if (this.startPromise) {
      return this.startPromise;
    }

    const startPromise = this.startSidecar();
    this.startPromise = startPromise;

    try {
      const client = await startPromise;
      this.apiClient = client;
      return client;
    } catch (error) {
      this.cleanupFailedStart();
      throw error;
    } finally {
      if (this.startPromise === startPromise) {
        this.startPromise = undefined;
      }
    }
  }

  dispose(): void {
    this.startPromise = undefined;
    this.apiClient = undefined;
    this.process?.kill();
    this.process = undefined;
    this.output.dispose();
  }

  private async startSidecar(): Promise<ApiClient> {
    const port = await pickFreePort();
    const binaryPath = this.resolveBinaryPath();

    if (!fs.existsSync(binaryPath)) {
      throw new Error(`WinPortKill sidecar binary not found: ${binaryPath}`);
    }

    this.output.appendLine(`Starting sidecar: ${binaryPath} --serve ${port}`);
    const sidecar = cp.spawn(binaryPath, ["--serve", String(port)], {
      cwd: path.dirname(binaryPath),
      windowsHide: true,
      stdio: ["ignore", "pipe", "pipe"]
    });
    this.process = sidecar;

    sidecar.stdout?.on("data", (chunk) => {
      this.output.appendLine(`[sidecar] ${String(chunk).trim()}`);
    });
    sidecar.stderr?.on("data", (chunk) => {
      this.output.appendLine(`[sidecar:error] ${String(chunk).trim()}`);
    });
    sidecar.on("error", (error) => {
      this.output.appendLine(`Sidecar failed to start: ${error.message}`);
      this.process = undefined;
      this.apiClient = undefined;
    });
    sidecar.on("exit", (code, signal) => {
      this.output.appendLine(`Sidecar exited with code=${code} signal=${signal}`);
      this.process = undefined;
      this.apiClient = undefined;
    });

    const client = new ApiClient(`http://127.0.0.1:${port}`);
    await waitForHealthOrExit(client, sidecar, 10000);
    return client;
  }

  private resolveBinaryPath(): string {
    if (process.platform !== "win32") {
      throw new Error("WinPortKill currently supports only Windows x64.");
    }
    if (process.arch !== "x64") {
      throw new Error(`WinPortKill currently supports only Windows x64. Detected: ${process.arch}`);
    }

    return this.context.asAbsolutePath(path.join("bin", "win32-x64", "winportkill.exe"));
  }

  private cleanupFailedStart(): void {
    this.apiClient = undefined;
    if (!this.process) {
      return;
    }

    this.output.appendLine("Cleaning up failed sidecar start");
    this.process.kill();
    this.process = undefined;
  }
}

async function pickFreePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.on("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        reject(new Error("Failed to allocate free port"));
        return;
      }
      const { port } = address;
      server.close((closeError) => {
        if (closeError) {
          reject(closeError);
          return;
        }
        resolve(port);
      });
    });
  });
}

async function waitForHealth(client: ApiClient, timeoutMs: number): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  let lastError: unknown;

  while (Date.now() < deadline) {
    try {
      const response = await client.health();
      if (response.status === "ok") {
        return;
      }
    } catch (error) {
      lastError = error;
    }
    await sleep(250);
  }

  throw new Error(`Sidecar health check timed out${lastError ? `: ${String(lastError)}` : ""}`);
}

async function waitForHealthOrExit(
  client: ApiClient,
  process: cp.ChildProcess,
  timeoutMs: number
): Promise<void> {
  return new Promise((resolve, reject) => {
    let settled = false;

    const cleanup = () => {
      process.off("error", onError);
      process.off("exit", onExit);
    };

    const finishResolve = () => {
      if (settled) {
        return;
      }
      settled = true;
      cleanup();
      resolve();
    };

    const finishReject = (error: Error) => {
      if (settled) {
        return;
      }
      settled = true;
      cleanup();
      reject(error);
    };

    const onError = (error: Error) => {
      finishReject(new Error(`Failed to start sidecar: ${error.message}`));
    };

    const onExit = (code: number | null, signal: NodeJS.Signals | null) => {
      finishReject(new Error(`Sidecar exited before becoming healthy (code=${code} signal=${signal})`));
    };

    process.once("error", onError);
    process.once("exit", onExit);

    waitForHealth(client, timeoutMs).then(finishResolve, (error: unknown) => {
      finishReject(error instanceof Error ? error : new Error(String(error)));
    });
  });
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
