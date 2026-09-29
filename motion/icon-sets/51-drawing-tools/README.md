# Drawing & design tools

Source set `51-drawing-tools` in the Drawing & design tools gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-drawing-tools`.

## Icons

- Pen tool (`pen-tool`)
- Brush (`brush`)
- Eraser (`eraser`)
- Corner radius (`corner-radius`)
- Line tool (`line-tool`)
- Rectangle tool (`rectangle-tool`)
- Ellipse tool (`ellipse-tool`)
- Polygon tool (`polygon-tool`)
- Fill bucket (`fill-bucket`)
- Eyedropper (`eyedropper`)
- Group objects (`group-objects`)
- Align objects (`align-objects`)
- Magic wand (`magic-wand`)
- Lasso select (`lasso-select`)
- Transform tool (`transform-tool`)
- Rotate tool (`rotate-tool`)
- Mirror tool (`mirror-tool`)
- Snap to grid (`snap-to-grid`)
- Ruler guides (`ruler-guides`)
- Scissors (`scissors`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/51-drawing-tools
node motion/icon-sets/51-drawing-tools/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/51/assets/set.json)
