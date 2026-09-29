# Knowledge and retrieval

Source set `08-ai-knowledge` in the AI · Knowledge gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-ai-knowledge`.

## Icons

- Knowledge base (`knowledge-base`)
- Semantic search (`semantic-search`)
- Citations (`citations`)
- Embeddings (`embeddings`)
- Knowledge graph (`knowledge-graph`)
- Document chat (`document-chat`)
- Source grounding (`source-grounding`)
- Retrieve context (`retrieve-context`)
- Vector store (`vector-store`)
- Dataset (`dataset`)
- Classification (`classification`)
- Clustering (`clustering`)
- Entity extraction (`entity-extraction`)
- Fact check (`fact-check`)
- Source rank (`source-rank`)
- Index data (`index-data`)
- Deduplicate (`deduplicate`)
- Data cleaning (`data-cleaning`)
- Context attach (`context-attach`)
- Long-term memory (`long-term-memory`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/08-ai-knowledge
node motion/icon-sets/08-ai-knowledge/verify.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/08/assets/set.json)
