# Video editing

Source set `53-video-editing`. This set contains 20 icons and 70 native actions. Native render checks use `/icons/53/verify.html?save=1` on the verification server. Catalog registration is outside this repair.

## Icons

- Cut clip (`cut-clip`)
- Trim clip (`trim-clip`)
- Transition (`transition`)
- Speed ramp (`speed-ramp`)
- Reverse clip (`reverse-clip`)
- Rotate clip (`rotate-clip`)
- Green screen (`green-screen`)
- Stabilize (`stabilize`)
- Thumbnail (`thumbnail`)
- Export video (`export-video`)
- Resolution (`resolution`)
- Aspect ratio (`aspect-ratio`)
- Mute track (`mute-track`)
- Voiceover (`voiceover`)
- Screen record (`screen-record`)
- Clapperboard (`clapperboard`)
- Merge clips (`merge-clips`)
- Add caption (`add-caption`)
- Video filter (`video-filter`)
- Render progress (`render-progress`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/preflight.mjs motion/icon-sets/53-video-editing
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/53-video-editing
node motion/icon-sets/53-video-editing/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest. The verifier checks native provenance, styles, stroke weights, state boundaries, rigid motion, and drawing contacts and clearances in both directions.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/53/assets/set.json)
