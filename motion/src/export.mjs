// Export an original Engine MCP scene through the native MCP server over stdio.
// Adapted from the local Engine MCP export CLI; accepts scene data as a file.
// node src/export.mjs scene.engine.js output.json scene.scene.json
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { spawn } from "node:child_process";
import readline from "node:readline";
import crypto from "node:crypto";
import os from "node:os";
import { exporterConfig } from "./exporter-config.mjs";

const [input, output, sceneFile] = process.argv.slice(2);
if (!input || !output || !sceneFile || process.argv.length !== 5)
  throw new Error("Usage: node motion/src/export.mjs scene.engine.js output.json scene.scene.json");
const { root, binary, env } = exporterConfig();
const params = JSON.stringify({
  scene: JSON.parse(fs.readFileSync(sceneFile, "utf8")),
});
const code = fs.readFileSync(input, "utf8");
// The legacy film helper sets both bounds, but the PR validates each setter
// immediately. Set the final end first when extending a fresh node's range.
const legacyRange =
  "engine.setStartTime(n, a); engine.setEndTime(n, Math.max(b, a + 0.001));";
const runnable = code.replace(
  legacyRange,
  "engine.setEndTime(n, Math.max(b, a + 0.001)); engine.setStartTime(n, a);",
);
const workspace = fs.mkdtempSync(path.join(os.tmpdir(), "lottie-export-"));
const child = spawn(binary, ["--stdio", "--workspace", workspace], {
  cwd: root,
  env,
  stdio: ["pipe", "pipe", "pipe"],
});
let serial = 1,
  stderr = "";
const pending = new Map();
child.stderr.on("data", (chunk) => {
  stderr = (stderr + chunk).slice(-16000);
});
const lines = readline.createInterface({ input: child.stdout });
lines.on("line", (line) => {
  try {
    const msg = JSON.parse(line),
      waiter = pending.get(msg.id);
    if (waiter) {
      pending.delete(msg.id);
      clearTimeout(waiter.timer);
      msg.error
        ? waiter.reject(new Error(JSON.stringify(msg.error)))
        : waiter.resolve(msg.result);
    }
  } catch (error) {
    abort(error);
  }
});
function abort(error) {
  for (const p of pending.values()) {
    clearTimeout(p.timer);
    p.reject(error);
  }
  pending.clear();
}
child.on("error", abort);
child.on("exit", (code) => abort(new Error(`MCP exited ${code}: ${stderr}`)));
function rpc(method, params) {
  return new Promise((resolve, reject) => {
    const id = serial++;
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(new Error(`MCP timeout: ${method}`));
    }, 90000);
    pending.set(id, { resolve, reject, timer });
    child.stdin.write(
      JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n",
    );
  });
}
async function call(name, args) {
  const result = await rpc("tools/call", { name, arguments: args });
  if (result.isError)
    throw new Error(
      result.content
        .filter((c) => c.type === "text")
        .map((c) => c.text)
        .join("\n"),
    );
  return result;
}
const sha = (data) => crypto.createHash("sha256").update(data).digest("hex");
try {
  await rpc("initialize", {
    protocolVersion: "2025-11-25",
    capabilities: {},
    clientInfo: { name: "native-lottie-export", version: "1.0.0" },
  });
  child.stdin.write(
    JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" }) +
      "\n",
  );
  const created = await call("engine_exec", {
    code: runnable,
    engineId: "export",
    params: JSON.parse(params),
    timeoutMs: 30000,
  });
  for (const [index, item] of created.content.entries())
    if (item.type === "image")
      fs.writeFileSync(
        `${output}.filmstrip-${index}.png`,
        Buffer.from(item.data, "base64"),
      );
  const state =
    created.structuredContent ||
    JSON.parse(created.content.find((c) => c.type === "text").text);
  if (state.result?.truncated) throw new Error("Scene result was truncated");
  const result = await call("engine_export_lottie", {
    engineId: "export",
    authoredGroups: true,
  });
  const text = result.content.find(
    (c) => c.type === "resource" && c.resource.mimeType === "application/json",
  )?.resource.text;
  if (!text) throw new Error("Export did not return a JSON resource");
  const lottie = JSON.parse(text);
  if (
    !Array.isArray(lottie.layers) ||
    !(lottie.fr > 0) ||
    !(lottie.op > lottie.ip)
  )
    throw new Error("Invalid Lottie composition");
  fs.mkdirSync(path.dirname(path.resolve(output)), { recursive: true });
  fs.writeFileSync(output, text);
  const record = {
    input: path.resolve(input),
    sceneFile: path.resolve(sceneFile),
    sceneSha256: sha(fs.readFileSync(sceneFile)),
    scriptSha256: sha(code),
    executedScriptSha256: sha(runnable),
    compatibility:
      runnable === code
        ? []
        : [
            "Set final end before start in the legacy show helper; final time range unchanged.",
          ],
    exporterVersion: 2,
    output: path.resolve(output),
    sha256: sha(text),
    bytes: Buffer.byteLength(text),
    engineRevision: "27a04e8598719d5357af502007ffb66b5d94edbd",
    documentRevision: state.revision,
    width: lottie.w,
    height: lottie.h,
    fps: lottie.fr,
    duration: (lottie.op - lottie.ip) / lottie.fr,
    layers: lottie.layers.length,
    completedAt: new Date().toISOString(),
  };
  fs.writeFileSync(
    `${output}.provenance.json`,
    JSON.stringify(record, null, 2) + "\n",
  );
  console.log(JSON.stringify(record));
} finally {
  child.stdin.end();
  child.kill();
  // This directory is made by this invocation and only holds its temporary document.
  fs.rmSync(workspace, { recursive: true, force: true });
}
