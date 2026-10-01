#!/usr/bin/env node
// Runs the prebuilt gen-animated-icons binary from the platform package npm installed
// as an optional dependency. No install scripts: npm 12, pnpm and Bun block them.
"use strict";
const { spawnSync } = require("node:child_process");

const PACKAGES = {
  "darwin-arm64": "@lottiefiles/gen-animated-icons-darwin-arm64",
  "darwin-x64": "@lottiefiles/gen-animated-icons-darwin-x64",
  "linux-arm64": "@lottiefiles/gen-animated-icons-linux-arm64-gnu",
  "linux-x64": "@lottiefiles/gen-animated-icons-linux-x64-gnu",
};

function binaryPath() {
  if (process.env.GEN_ANIMATED_ICONS_BINARY) return process.env.GEN_ANIMATED_ICONS_BINARY;
  const key = `${process.platform}-${process.arch}`;
  const pkg = PACKAGES[key];
  if (!pkg) {
    fail(`gen-animated-icons has no prebuilt binary for ${key}.`);
  }
  const exe = process.platform === "win32" ? "gen-animated-icons.exe" : "gen-animated-icons";
  try {
    return require.resolve(`${pkg}/bin/${exe}`);
  } catch {
    fail(
      `The platform package ${pkg} is missing. It installs as an optional dependency, ` +
        "so reinstall without --no-optional (or --omit=optional).",
    );
  }
}

// The intended launch address; the page must be live before the first publish (npm/README.md).
function fail(message) {
  console.error(`${message}\nOther ways to install: https://gen-animated-icons.lottiefiles.com/install`);
  process.exit(1);
}

const result = spawnSync(binaryPath(), process.argv.slice(2), { stdio: "inherit" });
if (result.error) fail(`Could not run gen-animated-icons: ${result.error.message}`);
process.exit(result.status === null ? 1 : result.status);
