# Text & document editing

Source set `29-text-document-editing` in the Text & document editing gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-text-document-editing`.

## Icons

- Bold (`bold`)
- Italic (`italic`)
- Underline (`underline`)
- Strikethrough (`strikethrough`)
- Heading (`heading`)
- Paragraph (`paragraph`)
- Text alignment (`text-alignment`)
- Bulleted list (`bulleted-list`)
- Numbered list (`numbered-list`)
- Indent / outdent (`indent-outdent`)
- Horizontal rule (`horizontal-rule`)
- Text color (`text-color`)
- Text highlight (`text-highlight`)
- Clear formatting (`clear-formatting`)
- Spell check (`spell-check`)
- Find and replace (`find-and-replace`)
- Page break (`page-break`)
- Word wrap (`word-wrap`)
- Letter case (`letter-case`)
- Line spacing (`line-spacing`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/29-text-document-editing
node motion/icon-sets/29-text-document-editing/verify.mjs
node motion/icon-sets/29-text-document-editing/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/29/assets/set.json)
