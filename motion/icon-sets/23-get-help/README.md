# Get help

Source set `23-get-help` in the Get help gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-get-help`.

## Icons

- Support agent (`support-agent`)
- Help center (`help-center`)
- Support ticket (`support-ticket`)
- Live support (`live-support`)
- Callback (`callback`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/23-get-help
node motion/icon-sets/23-get-help/verify.mjs
node motion/icon-sets/23-get-help/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/23/assets/set.json)
