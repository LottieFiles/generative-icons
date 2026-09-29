import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { exporterConfig } from './exporter-config.mjs';

test('legacy root resolves the executable without passing a retired variable to it', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'exporter-config-'));
  try {
    const binary = path.join(root, 'target/debug/creator-mcp');
    fs.mkdirSync(path.dirname(binary), { recursive: true });
    fs.writeFileSync(binary, '#!/bin/sh\n[ -z "${ENGINE_MCP_ROOT+x}" ] || exit 9\n');
    fs.chmodSync(binary, 0o755);
    const input = { ENGINE_MCP_ROOT: root, CREATOR_MCP_LOG: 'warn' };
    const result = exporterConfig(input);
    assert.equal(result.binary, binary);
    assert.equal(result.env.CREATOR_MCP_LOG, 'warn');
    assert.equal(input.ENGINE_MCP_ROOT, root);
    assert.equal(spawnSync(result.binary, [], { env: result.env, cwd: result.root }).status, 0);
    assert.equal(exporterConfig({ ...input, CREATOR_MCP_BIN: process.execPath }).binary, process.execPath);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test('missing, relative, non-executable and directory paths fail before launch', () => {
  assert.throws(() => exporterConfig({}), /Set CREATOR_MCP_BIN/);
  assert.throws(() => exporterConfig({ CREATOR_MCP_BIN: './creator-mcp' }), /absolute/);
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'exporter-invalid-'));
  try {
    for (const file of [root, path.join(root, 'missing')])
      assert.throws(() => exporterConfig({ CREATOR_MCP_BIN: file }), /not executable/);
    const file = path.join(root, 'plain');
    fs.writeFileSync(file, 'not executable', { mode: 0o644 });
    assert.throws(() => exporterConfig({ CREATOR_MCP_BIN: file }), /not executable/);
  } finally { fs.rmSync(root, { recursive: true, force: true }); }
});

test('export CLI validates arguments before reading input files', () => {
  const result = spawnSync(process.execPath, [new URL('./export.mjs', import.meta.url).pathname], { encoding: 'utf8' });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Usage: node motion\/src\/export.mjs/);
  assert.doesNotMatch(result.stderr, /TypeError/);
});
