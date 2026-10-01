# Production and verification

Run commands from the repository root with the `gen-animated-icons` CLI and confirm paths against the checkout. Use [polish-review.md](polish-review.md) for acceptance criteria and the per-icon record. Derive the expected icons and forward/reverse/reset actions from the current catalog and set metadata, never from a fixed count.

Follow [generation-harness.md](generation-harness.md) before export. `gen-animated-icons library build SET-ID` runs source preflight, then exports every action on LottieFiles' hosted engine, post-processes the native files, and packages the set: `.lottie` files, `set.json`, `evidence/build.json` and the set zip. A failing preflight stops it before anything is exported. A clean source report does not replace any native or delivery gate.

## Source layout

Every set lives in `motion/icon-sets/NN-slug/` and has its own `README.md` with its icon list and rebuild commands.

| Sets | Authoring | Build / verify |
| --- | --- | --- |
| `01-heart-bell`, `02-search` | Geometry inline in the set `build.mjs` (legacy; preflight cannot screen it) | Frozen: never rebuilt; `library verify` checks the committed assets |
| `03-essentials` to `06-tools` | `definitions.mjs` plus the matching `library/directed-*.mjs` helper | `library build` (built-in legacy profiles), then `library verify` |
| `07` to `10` (AI) | Set `batches/` and `definitions.mjs` with the shared builder settings | `library build`, then `library verify` |
| `11` onward | Set `batches/`, `definitions.mjs` and `set-config.json`; set-specific rules in `verify.rules.json` or `geometry-contract.json` where present | `library build`, then `library verify` |

The per-set `build.mjs`, `verify.mjs`, `verify-clearance.mjs` and `package.py` files, and the `build:*` and `verify:*` scripts in `motion/package.json`, are the legacy pipeline. The CLI reproduces their outputs byte for byte and its parity tests still run them; do not use them for new work.

`library/drawing.mjs` contains contour helpers; `library/direction.mjs` contains timing and result/reset helpers. Builds keep sampled scene points as an independent reference and export rigid parts as native transforms. Export needs the user's LottieFiles sign-in; if it is missing, stop at preflight and report the native gates as not run. Never substitute raster animation.

Build complete sets: a set build regenerates the set manifest and packages all its records. Rebuild the complete affected set and confirm that unaffected native asset hashes are unchanged.

Use `gen-animated-icons library new-set <slug> --name "Display name"` to scaffold a new set. Supply unique icon names and custom geometry/motion. After a successful complete build and verification, run `gen-animated-icons library register SET-ID` to check and register it. Do not hardcode an icon or action count into a new acceptance test.

## Paint and styles

Players and the CLI's renderer use the dotLottie core that dotLottie Web `0.80.0` ships. The project uses separate native stroke paints for Light/Regular/Bold, selected with opacity slots, because a scalar width override does not change rendered pixels in this player. Preserve the existing style contract unless a replacement is verified in the actual player.

Slots: `color.primary`, `style.outline.light`, `style.outline.regular`, `style.outline.bold`, `style.solid`, `style.tint`. Themeable colors remain solids. Duotone tint uses 18% only where a real enclosed surface exists. Compare the full action when that surface can leave the final frame; an open icon must match Outline at every frame. Do not add invisible dummy surfaces to satisfy a style-difference test.

For filled Outline marks, the weight is in the geometry. The More options check measures the exported dot diameters for each preset; native raster comparisons still must pass. Stroke-based icons retain the constant-width checks.

## Verification sequence

1. Snapshot affected current JSON, dotLottie, metadata, and pose images, plus hashes for untouched assets.
2. Build affected sets with `library build`, run `library verify SET-ID` for each, and run `library verify --paint` across the catalog. Source checks must compare actual exported geometry/paint with the intended constraints.
3. Run `gen-animated-icons library verify-visual SET-ID` (`--icons name1,name2` for a focused run). It renders every action in a child process with the players' renderer and writes `evidence/browser-verification.json` plus rest, style and pose sheets to both the set's `evidence/` and its served `assets/`. A focused run keeps the earlier records of the other icons. Check results and errors; a failed aggregate stays failed.
4. Use the available browser skill to inspect the real gallery (`npm run dev`, `/icons/01/`). Do not control a second browser through an unrelated automation stack. Leave the user's gallery server running, and stop only processes you started.
5. Look at the sheets, and at `gen-animated-icons preview <id> --times …` for the moments that matter. Feed observed flaws into the next correction.
6. Run `npm test` for catalog/state/controller changes, or a focused live interaction check for an asset-only correction. Keep new visual tests proportional to the actual risk.
7. Package the affected set after its final render proof with `gen-animated-icons library package SET-ID`, then refresh the aggregate download with `gen-animated-icons library package --icon-library`. Verify ZIP integrity, catalog action coverage, every selected initial animation, embedded sibling JSON, per-action hashes, and actual HTTP download bytes. Include current shared authoring dependencies, instructions and review evidence in source archives.

