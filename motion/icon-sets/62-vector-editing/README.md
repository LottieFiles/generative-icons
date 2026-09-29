# Vector editing & design

Source set `62-vector-editing` in the Vector editing & design category. Exactly 20 generic icons. Native render review is available at `/icons/62/verify.html?save=1` on the local verification server. Catalog registration is outside this set's authoring scope.

## Icons

- Boolean union (`boolean-union`) — explicit reverse
- Boolean subtract (`boolean-subtract`) — explicit reverse
- Boolean intersect (`boolean-intersect`) — explicit reverse
- Boolean exclude (`boolean-exclude`) — explicit reverse
- Outline stroke (`outline-stroke`) — explicit reverse
- Offset path (`offset-path`)
- Join paths (`join-paths`) — explicit reverse
- Simplify path (`simplify-path`) — explicit reverse
- Stroke dash (`stroke-dash`)
- Auto layout (`auto-layout`) — explicit reverse
- Component instance (`component-instance`)
- Detach instance (`detach-instance`) — explicit reverse
- Pin constraints (`pin-constraints`)
- Artboard frame (`artboard-frame`)
- Export slice (`export-slice`)
- Vectorize image (`vectorize-image`) — explicit reverse
- Blend steps (`blend-steps`)
- Gradient stops (`gradient-stops`)
- Convert to curves (`convert-to-curves`)
- Path direction (`path-direction`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/preflight.mjs motion/icon-sets/62-vector-editing
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/62-vector-editing
node motion/icon-sets/62-vector-editing/verify.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest. Each icon has Light and Strong motion on a 32-unit grid, round caps and joins, Outline, Solid and Duotone. The ten stateful icons retain their results and provide authored reverse actions, for 60 native actions in total.

`verify.mjs` also runs `verify-geometry.mjs`. This checks exported endpoint contacts, pin contacts, offset clearance, layout padding, blend spacing and gradient-stop spacing across every action frame. The browser verifier checks native rendering, color and style overrides, profile rest equality, reverse boundaries and playback.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/62/assets/set.json)
