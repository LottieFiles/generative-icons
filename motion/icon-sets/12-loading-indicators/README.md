# Loading indicators

Source set `12-loading-indicators` in the Loading indicators gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-loading-indicators`.

## Icons

- Comet ring (`orbit-comet`)
- Counter arcs (`counter-arcs`)
- Wave dots (`wave-dots`)
- Ripple (`ripple-rings`)
- Corner relay (`corner-relay`)
- Infinity runner (`infinity-run`)
- Bead relay (`bead-relay`)
- Equalizer (`equalizer-bars`)
- Bead track (`bead-track`)
- Planet orbit (`planet-orbit`)
- Square snake (`square-snake`)
- Sync arrows (`sync-arrows`)
- Pinwheel (`pinwheel`)
- Bounce ball (`bounce-ball`)
- Travel wave (`travel-wave`)
- Sweep hand (`sweep-hand`)
- Helix (`helix-strands`)
- Atom (`atom-rings`)
- Morph ring (`morph-ring`)
- Stack cycle (`stack-cycle`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/12-loading-indicators
node motion/icon-sets/12-loading-indicators/verify.mjs
node motion/icon-sets/12-loading-indicators/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/12/assets/set.json)
