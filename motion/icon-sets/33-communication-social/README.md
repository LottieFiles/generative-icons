# Communication & social

Source set `33-communication-social` in the Communication & social gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-communication-social`.

## Icons

- Inbox (`inbox`)
- Address book (`address-book`)
- Voicemail (`voicemail`)
- Transfer call (`transfer-call`)
- Dial pad (`dial-pad`)
- Emoji picker (`emoji-picker`)
- Thumbs up / down (`thumbs-up-down`)
- RSS feed (`rss-feed`)
- Announcement (`announcement`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/33-communication-social
node motion/icon-sets/33-communication-social/verify.mjs
node motion/icon-sets/33-communication-social/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/33/assets/set.json)
