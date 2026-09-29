# Agent operations

Source set `19-agent-operations` in the AI · Agent operations gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-agent-operations`.

## Icons

- Agent skills (`agent-skills`)
- Tool discovery (`tool-discovery`)
- MCP tools (`mcp-tools`)
- Browser agent (`browser-agent`)
- Terminal agent (`terminal-agent`)
- Sandbox (`agent-sandbox`)
- Agent permissions (`agent-permissions`)
- Human input (`human-input`)
- Agent checkpoint (`agent-checkpoint`)
- Resume run (`resume-run`)
- Retry step (`retry-step`)
- Execution trace (`execution-trace`)
- Parallel tools (`parallel-tools`)
- Merge results (`merge-results`)
- Background run (`background-run`)
- Scheduled agent (`scheduled-agent`)
- Event trigger (`event-trigger`)
- Context compaction (`context-compaction`)
- Agent artifact (`agent-artifact`)
- Budget limit (`budget-limit`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/19-agent-operations
node motion/icon-sets/19-agent-operations/verify.mjs
node motion/icon-sets/19-agent-operations/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/19/assets/set.json)
