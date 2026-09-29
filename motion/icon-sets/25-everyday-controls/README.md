# Everyday controls

Source set `25-everyday-controls` in the Everyday controls gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-everyday-controls`.

## Icons

- Add / remove (`add-remove`)
- Close (`close`)
- Undo / redo (`undo-redo`)
- More options (`more-options`)
- External link (`external-link`)
- Share (`share`)
- Cut (`cut`)
- Paste (`paste`)
- Zoom in / out (`zoom-in-out`)
- Select all (`select-all`)
- Move tool (`move-tool`)
- Grid / list view (`grid-list-view`)
- Sidebar toggle (`sidebar-toggle`)
- Split view (`split-view`)
- Tab switcher (`tab-switcher`)
- Command menu (`command-menu`)
- Drag handle (`drag-handle`)
- Archive (`archive`)
- Target (`target`)
- Power (`power`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/25-everyday-controls
node motion/icon-sets/25-everyday-controls/verify.mjs
node motion/icon-sets/25-everyday-controls/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/25/assets/set.json)
