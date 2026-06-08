#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");

const packageRoot = path.resolve(__dirname, "..");
const repoRoot = path.resolve(packageRoot, "..", "..");
const vendorDir = path.join(packageRoot, "vendor");
const vendorBinary = path.join(vendorDir, "winportkill.exe");

const candidates = [
  process.env.WINPORTKILL_BIN,
  path.join(repoRoot, "target", "release", "winportkill.exe"),
  path.join(repoRoot, "release-assets", "winportkill-cli", "winportkill.exe"),
  path.join(repoRoot, ".vscode-extension", "bin", "win32-x64", "winportkill.exe")
].filter(Boolean);

const sourceBinary = candidates.find(isFile);
if (!sourceBinary) {
  fail(
    [
      "Unable to prepare npm package: winportkill.exe not found.",
      "Checked:",
      ...candidates.map((candidate) => `- ${candidate}`),
      "Build the release binary first with `cargo build --release -p winportkill`,",
      "or set WINPORTKILL_BIN to an explicit binary path."
    ].join("\n")
  );
}

fs.mkdirSync(vendorDir, { recursive: true });
fs.copyFileSync(sourceBinary, vendorBinary);
console.log(`[winportkill] bundled binary: ${sourceBinary} -> ${vendorBinary}`);

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
