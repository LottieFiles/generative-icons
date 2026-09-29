# Conversation and reasoning

Source set `07-ai-conversation` in the AI · Conversation gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-ai-conversation`.

## Icons

- AI assistant (`ai-assistant`)
- Prompt (`prompt`)
- System instructions (`system-instructions`)
- Context window (`context-window`)
- Response stream (`response-stream`)
- Autocomplete (`autocomplete`)
- Rewrite (`rewrite`)
- Summarize (`summarize`)
- Translate (`translate`)
- Brainstorm (`brainstorm`)
- Reasoning (`reasoning`)
- Conversation branch (`conversation-branch`)
- Prompt template (`prompt-template`)
- Tone control (`tone-control`)
- Answer compare (`answer-compare`)
- Follow-up (`follow-up`)
- Intent detection (`intent-detection`)
- Response length (`response-length`)
- Conversation memory (`conversation-memory`)
- Structured output (`structured-output`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/07-ai-conversation
node motion/icon-sets/07-ai-conversation/verify.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/07/assets/set.json)
