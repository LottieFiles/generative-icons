# Feedback & status

Source set `11-feedback-status` in the Feedback & status gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-feedback-status`.

## Icons

- Loading spinner (`loading-spinner`)
- Loading dots (`loading-dots`)
- Progress ring (`progress-ring`)
- Progress steps (`progress-steps`)
- Hourglass (`hourglass`)
- Queued (`queued`)
- Paused (`paused`)
- Cancelled (`cancelled`)
- Error (`error`)
- Warning (`warning`)
- Info (`info`)
- Help (`help`)
- No results (`no-results`)
- Empty state (`empty-state`)
- Offline (`offline`)
- Reconnect (`reconnect`)
- Cloud sync (`cloud-sync`)
- Saved (`saved`)
- Notification dot (`notification-dot`)
- Session expired (`session-expired`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/11-feedback-status
node motion/icon-sets/11-feedback-status/verify.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/11/assets/set.json)
