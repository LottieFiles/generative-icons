# Generation context and early source checks

Use this sequence to retrieve current family geometry and screen editable source before native export.

## Setup before design

Use `generative-icons doctor` to check the binary, its math, the renderer, the sign-in and the engine. Use `generative-icons library new-set <slug> --name "Display name"` for a new set. It supplies the source, config and review record without choosing geometry or motion. Keep an unbuilt set out of the catalog; use `generative-icons library register <SET-ID>` after the first complete build and verification. This preserves retrieval and preflight while the asset manifest does not yet exist.

## Before the design proposal

Search by shape, meaning and mechanism. Preserve existing public IDs. Decide whether the request needs a new icon, an existing icon, or a motion variant. Related names are candidates, not proof of equivalent meaning.

From the project root:

```bash
generative-icons context "package return" --search
generative-icons context "package return" --neighbor return-package --neighbor delivery-truck
```

The second command prints a reference packet. It contains the current family contract, candidate matches, selected icons, Light/Strong metadata, the initial native Outline geometry with its local transforms, and input hashes. It verifies the selected Strong package hash against the catalog and the loose animation against the packaged animation. A missing/stale reference fails rather than silently supplying old geometry. No generated assets are written; `--output <dir>` saves the packet, and `--prompt` prints a design prompt that embeds it.

The context command makes no model call; read the packet directly. Use repeatable `--neighbor ID` arguments to choose up to four relevant examples. Without explicit neighbors, it uses the top two lexical candidates with documented vocabulary expansions. Review the selection; a word match can choose the wrong mechanism. Check an exact existing ID or label before accepting automatic neighbors. Similar labels can rank incorrectly; choose explicit neighbors for ambiguous cases and do not treat directional opposites or modified labels as equivalent. A no-match result is valid and must not be disguised as an established family reference. For a large batch, split briefs by mechanism instead of supplying the entire library.

Read the relevant source excerpts too. The initial path snapshot is not the motion source and may show an incomplete starting state. Save `generation-context.json` and the design review findings beside the proposal.

The numeric contract is [family-contract.json](../../../../motion/icon-sets/library/family-contract.json). The prompt and preflight read the same file. It holds the 32-unit grid, 1.5/2/2.5 Outline weights, 18% eligible Duotone, Strong default, and existing timeline. The CLI's tests check its agreement with every current catalog entry. Versioned exporters keep their constants to preserve prior provenance; a future numeric change requires coordinated authoring/export/paint/render validation. This file is not permission to rescale existing artwork.

## Before native export

Write the reviewed design as a spec (`<id>.icon.json`) in a scratch project and screen it before any engine call:

```bash
generative-icons create picnic-basket --library <checkout>    # or: create <library-id> --replace
generative-icons check picnic-basket                          # offline: structure, names, motion, lints
generative-icons draft picnic-basket                          # one engine call: a review sheet
generative-icons build picnic-basket --no-install             # export, verify, player sheets
generative-icons library adopt 39-food-drink --add .generative-icons/build/picnic-basket
```

`check` errors stop `draft` and `build` before the engine; its warnings (`NUDGE`, `DUPLICATE_DRAWING`, `LIGHT_STRONG`, `CANVAS_MARGIN`, `SMALL_AT_24`, `SOLID_HIDDEN`) name a design problem to fix, not to silence. A set's JS batches (continuous loaders and the legacy sets) go into trusted local `definitions.mjs` sources instead:

```bash
generative-icons library preflight 15-commerce-delivery
generative-icons library build 15-commerce-delivery
```

`library build` completes source screening before it exports anything. A failing check prevents the export. The screen covers both profiles, forward/reverse actions, all three Outline widths, every frame including the analytic endpoint, authored key poses, active time and rest time.

Hard failures include malformed/nonfinite paths or poses, changing normalized path topology, moving rigid pivots, invalid timing, out-of-range opacity, inconsistent paint/fill declarations, invalid names, and duplicate icon IDs within a set. Diagnostics identify icon, part, profile, direction, weight and first failing time where applicable. Clamp a visibility driver to its valid range in its own authoring logic; do not relax validation because the renderer appears to clamp it. Do not clamp all geometric motion and lose intentional overshoot.

Warnings include ambiguous layer labels and a control-point hull crossing the canvas (the hull can overestimate a cubic's painted bounds; inspect actual swept paint before editing geometry). Missing legacy Duotone declarations and set-specific reverse naming are informational; existing native checks still apply.

```bash
generative-icons library preflight --all
```

The all-source command discloses inline legacy builders that cannot be screened without executing exports. Heart, Bell and Search currently use that route: retain their existing source/native checks. Do not import an inline `build.mjs` to obtain definitions.

## Contextual review and evidence

Keep required native checks at 24/32/48/192 px, all supported styles/weights/themes, and equal elapsed-time Light/Strong playback. Also inspect the changed icon beside the selected neighbors, in controls and beside text. Treat 16/20 px icons in 32/36 px controls as extra stress cases unless the task explicitly supports those icon sizes. Record what fails at an unsupported small size; do not silently shrink every icon or change public size support to pass it.

Assert actual render size before accepting size evidence. Record requested CSS size, measured CSS width/height, backing width/height and the explicit render pixel ratio. Assert backing size equals CSS size times that ratio; verify exported image dimensions too. A fixed staging CSS rule can override requested dimensions. Reject samples whose actual size differs from their labels and regenerate after correcting the staging rule. Never accept labels or constructor dimensions alone as size proof. Keep 1x and 2x results separate.

Optical mass and centering estimates can flag a review. Do not apply automatic median corrections or grid snapping to animated points.

Preflight is source screening only: painted contacts, closed Duotone surfaces, timing, continuous-loop velocity, player behavior and downloads are checked by the [production gates](production.md). Generation-context hashes record inputs; final review evidence must still refer to the current final packages.
