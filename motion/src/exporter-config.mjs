import fs from 'node:fs';
import path from 'node:path';

// ENGINE_MCP_ROOT is a repository-side compatibility alias. New exporter
// binaries reject it in their environment, so consume it here, not in the child.
export function exporterConfig(env = process.env) {
  const configured = env.CREATOR_MCP_BIN || (env.ENGINE_MCP_ROOT &&
    path.join(env.ENGINE_MCP_ROOT, 'target/debug/creator-mcp'));
  if (!configured) throw Error('Set CREATOR_MCP_BIN to a compatible exporter executable. See docs/development.md.');
  if (!path.isAbsolute(configured)) throw Error('The exporter path must be absolute. Set CREATOR_MCP_BIN.');
  const binary = path.resolve(configured);
  try {
    if (!fs.statSync(binary).isFile()) throw Error('not a file');
    fs.accessSync(binary, fs.constants.X_OK);
  } catch { throw Error(`Exporter is missing or not executable: ${binary}`); }
  const childEnv = { ...env };
  delete childEnv.ENGINE_MCP_ROOT;
  return { binary, root: path.dirname(binary), env: childEnv };
}
