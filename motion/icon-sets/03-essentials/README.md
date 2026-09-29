# Five useful actions

Source set `03-essentials` in the Essentials gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-essentials`.

## Icons

- Star (`star`)
- Check (`check`)
- Sliders (`sliders`)
- Lock (`lock`)
- Folder (`folder`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/03-essentials
node motion/icon-sets/03-essentials/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/03/assets/set.json)
