#!/usr/bin/env node
"use strict";

const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

if (process.platform !== "win32") {
  fail(`WinPortKill currently supports only Windows x64. Detected platform: ${process.platform}`);
}

if (process.arch !== "x64") {
  fail(`WinPortKill currently supports only Windows x64. Detected arch: ${process.arch}`);
}

const binaryPath = resolveBinaryPath();
const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
  windowsHide: true
});

if (result.error) {
  fail(`Failed to start ${binaryPath}: ${result.error.message}`);
}

if (typeof result.status === "number") {
  process.exit(result.status);
}

fail(`WinPortKill terminated unexpectedly${result.signal ? ` with signal ${result.signal}` : ""}`);

function resolveBinaryPath() {
  const packageRoot = path.resolve(__dirname, "..");
  const repoRoot = path.resolve(packageRoot, "..", "..");

  const candidates = [
    process.env.WINPORTKILL_BIN,
    path.join(repoRoot, "target", "debug", "winportkill.exe"),
    path.join(repoRoot, "target", "release", "winportkill.exe"),
    path.join(repoRoot, ".vscode-extension", "bin", "win32-x64", "winportkill.exe"),
    path.join(packageRoot, "vendor", "winportkill.exe")
  ].filter(Boolean);

  for (const candidate of candidates) {
    if (isFile(candidate)) {
      return candidate;
    }
  }

  fail(
    [
      "Unable to locate winportkill.exe.",
      "Checked:",
      ...candidates.map((candidate) => `- ${candidate}`),
      "Set WINPORTKILL_BIN to an explicit binary path, or build the Rust workspace first."
    ].join("\n")
  );
}

function isFile(filePath) {
  try {
    return fs.statSync(filePath).isFile();
  } catch {
    return false;
  }
}

function fail(message) {
  console.error(`[winportkill] ${message}`);
  process.exit(1);
}
