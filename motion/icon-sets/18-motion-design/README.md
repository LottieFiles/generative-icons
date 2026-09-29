# Motion design

Source set `18-motion-design` in the Motion design gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-motion-design`.

## Icons

- Timeline (`timeline`)
- Add keyframe (`add-keyframe`)
- Remove keyframe (`remove-keyframe`)
- Auto keyframe (`auto-keyframe`)
- Keyframe easing (`keyframe-easing`)
- Graph editor (`graph-editor`)
- Bezier handles (`bezier-handles`)
- Motion path (`motion-path`)
- Anchor point (`anchor-point`)
- Layer stack (`layer-stack`)
- Parent layers (`parent-layers`)
- Precompose (`precompose`)
- Trim layer (`trim-layer`)
- Split layer (`split-layer`)
- Time stretch (`time-stretch`)
- Onion skin (`onion-skin`)
- Shape morph (`shape-morph`)
- Mask path (`mask-path`)
- Track matte (`track-matte`)
- Render queue (`render-queue`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/18-motion-design
node motion/icon-sets/18-motion-design/verify.mjs
node motion/icon-sets/18-motion-design/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/18/assets/set.json)