Keep current evidence separate from failed attempts. Report source correctness, all-frame rendering, sampled visual review, live interaction and delivery bytes as separate evidence.

## Timing review and unchanged-profile proof

Inspect native playback at 1x, then compare Light/Strong at identical elapsed times from the same origin (`preview` puts Light and Strong in the same columns). Sample the meaningful drive and the entire action through its reset; include opacity-driven onset and short actions. A separate normalized strip for each profile can show geometry but cannot compare speed. Use nonuniform samples for symmetric rotating forms, and retain the all-frame checks.

When only Strong changes, compare Light native geometry and active/rest metadata for all Light actions, including reverse/reset. Prefer unchanged animation bytes. A package may change because it embeds Strong; any visible-equivalence exception must state and verify why the changed Light data cannot alter the rendered action. Confirm the authoring helper's profile argument order before integration.

A fully completed, error-free individual action can contribute to merged evidence only if its current SHA and all required checks match; an interrupted action must be rerun.

## Gallery verification

Reload the exact gallery after local edits and confirm the loaded module version before judging the change. A new tab can reuse old HTML or JavaScript. `?verifyPlayback=1` enables DOM frame counters for the native player; a running controller flag alone does not prove advancing frames. Timing metadata and browser-control latency are not measured frame-rate benchmarks.

Check whole-card hover/focus, profile switching during a loop, pointer exit, selected downloads and reduced motion. With Loop all active, navigate to a distant category and confirm its visible native frames advance. Offscreen and hidden-tab clocks pause; on entry they resume their phase. Stop loop must cancel both visible and paused work. Test a queued completion during suspension when changing the controller. Keep one owner for viewport and tab suspension; preserve one-at-a-time sequence review. The current gallery owns suspension and disables the player's separate automatic offscreen freeze.

Leave the gallery's minimal cards and category navigation in place. Technical frame counters and review sheets are verification tools, not new product controls. Instructions-only edits do not require an animation rebuild or another full native audit.

## Mechanism-specific checks

For intentionally discrete typing, verify the authored character states and a matching raster-state minimum. Do not require extra smooth frames or decorative movement. The AI verification has explicit minima for Image Caption and Code Generate, backed by independent source state counts.

The four AI sets (07–10) also pass an AI-scope gate that rejects missing or reordered icons; `library build` and `library verify` run it.

Set-specific geometric gates live in each set's rules (`verify.rules.json`, `geometry-contract.json`) or in the CLI's built-in rules, and `library verify` runs them. Examples: finance checks the Bank door opening, contactless spacing and Portfolio handle attachments against sampled exported cubics expanded by stroke radius; travel compares attachment endpoints with moving housing curves and checks swept cranks and boards; commerce checks roll tangency, wheel counters, drawer and locker clearances and Duotone apertures. Financial values hold after posting; preview resets affect only changing parts. Fabric control points may flex while attachment endpoints stay fixed. Never accept image-estimated dimensions or a designer's arithmetic without checking the actual paths.

When adding a catalog item, run the gallery's duplicate-ID check. UI SVG symbols must use their UI namespace and must not share an ID with an icon card or category anchor. Preserve current public icon anchors when resolving a UI-symbol collision.

### Native path assembly

Read the current authoring helper signatures before using options such as pose or visibility; a source schema pass does not prove the intended motion reached each part. For the engine's Lottie export, each part has one initial MoveTo. Split disconnected strokes into separately named parts instead of placing multiple MoveTo commands in one path. Use the verified winding helper only for fully contained cutouts. Inspect the source study and run the native export; preserve an observed export failure until the corrected representation passes.
