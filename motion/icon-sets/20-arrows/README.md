# Arrows

Source set `20-arrows` in the Arrows gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-arrows`.

## Icons

- Arrow right (`arrow-right`)
- Arrow left (`arrow-left`)
- Arrow up (`arrow-up`)
- Arrow down (`arrow-down`)
- Arrow up right (`arrow-up-right`)
- Arrow up left (`arrow-up-left`)
- Arrow down right (`arrow-down-right`)
- Arrow down left (`arrow-down-left`)
- Chevron right (`chevron-right`)
- Chevron left (`chevron-left`)
- Chevron up (`chevron-up`)
- Chevron down (`chevron-down`)
- Horizontal arrows (`arrows-horizontal`)
- Vertical arrows (`arrows-vertical`)
- Swap arrows (`arrows-swap`)
- Turn right (`arrow-turn-right`)
- Turn left (`arrow-turn-left`)
- U-turn (`arrow-u-turn`)
- Expand arrows (`arrows-expand`)
- Collapse arrows (`arrows-collapse`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/20-arrows
node motion/icon-sets/20-arrows/verify.mjs
node motion/icon-sets/20-arrows/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/20/assets/set.json)
