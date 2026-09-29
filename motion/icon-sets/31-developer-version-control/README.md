# Developer & version control

Source set `31-developer-version-control` in the Developer & version control gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-developer-version-control`.

## Icons

- Repository (`repository`)
- Git branch (`git-branch`)
- Git commit (`git-commit`)
- Git merge (`git-merge`)
- Pull request (`pull-request`)
- Repository fork (`repository-fork`)
- Merge conflict (`merge-conflict`)
- Git stash (`git-stash`)
- Debug (`debug`)
- Breakpoint (`breakpoint`)
- Debug step (`debug-step`)
- Regular expression (`regular-expression`)
- Webhook (`webhook`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/31-developer-version-control
node motion/icon-sets/31-developer-version-control/verify.mjs
node motion/icon-sets/31-developer-version-control/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/31/assets/set.json)
