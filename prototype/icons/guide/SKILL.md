---
name: animated-icon-workflow
description: Create consistent animated icons with a coding agent, editable vector source, native verification, and review for this project's native Lottie library. Use for new icons and drawing or motion revisions, including Light/Strong, strokes, and Duotone. Do not use for unrelated motion films or general interface work.
---

# Animated icon workflow

## Quickstart

This skill is for library sets in this repository. In an app, the `generative-icons` skill that `generative-icons init` installs covers the same loop for local icons.

Run everything from the repository root with the `generative-icons` CLI (build it with `cargo build --release -p generative-icons`). Replace `SET-ID` with the set directory, for example `39-food-drink`.

1. **Check tools:** `generative-icons doctor`. At the repository root its app checks fail because the repository is not an app; the cpu, quickjs, renderer, sign-in and engine checks are the ones that matter here.
2. **Search:** `generative-icons context "<meaning>" --search`, then rerun without `--search` and with `--neighbor <id>` (up to four) to get the reference packet.
3. **Brief:** per icon, write meaning, states, neighbors and a **cause → action → response → final state** plan for Light and Strong. Read [polish-review.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/polish-review.md) first.
4. **Author:** new set: `generative-icons library new-set <slug> --name "Display name"`. Draw and animate in `motion/icon-sets/SET-ID/definitions.mjs` and its `batches/`.
5. **Preflight (no sign-in needed):** `generative-icons library preflight SET-ID`
6. **Build and verify:** `generative-icons library build SET-ID`, then `generative-icons library verify SET-ID` and `generative-icons library verify --paint`.
7. **Inspect:** `generative-icons library verify-visual SET-ID` renders every action and writes review sheets; `generative-icons preview <id>` renders one icon's motions at chosen times. Review native playback in the local gallery (`npm run dev`, `/icons/01/`); see [production.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/production.md).
8. **Register:** `generative-icons library register SET-ID` after the first complete build and verification.
9. **Finish:** `npm test`, then open a pull request.

**Building needs the user's LottieFiles sign-in.** Export runs on LottieFiles' hosted engine, which is internal until launch. On `NOT_LOGGED_IN` or `SESSION_EXPIRED`, ask the user to run `generative-icons login` in their terminal; never sign in yourself. Meanwhile complete steps 1–5 and report the native gates as not run.

## Scope and roles

Apply this workflow to the requested scope and complete the authorized work without adding approval steps for routine drawing, animation, fixes or local review. The latest user direction wins over past accepted designs or these defaults. For a critique-only request, deliver findings and motion briefs without editing icons. For an explicitly static icon, apply the drawing and export checks without adding animation.

**One agent, separate stages.** One developer-chosen coding agent runs the whole loop: retrieve references → design → author → check → render and inspect → correct → deliver. No second model, supervisor or handoff is required; invoke another model only when the developer asks. Self-review is not independent approval, a passing check does not prove artistic quality, and a text-only agent must not claim it inspected an image. This is the canonical statement of these limits; the references do not repeat it.

## Where things live

- Gallery: `prototype/icons/01/index.html`, served by `npm run dev` on port 4189 at `/icons/01/`. All icons belong on that page; it starts each icon in **Strong** and Light stays selectable.
- Sources: `motion/icon-sets/SET-ID/`. Catalog: `prototype/icons/catalog.json`; each set is registered in a named browse category.
- Rules: [polish-review.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/polish-review.md) (rejection criteria and per-icon record), [generation-harness.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/generation-harness.md) (reference retrieval and preflight), [production.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/production.md) (build and evidence), `docs/design-principles.md`, and `motion/icon-sets/library/motion-directions.json` for existing motion intent.

## 1. Establish the change

`generative-icons library new-set` creates the set's source, config, README and review record but no artwork, and does not touch the catalog. Register only after a complete build so the catalog never references a missing manifest. Requirements and sign-in are in `docs/development.md`.

- Inspect the catalog, the affected set's definitions and builder, and the current page. Current source and rendered assets decide current behavior, not old critique text.
- Record which icons, profiles, styles, and states will change. Snapshot affected assets and record hashes before editing. Preserve unrelated work and accepted motion, unless the current request changes it.
- For a new icon, define its meaning and resting silhouette first. Check for an existing icon or mechanism before adding a duplicate. State reasonable assumptions and continue when the request is clear enough.
- Prepare a brief with the actual user request, applicable rules, relevant current geometry and motion, and protected assets. Select relevant prior failures by mechanism, including failures from other categories. Save the design and review findings before integration. For batches, write a distinct design and motion brief for each icon, then review the family together.

## 2. Draw the complete object

