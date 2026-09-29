# Production and verification

Run commands from the repository root and confirm paths against the checkout. Use [polish-review.md](https://github.com/LottieFiles/lottie-generative-icons/blob/main/.agents/skills/animated-icon-workflow/references/polish-review.md) for acceptance criteria and the per-icon record. Derive the expected icons and forward/reverse/reset actions from the current catalog and set metadata, never from a fixed count.

Follow [generation-harness.md](https://github.com/LottieFiles/lottie-generative-icons/blob/main/.agents/skills/animated-icon-workflow/references/generation-harness.md) before export. For sets with `definitions.mjs`, build with `node motion/icon-sets/library/build-checked.mjs motion/icon-sets/SET-ID`: it runs source preflight, then that set's `build.mjs`. A clean source report does not replace any native or delivery gate.

## Source layout

Every set lives in `motion/icon-sets/NN-slug/` and has its own `README.md` with its icon list and rebuild commands.

| Sets | Authoring | Build / verify |
| --- | --- | --- |
| `01-heart-bell`, `02-search` | Geometry inline in the set `build.mjs` (legacy; preflight cannot screen it) | `node motion/icon-sets/SET-ID/build.mjs`, then `verify.mjs` |
| `03-essentials` to `06-tools` | `definitions.mjs` plus the matching `library/directed-*.mjs` helper | Checked build, then `verify.mjs` |
| `07` to `10` (AI) | Set `batches/` and `definitions.mjs` with the shared `library/build-set.mjs` | `npm --prefix motion run build:ai` / `verify:ai` |
| `11` onward | Set `batches/` and `definitions.mjs`, set-owned `build.mjs` | Checked build, then `verify.mjs` and `verify-clearance.mjs` when present |

Some sets also have named shortcuts in `motion/package.json` (for example `build:finance` / `verify:finance`); they call the same scripts.

`library/drawing.mjs` contains contour helpers; `library/direction.mjs` contains timing and result/reset helpers. Builders keep sampled scene points as an independent reference and export rigid parts as native transforms. `motion/src/export.mjs` runs the Engine exporter (`CREATOR_MCP_BIN`) in a separate workspace. If it is unavailable, stop at preflight; never substitute raster animation.

Use `npm --prefix motion run <command>`. The current set builders regenerate the set manifest and package all its records. **Do not use `ICON_NAMES` for a partial build without merging the full manifest**: it currently writes only selected definitions to `set.json`. Rebuild the complete affected set and confirm unaffected native asset hashes.

Use `npm run icons:new -- <slug> --name "Display name"` to scaffold a new set with its source/build/package/verification files. Supply unique icon names and custom geometry/motion. After a successful complete build, run `npm run icons:register -- <SET-ID>` to verify and register it. The verification server discovers numbered source sets at startup; restart it after scaffolding. Do not hardcode an icon or action count into a new acceptance test.

## Paint and styles

Pinned dotLottie Web is `0.80.0`. The project uses separate native stroke paints for Light/Regular/Bold, selected with opacity slots, because a scalar width override does not change rendered pixels in this player. Preserve the existing style contract unless a replacement is verified in the actual player.

Slots: `color.primary`, `style.outline.light`, `style.outline.regular`, `style.outline.bold`, `style.solid`, `style.tint`. Themeable colors remain solids. Duotone tint uses 18% only where a real enclosed surface exists. Compare the full action when that surface can leave the final frame; an open icon must match Outline at every frame. Do not add invisible dummy surfaces to satisfy a style-difference test.

For filled Outline marks, the weight is in the geometry. The More options check in `library/verify-filled-ellipsis.mjs` measures the exported dot diameters for each preset; native raster comparisons still must pass. Stroke-based icons retain the constant-width checks.

## Verification sequence

1. Snapshot affected current JSON, dotLottie, metadata, and pose images, plus hashes for untouched assets.
2. Build affected sets, run their `verify:*` scripts, and run `verify:paint` across the catalog. Source checks must compare actual exported geometry/paint with the intended constraints.
3. Use the available browser skill to inspect the real gallery. Do not control a second browser through an unrelated automation stack. Local technical verification pages are test tools, not additional product galleries.
4. Start `python3 motion/icon-sets/library/director-revision/serve-verification.py` (default port 4191; choose another with `--port`). It serves the prototype and saves bounded renderer reports and images. Leave the user's gallery server (`npm run dev`, port 4189) running, and stop only processes you started.
5. Open `http://127.0.0.1:4191/icons/NN/verify.html?icons=name1,name2` and run verification. Newly scaffolded pages need `?save=1` (or `&save=1`) to save evidence; older pages save only on port 4191. Failed reports are kept with a timestamp. Use `&batch=a` for disjoint groups in a harness that supplies `report.batch`; the collector then keeps each report under a separate suffix. Click Run verification and check status. These pages use the native player, save per-action progress, and yield with MessageChannel to avoid background animation-frame throttling. Check results/errors; do not infer completion from time elapsed.
6. Preserve a complete previous renderer report before a focused run overwrites `browser-verification.json`. `merge-render-evidence.py` currently considers named archived reports in each set's evidence folder. Merge only individually passing results whose package SHA matches the current asset. If coverage is missing, locate matching proof or render the missing actions; do not relabel old proof.
7. The collector saves pose/style/rest PNGs in both source `evidence/` and served `assets/`. After packaging, confirm both copies match. Optionally rebuild family review sheets with `python3 motion/icon-sets/library/director-revision/build-review-sheets.py` when their inputs change; it needs Pillow (`python3 -m pip install pillow`) and macOS system fonts.
8. Run `test:library` for catalog/state/controller changes, or a focused live interaction check for an asset-only correction. Keep new visual tests proportional to the actual risk.
9. Package the affected set after its final render proof, then run `package:library` and the affected category archive command where applicable. Verify ZIP integrity, catalog action coverage, every selected initial animation, embedded sibling JSON, per-action hashes, and actual HTTP download bytes. Include current shared authoring dependencies, instructions and review evidence in source archives.

Keep current evidence separate from failed attempts. Report source correctness, all-frame rendering, sampled visual review, live interaction and delivery bytes as separate evidence.

## Timing review and unchanged-profile proof

Inspect native playback at 1x, then capture Light/Strong at identical elapsed times from the same origin. Sample the meaningful drive and the entire action through its reset; include opacity-driven onset and short actions. A separate normalized strip for each profile can show geometry but cannot compare speed. Use nonuniform samples for symmetric rotating forms, and retain the all-frame checks.

`prototype/icons/01/clock-review.html` is an example equal-time capture page. Adapt its inputs to the current task rather than reusing a fixed icon list.

When only Strong changes, compare Light native geometry and active/rest metadata for all Light actions, including reverse/reset. Prefer unchanged animation bytes. A package may change because it embeds Strong; any visible-equivalence exception must state and verify why the changed Light data cannot alter the rendered action. Confirm the authoring helper's profile argument order before integration.

Keep actual-clock completion checks in an active browser tab. A failed aggregate stays failed. A fully completed, error-free individual action can contribute to merged evidence only if its current SHA and all required checks match; an interrupted action must be rerun. Keep expensive report thumbnails out of the frame-sweep layout without removing native assertions.

## Gallery verification

Reload the exact gallery after local edits and confirm the loaded module version before judging the change. A new tab can reuse old HTML or JavaScript. `?verifyPlayback=1` enables DOM frame counters for the native player; a running controller flag alone does not prove advancing frames. Timing metadata and browser-control latency are not measured frame-rate benchmarks.

Check whole-card hover/focus, profile switching during a loop, pointer exit, selected downloads and reduced motion. With Loop all active, navigate to a distant category and confirm its visible native frames advance. Offscreen and hidden-tab clocks pause; on entry they resume their phase. Stop loop must cancel both visible and paused work. Test a queued completion during suspension when changing the controller. Keep one owner for viewport and tab suspension; preserve one-at-a-time sequence review. The current gallery owns suspension and disables the player's separate automatic offscreen freeze.

Leave the gallery's minimal cards and category navigation in place. Technical frame counters and review sheets are verification tools, not new product controls. Instructions-only edits do not require an animation rebuild or another full native audit.

## Mechanism-specific checks

For intentionally discrete typing, verify the authored character states and a matching raster-state minimum. Do not require extra smooth frames or decorative movement. The AI verification has explicit minima for Image Caption and Code Generate, backed by independent source state counts.

The four AI sets build with `npm --prefix motion run build:ai` and verify with `npm --prefix motion run verify:ai`. The scope gate rejects missing or reordered icons. `package:ai` requires complete current native render evidence before it creates the AI-only archive. `prototype/icons/09/family-review.html` renders each AI set at native 24/32/48 px for visual checks; the product gallery stays at `icons/01/`.

Set-specific geometric gates live in each set's `verify-clearance.mjs`. Examples: finance checks the Bank door opening, contactless spacing and Portfolio handle attachments against sampled exported cubics expanded by stroke radius; travel compares attachment endpoints with moving housing curves and checks swept cranks and boards; commerce checks roll tangency, wheel counters, drawer and locker clearances and Duotone apertures. Financial values hold after posting; preview resets affect only changing parts. Fabric control points may flex while attachment endpoints stay fixed. Never accept image-estimated dimensions or a designer's arithmetic without checking the actual paths.

Run completion-sensitive browser batches sequentially in an active tab: MessageChannel keeps frame sampling reliable in background tabs, but real playback completion still depends on animation frames. A timeout is a failed attempt until a current rerun completes. Merge only individually completed records whose package hashes match, using `motion/icon-sets/library/director-revision/merge-render-evidence.py`.

When adding a catalog item, run the gallery's duplicate-ID check. UI SVG symbols must use their UI namespace and must not share an ID with an icon card or category anchor. Preserve current public icon anchors when resolving a UI-symbol collision.

### Native path assembly

Read the current authoring helper signatures before using options such as pose or visibility; a source schema pass does not prove the intended motion reached each part. For the current native Engine exporter, each part has one initial MoveTo. Split disconnected strokes into separately named parts instead of placing multiple MoveTo commands in one path. Use the verified winding helper only for fully contained cutouts. Inspect the source study and run the native export; preserve an observed exporter failure until the corrected representation passes.
