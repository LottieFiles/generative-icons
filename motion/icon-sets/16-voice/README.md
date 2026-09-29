# Voice

Source set `16-voice` in the Voice gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-voice`.

## Icons

- Listening (`voice-listening`)
- Speaking (`voice-speaking`)
- Voice activity (`voice-activity`)
- Voice recording (`voice-recording`)
- Push to talk (`push-to-talk`)
- Microphone muted (`microphone-muted`)
- Wake word (`wake-word`)
- Switch voice (`switch-voice`)
- Voice isolation (`voice-isolation`)
- Voice speed (`voice-speed`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/16-voice
node motion/icon-sets/16-voice/verify.mjs
node motion/icon-sets/16-voice/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/16/assets/set.json)
