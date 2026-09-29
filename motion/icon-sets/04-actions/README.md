# Ten everyday actions

Source set `04-actions` in the Actions gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-actions`.

## Icons

- Download (`download`)
- Upload (`upload`)
- Copy (`copy`)
- Trash (`trash`)
- Edit (`edit`)
- Send (`send`)
- Bookmark (`bookmark`)
- Refresh (`refresh`)
- Eye (`eye`)
- Menu (`menu`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/04-actions
node motion/icon-sets/04-actions/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/04/assets/set.json)
