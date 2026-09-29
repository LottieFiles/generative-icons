# Annotation arrows

Source set `21-annotation-arrows` in the Annotation arrows gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-annotation-arrows`.

## Icons

- Curved pointer right (`annotation-curve-right`)
- Curved pointer left (`annotation-curve-left`)
- S-curve pointer (`annotation-s-curve`)
- Swoop up (`annotation-swoop-up`)
- Swoop down (`annotation-swoop-down`)
- Curled pointer (`annotation-loop`)
- Elbow callout (`annotation-elbow`)
- Bracket callout (`annotation-bracket`)
- Callout pointer (`annotation-callout`)
- Split connector (`annotation-split`)
- Merge connector (`annotation-merge`)
- Curved double pointer (`annotation-double-ended`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/21-annotation-arrows
node motion/icon-sets/21-annotation-arrows/verify.mjs
node motion/icon-sets/21-annotation-arrows/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/21/assets/set.json)