- Work on the current 32-unit vector grid. Keep caps and joins round; round actual corners unless recognition, geometry, or a supplied brand shape requires a sharp corner. Check the family together at 24, 32, and 48 px, including Bold.
- Use the shared Outline stroke presets: **1.5 / 2 / 2.5 grid units**. Do not vary widths per icon or scale a layer/group to fake heavier weight. Regular is the default. Solid is a separate drawing style, not the Bold outline preset.
- Check **visible weight**, not only stroke numbers. Narrow outlined columns can read as two adjacent strokes. Use a single centerline for simple line symbols such as Chart and Filter; otherwise adjust the drawing's spacing and openings while keeping the selected stroke width constant. Measure the rendered result when a mismatch is reported. Do not claim that equal source numbers settle a visual complaint.
- **Duotone fills only a real enclosed surface.** Open trays, menu lines, signal arcs, scan brackets, and separate chart/filter strokes have no area to tint. Do not invent a rectangle, wedge, disk, or background to make Duotone look different. With no enclosed surface, Duotone intentionally renders like Outline. Declare `duotoneFill: false` and test that equality. Retain the theme/style slots.
- Draw what motion will expose. Opening a lid requires a complete rim; a flap needs a hinge; a page needs the correct hidden and visible sections. No loose line ends, overlapping seams, collapsing counters, or background-colored patches. A transparent outline cannot hide another outline.
- Inspect arrow junctions as painted shapes. Keep the inner barb clear of a tight curve, using a straight approach and sufficient splay where needed. During a redraw, retract the head before its body and restore it only when the body reaches the tip.
- Match a Solid drawing to the intended painted silhouette. Reusing an Outline's centerline as a filled contour can make Solid smaller. Where a stroke joins a Solid surface, bury its cap inside that surface; tangent-only contacts can leave raster gaps. Keep Outline counters clear and inspect each style independently.

## 3. Direct the motion before keyframing

Write a short motion brief per icon: **cause → action → response → final state**. Review the fixed parts, driver, followers, pivots, and state policy before keyframing.

- Light is one clear, quiet action. **Strong must have a clearly stronger primary action at normal speed and at 24/32 px.** Give the main shape a larger useful travel range, a wider physical rotation, or a distinctly faster decisive stroke with visible preparation and response. A tiny follower, an extra pause, more keyframes or a longer timeline is insufficient. Do not obtain Strong by multiplying every transform.
- Compare the two profiles directly, then temporarily ignore their labels. If they are difficult to distinguish, revise before delivery. Measure the main part in real seconds: travel, rotation and speed can expose a weak pair, but metrics do not replace native visual review. A full check draw or symmetric turn may use stronger timing without changing its meaningful endpoint. A tiny movement cannot become Strong merely by playing faster.
- Reserve space for Strong when drawing the resting object. Redesign its proportions or opening if more travel would break a joint or close a counter. Check the swept Bold and Solid paint after increasing motion; preserve common endpoint states for profile switching. Direct true reverse actions too; preview resets can remain quiet where stronger motion would undo the meaning of a posted result.
- Match the object's mechanics: hardware stays rigid, paper feeds without stretching, cloth carries a travelling wave, ink follows the pencil, and a bow follows its lid. Use native transforms for rigid parts and compatible path topology for flexible parts.
- Design joints as shared geometry. A flap keeps its hinge, a drawer exposes guides between its moving front and fixed console, and a tape strip meets the roll tangent. Measure the full swept paint and the space inside counters at Bold and Solid widths. Thin supports or one-frame contact can still look detached.
- Read small symbols as a silhouette. A dial, seal and single hand can combine into an unintended power symbol. Keep a clock's two hands distinct, and avoid collinear details that merge at 24 px.
- When separate paints form one opaque surface, give them a small real overlap at the thinnest supported weight. Exactly tangent antialiased boundaries can leave a visible seam. Preserve actual negative-space counters.
- Carry velocity through ordinary same-direction waypoints. Do not give every key its own full ease-in and ease-out. Use a bounded shared tangent; preserve exact endpoints and explicit holds. For Strong, direct one primary gesture and a short damped response. Do not collect repeated micro-corrections at an open midpoint before the return.
- A physical impact can have an explicit velocity break; its follower starts from that contact. Data progress stays monotone. Check the main 10–90 percent traversal and normal-speed native pairs, not only peak speed or different asset hashes. Compare the whole active sequence; one fixed comparison frame can fall after a faster action has settled.
- Tune easing, stagger, and holds in real seconds. Avoid a common rise/hold/return pulse across unrelated icons. Secondary movement needs a cause. Use overshoot only where the mechanism supports it; data values must not overshoot.
- Retain meaningful results. Charging, writing, scanning, and advancing a clock should not undo themselves inside the forward action. Give the gallery a separate preview-reset or true reverse action. A symmetric index can finish at the next equivalent position without unwinding.

A whole-library review must leave a per-icon ledger: first design proposal, design review findings, current motion intent, Light preservation result, native package hashes, rendered review and any retained screening flags. A source audit alone does not complete the rendered review.

## 4. Author and review a complete cycle

