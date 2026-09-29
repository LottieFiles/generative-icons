# Files & transfers

Source set `24-file-transfer` in the Files & transfers gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-file-transfer`.

## Icons

- File upload (`file-upload`)
- File download (`file-download`)
- Attach file (`attach-file`)
- Detach file (`detach-file`)
- Cloud upload (`cloud-upload`)
- Cloud download (`cloud-download`)
- Drag and drop (`drag-and-drop`)
- Batch upload (`batch-upload`)
- Compress files (`compress-files`)
- Extract files (`extract-files`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/24-file-transfer
node motion/icon-sets/24-file-transfer/verify.mjs
node motion/icon-sets/24-file-transfer/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/24/assets/set.json)
