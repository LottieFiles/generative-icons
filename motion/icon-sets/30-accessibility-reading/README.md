# Accessibility & reading

Source set `30-accessibility-reading` in the Accessibility & reading gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-accessibility-reading`.

## Icons

- Accessibility (`accessibility`)
- Wheelchair access (`wheelchair-access`)
- Hearing assistance (`hearing-assistance`)
- High contrast (`high-contrast`)
- Reduced motion (`reduced-motion`)
- Text size (`text-size`)
- Keyboard (`keyboard`)
- Keyboard shortcuts (`keyboard-shortcuts`)
- Focus indicator (`focus-indicator`)
- Reading mode (`reading-mode`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/30-accessibility-reading
node motion/icon-sets/30-accessibility-reading/verify.mjs
node motion/icon-sets/30-accessibility-reading/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/30/assets/set.json)