Read [references/production.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/production.md) for the current build and evidence procedure. Export only through `generative-icons library build`, which exports on the hosted engine. Do not claim one tool produced an asset when another did.

Integrate the reviewed design. If a check or render exposes a flaw, record the diagnostic or observed frame, revise the affected geometry or motion, and rerun the affected checks in the same agent session. Stop an unproductive loop when the same failure repeats without new evidence; change the approach or report the unresolved issue. Missing export or visual inspection tools leave those gates incomplete. Keep one writer per production file.

Review Light and Strong at normal speed, then inspect preparation, peak, contact, settle, hold, and reset frames. Compare styles, weights, and small-size silhouettes. Fix the full action and all dependent representations, not only the selected still. For a local correction, limit rebuilds and new rendering checks to affected assets while checking that other asset hashes remain unchanged.

Check partial-opacity joins as well as opaque holds. Overlapping strokes can make a darker dot when each fades separately. Use a supported common-opacity group or an equivalent joined vector contour, then verify constant visible width, transparent output, and the exported geometry. Do not add background patches to hide the overlap.

### Continuous indicators

For an indeterminate loader or active voice indicator, author a complete periodic cycle without the gesture family's end hold. Declare `continuous: true` on each applicable icon; a category can also contain gestures and retained states. Use set-owned export support for this mixed metadata and native player looping while hover, focus, or Loop all remains active. Finish on the next cycle boundary when the pointer leaves; touch remains one cycle and reduced motion stays at a readable static pose. Test profile switches and stale loop listeners.

Validate position and velocity at the analytic cycle endpoint, then render the playable frames and compare the wrap against ordinary adjacent-frame motion. Check for unwanted still runs. A part can recycle only inside a verified zero-opacity interval; the whole indicator must stay visible. Do not force exact first/last playable-frame equality because adjacent frames contain different phase positions. Keep the old still-hold checks for gesture and state icons. Light and Strong may use different readable phase poses when they are continuous loaders.

## 5. Pass the relevant gates

- Source/export: provenance, native vector shapes, finite points, compatible topology, correct pivots and attachments, fixed geometry, state boundaries, and still holds.
- Paint: `generative-icons library verify --paint`. Check the entire current catalog for shared stroke values, round caps/joins, transform scaling, and the declared Duotone policy. Add a new open icon to the explicit policy check when applicable.
- Native renderer: `generative-icons library verify-visual SET-ID` covers every frame of changed actions at 24/32/48/192 px, all three styles, three outline weights, both themes, and actual playback completion. A no-fill Duotone must equal Outline. Keep exact holds; apply a documented small rendered-boundary tolerance only to declared symmetric cycles.
- Human-facing review: inspect the same gallery, including the reported problem. For weight complaints, compare straight sections at the same size/weight and inspect the small-size result.
- Interaction: whole-card hover and keyboard focus loop, Light/Strong switching while hovered keeps playback, Loop all covers every icon, touch plays once, leaving finishes the cycle, downloads select the active profile, and reduced motion remains still. Switching reduced motion off must re-enable every preview control without starting playback. Run the existing controller tests when catalog/state behavior changes; do not add tests that merely mirror new wording.

Do not treat a successful export, nonempty frame, or screenshot alone as complete delivery proof. If a check fails, correct the cause and rerun the affected check. Disclose unresolved limitations rather than label partial work complete.

## 6. Deliver and retain the lesson

Update the existing catalog/page and downloads. Match final evidence to current package hashes; source evidence and served pose images must agree. Save concise before/after evidence for visual corrections. Complete the per-icon record from `polish-review.md`; link existing reports instead of duplicating them. Promote verified reusable findings into `docs/design-principles.md` or the polish checklist; keep per-icon findings with the affected source. Correct superseded rules and linked prompts together. Do not invent a new lesson when the existing rule was sufficient.

Finish with the gallery location of the changed icons, concrete changes, checks that passed or were not run, and any remaining limitation. Contribute through a GitHub pull request; maintainers review the rendered output and merge.

### Strong timing acceptance

After changing interpolation, compare the meaningful main stroke in seconds. Smooth curves alone do not guarantee strong attack. Review preparation, contact, drive and settling separately, then inspect native Light/Strong captures at identical playback times. Use normalized key-pose strips only for geometry. Preserve Light active durations and rest frames during Strong-only changes. A whole-library release review must link every icon to current native package proofs and its equal-time visual review.

Check time to the first visible change, including opacity-only icons. Do not allocate the opening timing slot to a part that is already in its completed state. Remove unintended dead intervals and let adjacent progress confirmations overlap without reversing their values. Confirm the authoring helper's Light/Strong argument order before applying proposed edits.

For a large gallery, separate global loop intent from visible rendering. Use one owner for viewport and tab suspension; preserve the paused phase and prevent queued completions from restarting hidden clocks. Verify Loop all after navigating to a distant category, then check that Stop loop cancels all pending cards. Keep verification pages unfrozen so all required frames are rendered.
