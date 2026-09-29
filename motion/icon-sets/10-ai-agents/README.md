# Agents and model operations

Source set `10-ai-agents` in the AI · Agents gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-ai-agents`.

## Icons

- AI agent (`ai-agent`)
- Agent team (`agent-team`)
- Tool call (`tool-call`)
- Agent workflow (`agent-workflow`)
- Task plan (`task-plan`)
- Agent handoff (`agent-handoff`)
- Approval step (`approval-step`)
- Guardrails (`guardrails`)
- Model router (`model-router`)
- Model training (`model-training`)
- Fine-tune (`fine-tune`)
- Model evaluation (`model-evaluation`)
- Confidence score (`confidence-score`)
- Inference (`inference`)
- Batch inference (`batch-inference`)
- Local model (`local-model`)
- Model deploy (`model-deploy`)
- Model version (`model-version`)
- Usage meter (`usage-meter`)
- AI connection (`ai-connection`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/10-ai-agents
node motion/icon-sets/10-ai-agents/verify.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/10/assets/set.json)
