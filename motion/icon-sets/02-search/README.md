# Search

Source set `02-search` in the Essentials gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-essentials`.

## Icons

- Search (`search`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/02-search/build.mjs
node motion/icon-sets/02-search/verify.mjs
```

This is a legacy set: geometry and motion are defined inline in `build.mjs`, so source preflight cannot screen it. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/02/assets/set.json)
