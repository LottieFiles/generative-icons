# Generative Animated Icons — AI operating guide

Give this file to your coding agent to create or revise icons that match the Generative Animated Icons family by LottieFiles. It bundles the canonical workflow, rules and contract from the source repository into one file.

Repository: https://github.com/LottieFiles/gen-animated-icons

## Quickstart

In an app, `npx gen-animated-icons init` installs the gen-animated-icons agent skill, which covers creating a missing icon in the app itself. This guide is the full family workflow used for the library's own sets; give it to your agent for deeper reference.

1. Search existing icons: `gen-animated-icons search "<meaning>"`, then `gen-animated-icons context "<meaning>"` for the reference packet.
2. Fill in the task brief below and follow the workflow in the first source section.
3. In an app: `gen-animated-icons create <id>`, then `check`, `build` and `preview`. In the library repository: `gen-animated-icons library new-set`, `preflight`, `build`, `verify`, `verify-visual` and `register`.

Building exports native Lottie on LottieFiles' hosted engine, so it needs a LottieFiles sign-in (`gen-animated-icons login`). The hosted engine is internal until launch.

## Task brief

- Icon name and meaning:
- Product context and user action:
- Existing neighbor IDs (search first):
- Initial, completed and reverse/reset states:
- Details to preserve or avoid:
- Animated Light/Strong pair, or explicitly static only:

The sections below are copied from the maintained guides; their paths and hashes are in sources.json. Relative links point to the source repository.


---

## Source: .agents/skills/animated-icon-workflow/SKILL.md

---
name: animated-icon-workflow
description: Create consistent animated icons with a coding agent, editable vector source, native verification, and review for this project's native Lottie library. Use for new icons and drawing or motion revisions, including Light/Strong, strokes, and Duotone. Do not use for unrelated motion films or general interface work.
---

# Animated icon workflow

## Quickstart

This skill is for library sets in this repository. In an app, the `gen-animated-icons` skill that `gen-animated-icons init` installs covers the same loop for local icons.

Run everything from the repository root with the `gen-animated-icons` CLI (build it with `cargo build --release -p gen-animated-icons`). Replace `SET-ID` with the set directory, for example `39-food-drink`.

1. **Check tools:** `gen-animated-icons doctor`. At the repository root its app checks fail because the repository is not an app; the cpu, quickjs, renderer, sign-in and engine checks are the ones that matter here.
2. **Search:** `gen-animated-icons context "<meaning>" --search`, then rerun without `--search` and with `--neighbor <id>` (up to four) to get the reference packet.
3. **Brief:** per icon, write meaning, states, neighbors and a **cause → action → response → final state** plan for Light and Strong. Read [polish-review.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/polish-review.md) first.
4. **Author:** new set: `gen-animated-icons library new-set <slug> --name "Display name"`. Draw and animate in `motion/icon-sets/SET-ID/definitions.mjs` and its `batches/`.
5. **Preflight (no sign-in needed):** `gen-animated-icons library preflight SET-ID`
6. **Build and verify:** `gen-animated-icons library build SET-ID`, then `gen-animated-icons library verify SET-ID` and `gen-animated-icons library verify --paint`.
7. **Inspect:** `gen-animated-icons library verify-visual SET-ID` renders every action and writes review sheets; `gen-animated-icons preview <id>` renders one icon's motions at chosen times. Review native playback in the local gallery (`npm run dev`, `/icons/01/`); see [production.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/production.md).
8. **Register:** `gen-animated-icons library register SET-ID` after the first complete build and verification.
9. **Finish:** `npm test`, then open a pull request.

**Building needs the user's LottieFiles sign-in.** Export runs on LottieFiles' hosted engine, which is internal until launch. On `NOT_LOGGED_IN` or `SESSION_EXPIRED`, ask the user to run `gen-animated-icons login` in their terminal; never sign in yourself. Meanwhile complete steps 1–5 and report the native gates as not run.

## Scope and roles

Apply this workflow to the requested scope and complete the authorized work without adding approval steps for routine drawing, animation, fixes or local review. The latest user direction wins over past accepted designs or these defaults. For a critique-only request, deliver findings and motion briefs without editing icons. For an explicitly static icon, apply the drawing and export checks without adding animation.

**One agent, separate stages.** One developer-chosen coding agent runs the whole loop: retrieve references → design → author → check → render and inspect → correct → deliver. No second model, supervisor or handoff is required; invoke another model only when the developer asks. Self-review is not independent approval, a passing check does not prove artistic quality, and a text-only agent must not claim it inspected an image. This is the canonical statement of these limits; the references do not repeat it.

## Where things live

- Gallery: `prototype/icons/01/index.html`, served by `npm run dev` on port 4189 at `/icons/01/`. All icons belong on that page; it starts each icon in **Strong** and Light stays selectable.
- Sources: `motion/icon-sets/SET-ID/`. Catalog: `prototype/icons/catalog.json`; each set is registered in a named browse category.
- Rules: [polish-review.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/polish-review.md) (rejection criteria and per-icon record), [generation-harness.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/generation-harness.md) (reference retrieval and preflight), [production.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/production.md) (build and evidence), `docs/design-principles.md`, and `motion/icon-sets/library/motion-directions.json` for existing motion intent.

## 1. Establish the change

`gen-animated-icons library new-set` creates the set's source, config, README and review record but no artwork, and does not touch the catalog. Register only after a complete build so the catalog never references a missing manifest. Requirements and sign-in are in `docs/development.md`.

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

Read [references/production.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/production.md) for the current build and evidence procedure. Export only through `gen-animated-icons library build`, which exports on the hosted engine. Do not claim one tool produced an asset when another did.

Integrate the reviewed design. If a check or render exposes a flaw, record the diagnostic or observed frame, revise the affected geometry or motion, and rerun the affected checks in the same agent session. Stop an unproductive loop when the same failure repeats without new evidence; change the approach or report the unresolved issue. Missing export or visual inspection tools leave those gates incomplete. Keep one writer per production file.

Review Light and Strong at normal speed, then inspect preparation, peak, contact, settle, hold, and reset frames. Compare styles, weights, and small-size silhouettes. Fix the full action and all dependent representations, not only the selected still. For a local correction, limit rebuilds and new rendering checks to affected assets while checking that other asset hashes remain unchanged.

Check partial-opacity joins as well as opaque holds. Overlapping strokes can make a darker dot when each fades separately. Use a supported common-opacity group or an equivalent joined vector contour, then verify constant visible width, transparent output, and the exported geometry. Do not add background patches to hide the overlap.

### Continuous indicators

For an indeterminate loader or active voice indicator, author a complete periodic cycle without the gesture family's end hold. Declare `continuous: true` on each applicable icon; a category can also contain gestures and retained states. Use set-owned export support for this mixed metadata and native player looping while hover, focus, or Loop all remains active. Finish on the next cycle boundary when the pointer leaves; touch remains one cycle and reduced motion stays at a readable static pose. Test profile switches and stale loop listeners.

Validate position and velocity at the analytic cycle endpoint, then render the playable frames and compare the wrap against ordinary adjacent-frame motion. Check for unwanted still runs. A part can recycle only inside a verified zero-opacity interval; the whole indicator must stay visible. Do not force exact first/last playable-frame equality because adjacent frames contain different phase positions. Keep the old still-hold checks for gesture and state icons. Light and Strong may use different readable phase poses when they are continuous loaders.

## 5. Pass the relevant gates

- Source/export: provenance, native vector shapes, finite points, compatible topology, correct pivots and attachments, fixed geometry, state boundaries, and still holds.
- Paint: `gen-animated-icons library verify --paint`. Check the entire current catalog for shared stroke values, round caps/joins, transform scaling, and the declared Duotone policy. Add a new open icon to the explicit policy check when applicable.
- Native renderer: `gen-animated-icons library verify-visual SET-ID` covers every frame of changed actions at 24/32/48/192 px, all three styles, three outline weights, both themes, and actual playback completion. A no-fill Duotone must equal Outline. Keep exact holds; apply a documented small rendered-boundary tolerance only to declared symmetric cycles.
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


---

## Source: .agents/skills/animated-icon-workflow/references/polish-review.md

# Icon polish review

Read this before briefing or reviewing a new icon or a drawing/motion revision. Apply only the checks relevant to the requested change. These are working instructions for this icon family; the latest user brief takes precedence. They do not prescribe the motion of unrelated films or product heroes.

Use `SKILL.md` for family geometry and workflow, `production.md` for commands, and `docs/design-principles.md` at the project root for the maintained construction rules. Past amplitudes, counts, tolerances and accepted versions are examples, not permanent targets. Inspect current source and native output before reusing a technique.

## Before drawing

Name the icon's meaning, trigger, material, primary moving part, fixed parts, attachments, protected openings and resulting state. Select relevant earlier failures by mechanism, not just category: a finance drawer and a delivery locker share a clearance problem; a progress ring and typed output share onset and state questions.

Include those applicable lessons in the brief. Create a drawable proposal and separate Light/Strong plans with travel in grid units, rotation in degrees, and event times in seconds. Verify numerical estimates against source checks and actual rendered output.

Choose the state policy before animation:

| Policy | Required behavior |
| --- | --- |
| Gesture | One complete action and a readable rest. A physical response may settle after contact. |
| State or output | Retain the result. A separate reverse or preview-reset action rearms the demo; fixed housing remains visible. |
| Continuous indicator (loading or active voice) | A periodic native cycle without the gesture's end hold. Show a meaningful still pose for reduced motion. |

A category can mix these policies. Select and verify the policy per icon; do not give every icon in a voice or feedback set a loop without a hold.

## Reject construction that fails in motion

| Failure to look for | Required correction and check |
| --- | --- |
| Round caps but sharp or hooked contours | Author curvature in the path. Inspect inner and outer turns separately in Solid and Bold, including compressed poses. Clamp a changing corner radius to its available width and height. |
| Equal stroke values but unequal visible weight | Compare native 24/32/48 px neighbors. Use centerlines for simple bars, improve spacing or optical balance, and retain the shared stroke presets. Do not scale stroked layers to compensate. |
| A lid, handle or shackle exposes loose terminals | Draw the exposed rim, guides and contact surfaces. Attach endpoints to the actual moving boundary; include stroke radius. Inspect outward movement, peak, contact and return. |
| A clear path counter disappears in the raster | Inspect the complete paint stack. An outline or transparent hole cannot hide other paint. Use explicit visible geometry, supported masks or real openings; no background-colored patches. |
| A nested stroke closes its own inner channel | Test non-adjacent segments within the same path, including round caps and return curves. Layer-to-layer gap checks cannot detect every paperclip or spiral counter. Check the swept Bold and Solid paint, then inspect the native small-size result; keep intentional continuous joints distinct from free openings. |
| A Solid detail vanishes, or a joined fill has a seam | Give Solid its own complete painted contour. Keep a closed lid visible. Use real overlap where paints form one opaque surface, while retaining genuine counters. Verify partial-opacity joins too. |
| Duotone paints an open tray, doorway or gap | Tint only real enclosed surfaces. Check the entire action: a surface may exist during motion and disappear at rest. A fully open icon must match Outline throughout. |
| An arrow barb merges into a curve or leaves a dot | Check the complete painted head against the curve and stem, including the curve-to-outlet endpoint; a clear barb endpoint does not prove that the barb segment is clear. Give the head a clear approach. Coordinate head/body withdrawal and reveal; explicitly hide withdrawn zero-length round-cap paths. |
| Parts drift, deform rigid hardware or jump topology | Share the physical driver and derive contacts from actual curves. Use native rigid transforms; use compatible cubic topology for flexible parts. A revealed route and its traveller use the same advancing path position. |

Measure the full swept paint, including cubic extrema, caps and the widest supported stroke. A positive mathematical gap is insufficient if it reads as a sliver at 24 px. Reserve space for Strong in the resting drawing; adjust the construction or pivot when needed. Do not weaken a clearance test to fit a proposal. A justified renderer-specific tolerance must remain local and retain an independent check of the intended opening.

## Strong acceptance

Strong must change the **primary action** visibly at normal speed and at 24/32 px. Accept useful extra travel, coherent rotation, or a more decisive complete stroke with readable preparation and response. A meaningful full check draw or index turn can use the same endpoint with stronger timing. Do not require a universal amplitude ratio, spring, or duration.

Reject a pair whose difference is only a tiny follower, extra keyframes, extra holds, a longer file, heavier paint, or a speed multiplier on an otherwise tiny action. Compare without relying on the profile labels. For a batch, review adjacent icons together: shared geometry must not turn into the same rise/hold/return performance on every object.

Direct one main action with a bounded response. Use spring settling where the object supports it: suspension, a flexible handle, a needle or a landing. Keep rigid hardware rigid and displayed progress monotone. Do not add repeated bounces or several small stops to make Strong seem complex.

Carry velocity through ordinary same-direction waypoints. Preserve deliberate holds, direction changes and physical impacts. A contact may break velocity; its follower must start from that contact. If interpolation makes the action smoother but weaker, retune the main drive rather than accepting the curve alone.

Inspect the first visible change, including opacity. Do not spend the first timing slot animating an already-complete part. Overlap adjacent confirmations when their meaning permits it. Discrete typing and output boundaries can retain short holds; do not add unrelated smooth movement to satisfy a frame-count test.

For gaps, subtract the painted half-width of both neighboring parts. Check separately authored Solid geometry too; its stroke envelope may be wider than Bold. Mirroring an arrow must not mirror a numeral or change its reading direction.

For a drawn annotation, make the dependencies explicit: the supporting route reaches its target before the head paints; a branch starts after its parent reaches the junction. A preview reset follows the reverse order. Hide a zero-length revealed path to suppress its round-cap dot. Keep the completed annotation as the retained result. A flexible curve can respond while its endpoints remain fixed, but its control points must preserve the tangent where it meets a straight approach.

When a transparent counter grows from zero area, preserve its explicit closing edge and winding through normalization. Verify the zero state and both directions before export. A reversed contour must stay inside its filled surface; it does not perform a Boolean subtraction outside that surface.

## Review the actual timing and boundaries

- Play both native profiles at 1x, then inspect identical elapsed times from the same start. Include onset, preparation, main drive, contact, settle, hold and reset. Use enough samples to catch a short action; use uneven samples when symmetry aliases regular snapshots. Normalized key-pose columns establish geometry, not comparative speed.
- Measure the meaningful drive in real seconds, including its 10–90 percent traversal when useful. Inspect separate axes or phases when an aggregate hides the actual movement. Peak speed, different hashes and a long cumulative path do not prove stronger motion.
- Resolve every relevant pause or slow-travel flag: revise an accidental stop, or record the exact part/time and why the boundary or final damping is intentional.
- Check resets for a second unwanted follower gesture, blank housing or erased product state. Preserve common semantic endpoints across Light/Strong and their true reverses. For a Strong-only change, verify Light geometry, active duration and rest metadata, including reset/reverse actions. A shared package hash can change while its Light animation is unchanged; document any visible-equivalence proof.
- Interrupt a preview reset with Stop loop and reduced motion, including a reset that finishes offscreen before Stop loop. The stopped picture must retain the completed output, not a temporary rearm state. Compare the actual stopped raster with the completed gallery pose; stopped clocks alone do not prove a useful still icon. Keep genuine reverse state actions distinct from demo-only resets.
- An icon can use filled marks in Outline, such as an ellipsis made of three disks. Test the actual exported dimensions for every weight and verify different native rasters. Keep the same theme/style slots. Do not add invisible stroke paints to satisfy a stroke-only audit; retain the stroke audit for icons that use centerlines.
- Compare stroke-weight presets at a pose that contains strokes. A preview reset can correctly finish at a filled source disk, where the stroke presets have no visible effect. Keep the full-action paint checks, and use the complete start drawing for the separate reset style comparison; do not add fake strokes to satisfy a rest-frame-only test.
- For continuous motion, check analytic wrap position and velocity, plus playable adjacent-frame motion. Directional phase must keep its intended sign; deliberate counter-rotation is valid. Recycle only inside a verified zero-opacity interval. Do not demand identical first/last playable frames or insert a rest to hide a seam.

## Evidence required before delivery

Use a compact per-icon row or record beside the task's proposal. Reuse an existing ledger where available. Record:

```text
Icon / source set / requested profiles and states:
Current source and affected action IDs:
Applicable earlier lessons and any local exception:
Design proposal and integration acceptance or revision:
Meaning / material / driver / followers / attachments / protected gaps:
Light versus Strong: travel, rotation, onset, drive, contact, settle:
Native 1x and equal-time review: findings and current capture paths:
Source, paint and renderer checks: result and exact package hashes:
Remaining screening flags: part, time, decision and evidence:
Protected motion: unchanged bytes or explicit geometry/timing proof:
Gallery and served-download checks:
New reusable lesson, or no new lesson:
Status: proposed / implemented / verified; user acceptance separately:
```

Determine action coverage from the current catalog and metadata, including sibling reverse/reset actions. Every changed action needs current native proof; unchanged action proof can be reused only when its exact package hash matches. Keep all-frame technical checks, sampled visual review, actual playback completion and HTTP download verification separate. Preserve failed attempts and rerun incomplete checks. Do not lower assertions to obtain a pass.

## Carry the lesson into the next icon

After a verified correction, record the observed fault, evidence-supported cause, change, applicable mechanism, check and limits in `docs/design-principles.md`. Distinguish measured behavior from a preference or an untested explanation.

Promote a finding into this reference or `SKILL.md` only when it changes a reusable decision or check. Keep exact one-icon dimensions and timing with that asset. Amend a rule that the evidence supersedes; do not keep appending conflicting defaults. Update linked prompts or production instructions in the same change.

For a recurring defect, improve the applicable independent export, paint, renderer or controller check so the next occurrence is detected. Verify the reported failure and the corrected behavior; avoid a test that only repeats the implementation's own calculation. Cosmetic wording changes do not require rebuilding animations.

For the next design, include the current instructions and select the relevant historical examples. Do not claim that the next icon is better merely because it has more rules or keyframes; compare its native drawing and movement against the existing family.

### Visible speed and hidden resets

Compare Light and Strong using visible motion. A fully transparent position wrap is an implementation reset and must not inflate peak-speed or travel measurements. Keep raw measurements for diagnosis, add visible-only values, and confirm the result with native equal-time and normal-speed playback. Do not hide collisions or weaken bounds checks by lowering opacity.

For small selected controls, reserve a real open well and draw the check as a separate path when a shrinking negative counter becomes unstable or unreadable. Keep path topology constant; future segments collapse at the current pen tip until reached. A zero-area counter is not a safe morph endpoint.

### Recognition and style lessons from sets 35–39

These were verified across Weather, Health & Fitness, Smart Home, Education and Food & Drink. In every case, a native styles sheet exposed a fault that the numeric gates had passed.

- **Ask what else the rest pose could read as.** About a quarter of the first drafts passed every gate yet read as another object: a clock face, a remote, a key, a warning sign, "!!", a checkbox, a plus sign or a device screen. Compare the rest pose with the catalog's existing meanings. After two failed redraws of a figure, switch to the object that carries the meaning (for example, a yoga mat instead of a pose).
- **A stateful prepared state still paints.** Keep a visible housing (a horizon, rim, shelf or card), or the native check reports an invisible action and a colour override with no visible effect.
- **Solid needs its own construction.** A container that holds moving detail is a Solid ring, not a filled disc. Stroke detail inside a filled Solid silhouette gets no Solid variant, because its caps poke out. Scores and seeds that must survive in Solid are real counters.
- **Small closed marks close at Bold.** Ring pellets, polygon bolts and 3-wide hollow capsules close up. Use centreline strokes or filled marks, and keep hollow widths at least the Bold stroke plus 1.5. A Duotone surface narrower than about 3.4 renders like Outline.
- **Write gap contracts only between parts that are meant to be separate.** A joint is not a gap. Measure insets between curves from sampled curve distance, not local slope.
- **Followers end inside the active time.** Window exponential decays and staggered fades so the rest is exactly still. Hide scaled glyph strokes by about 35% scale, and bring multi-part marks in rigidly with a fade rather than scaling from zero.


---

## Source: .agents/skills/animated-icon-workflow/references/generation-harness.md

# Generation context and early source checks

Use this sequence to retrieve current family geometry and screen editable source before native export.

## Setup before design

Use `gen-animated-icons doctor` to check the binary, its math, the renderer, the sign-in and the engine. Use `gen-animated-icons library new-set <slug> --name "Display name"` for a new set. It supplies the source, config and review record without choosing geometry or motion. Keep an unbuilt set out of the catalog; use `gen-animated-icons library register <SET-ID>` after the first complete build and verification. This preserves retrieval and preflight while the asset manifest does not yet exist.

## Before the design proposal

Search by shape, meaning and mechanism. Preserve existing public IDs. Decide whether the request needs a new icon, an existing icon, or a motion variant. Related names are candidates, not proof of equivalent meaning.

From the project root:

```bash
gen-animated-icons context "package return" --search
gen-animated-icons context "package return" --neighbor return-package --neighbor delivery-truck
```

The second command prints a reference packet. It contains the current family contract, candidate matches, selected icons, Light/Strong metadata, the initial native Outline geometry with its local transforms, and input hashes. It verifies the selected Strong package hash against the catalog and the loose animation against the packaged animation. A missing/stale reference fails rather than silently supplying old geometry. No generated assets are written; `--output <dir>` saves the packet, and `--prompt` prints a design prompt that embeds it.

The context command makes no model call; read the packet directly. Use repeatable `--neighbor ID` arguments to choose up to four relevant examples. Without explicit neighbors, it uses the top two lexical candidates with documented vocabulary expansions. Review the selection; a word match can choose the wrong mechanism. Check an exact existing ID or label before accepting automatic neighbors. Similar labels can rank incorrectly; choose explicit neighbors for ambiguous cases and do not treat directional opposites or modified labels as equivalent. A no-match result is valid and must not be disguised as an established family reference. For a large batch, split briefs by mechanism instead of supplying the entire library.

Read the relevant source excerpts too. The initial path snapshot is not the motion source and may show an incomplete starting state. Save `generation-context.json` and the design review findings beside the proposal.

The numeric contract is [family-contract.json](https://github.com/LottieFiles/gen-animated-icons/blob/main/motion/icon-sets/library/family-contract.json). The prompt and preflight read the same file. It holds the 32-unit grid, 1.5/2/2.5 Outline weights, 18% eligible Duotone, Strong default, and existing timeline. The CLI's tests check its agreement with every current catalog entry. Versioned exporters keep their constants to preserve prior provenance; a future numeric change requires coordinated authoring/export/paint/render validation. This file is not permission to rescale existing artwork.

## Before native export

Integrate the reviewed design into trusted local `definitions.mjs` sources. Then run:

```bash
gen-animated-icons library preflight 15-commerce-delivery
gen-animated-icons library build 15-commerce-delivery
```

`library build` completes source screening before it exports anything. A failing check prevents the export. The screen covers both profiles, forward/reverse actions, all three Outline widths, every frame including the analytic endpoint, authored key poses, active time and rest time.

Hard failures include malformed/nonfinite paths or poses, changing normalized path topology, moving rigid pivots, invalid timing, out-of-range opacity, inconsistent paint/fill declarations, invalid names, and duplicate icon IDs within a set. Diagnostics identify icon, part, profile, direction, weight and first failing time where applicable. Clamp a visibility driver to its valid range in its own authoring logic; do not relax validation because the renderer appears to clamp it. Do not clamp all geometric motion and lose intentional overshoot.

Warnings include ambiguous layer labels and a control-point hull crossing the canvas (the hull can overestimate a cubic's painted bounds; inspect actual swept paint before editing geometry). Missing legacy Duotone declarations and set-specific reverse naming are informational; existing native checks still apply.

```bash
gen-animated-icons library preflight --all
```

The all-source command discloses inline legacy builders that cannot be screened without executing exports. Heart, Bell and Search currently use that route: retain their existing source/native checks. Do not import an inline `build.mjs` to obtain definitions.

## Contextual review and evidence

Keep required native checks at 24/32/48/192 px, all supported styles/weights/themes, and equal elapsed-time Light/Strong playback. Also inspect the changed icon beside the selected neighbors, in controls and beside text. Treat 16/20 px icons in 32/36 px controls as extra stress cases unless the task explicitly supports those icon sizes. Record what fails at an unsupported small size; do not silently shrink every icon or change public size support to pass it.

Assert actual render size before accepting size evidence. Record requested CSS size, measured CSS width/height, backing width/height and the explicit render pixel ratio. Assert backing size equals CSS size times that ratio; verify exported image dimensions too. A fixed staging CSS rule can override requested dimensions. Reject samples whose actual size differs from their labels and regenerate after correcting the staging rule. Never accept labels or constructor dimensions alone as size proof. Keep 1x and 2x results separate.

Optical mass and centering estimates can flag a review. Do not apply automatic median corrections or grid snapping to animated points.

Preflight is source screening only: painted contacts, closed Duotone surfaces, timing, continuous-loop velocity, player behavior and downloads are checked by the [production gates](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/production.md). Generation-context hashes record inputs; final review evidence must still refer to the current final packages.


---

## Source: .agents/skills/animated-icon-workflow/references/production.md

# Production and verification

Run commands from the repository root with the `gen-animated-icons` CLI and confirm paths against the checkout. Use [polish-review.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/polish-review.md) for acceptance criteria and the per-icon record. Derive the expected icons and forward/reverse/reset actions from the current catalog and set metadata, never from a fixed count.

Follow [generation-harness.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/generation-harness.md) before export. `gen-animated-icons library build SET-ID` runs source preflight, then exports every action on LottieFiles' hosted engine, post-processes the native files, and packages the set: `.lottie` files, `set.json`, `evidence/build.json` and the set zip. A failing preflight stops it before anything is exported. A clean source report does not replace any native or delivery gate.

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


---

## Source: motion/icon-sets/library/family-contract.json

{
  "version": 1,
  "grid": 32,
  "outlineWeights": {
    "light": 1.5,
    "regular": 2,
    "bold": 2.5
  },
  "defaultOutlineWeight": "regular",
  "outlineWeightRange": [
    1,
    3
  ],
  "strokeWidthToken": "style.outline.width",
  "sourceStyles": [
    "outline",
    "solid",
    "tint"
  ],
  "presentationStyles": [
    "outline",
    "solid",
    "duotone"
  ],
  "duotoneOpacity": 0.18,
  "roundCapsAndJoins": true,
  "profiles": [
    "light",
    "strong"
  ],
  "defaultMotion": "strong",
  "fps": 60,
  "durationSeconds": 2.4,
  "review": {
    "requiredSizes": [
      24,
      32,
      48,
      192
    ],
    "contextStressSizes": [
      16,
      20
    ],
    "controlSizes": [
      32,
      36
    ],
    "themes": [
      "dark",
      "light"
    ],
    "compareAtEqualElapsedTime": true
  },
  "scope": "Current native icon family. Source preflight and design prompts consume this file. Existing exporters retain their versioned constants; export/paint checks must still confirm agreement. Do not quantize animated coordinates, enforce a competitor's grid, or infer visual approval from these values."
}


---

## Source: docs/using-packages.md

# Install and use the packages

Add animated icons to any JavaScript app: install only the icons you need with the `gen-animated-icons` CLI, and render them with the shared runtime or the React component.

| Package | Purpose |
| --- | --- |
| `gen-animated-icons` | The CLI: project setup, search, icon installation, and creating icons your app is missing |
| `@lottiefiles/generative-icons-runtime` | Playback, paint, state, reduced motion and lifecycle |
| `@lottiefiles/generative-icons-react` | React and Next.js component |
| `@lottiefiles/generative-icons` | Per-icon descriptors, native dotLottie assets and the catalog, for apps that import icons without the CLI |

Keep the `@lottiefiles/generative-icons*` packages on the same version.

## Install

```sh
npm install @lottiefiles/generative-icons-runtime
# React or Next.js:
npm install @lottiefiles/generative-icons-react

npx gen-animated-icons init
npx gen-animated-icons search heart
npx gen-animated-icons add heart
```

In a monorepo, pass `--cwd apps/web` to target the app.

`init` reads the app's package metadata and creates `gen-animated-icons.json`, the agent skill in `.claude/skills/gen-animated-icons/` and `.agents/skills/gen-animated-icons/`, and a copy of the authoring kit. It does not edit `AGENTS.md`, global agent settings or components. If your app uses a base URL, include it in `publicPath`.

`add` copies native files to `public/gen-animated-icons/` and descriptors to `src/gen-animated-icons/` (or `gen-animated-icons/` without a `src` folder), and records hashes in `.gen-animated-icons/lock.json`. Commit the config, assets, descriptors and lock. `diff` reports local changes, `add` refuses to overwrite modified managed files, and `remove` uninstalls icons whose files are unedited. A project set up by the earlier `lottie-icons` command moves over with `npx gen-animated-icons init --migrate`.

## Vanilla JavaScript

```js
import { createIcon } from '@lottiefiles/generative-icons-runtime';
import icon, { src } from './gen-animated-icons/heart.mjs';

const handle = await createIcon(document.querySelector('canvas'), {
  icon, src, color: '#008c76', style: 'outline', weight: 2, motion: 'light'
});
document.querySelector('button').addEventListener('click', () => handle.play());
// On removal: handle.destroy();
```

Without the CLI, install `@lottiefiles/generative-icons`, import a single descriptor from `@lottiefiles/generative-icons/icons/heart` and resolve `@lottiefiles/generative-icons/assets/heart.lottie` with your bundler. A descriptor import does not load the catalog or any asset bytes.

## React and Next.js

```jsx
'use client';
import { AnimatedIcon } from '@lottiefiles/generative-icons-react';
import icon, { src } from './gen-animated-icons/heart.mjs';

export function Favorite() {
  return <AnimatedIcon icon={icon} src={src} size={24} color="#008c76" />;
}
```

Components that handle events in Next.js need `'use client'`. The adapter loads the player after mount and destroys it on unmount; server output is a static canvas.

**Accessibility:** icons are decorative by default and hidden from assistive technology, so the parent button must provide the accessible name. Set `label` only when the icon conveys information without visible text.

## Appearance and behavior

| Setting | Values |
| --- | --- |
| Size | CSS pixels, default `32` |
| Color | Six-digit hex; resolve theme tokens before passing them |
| Style | `outline`, `solid`, `duotone` (React prop: `variant`) |
| Weight | Any value from `1` to `3` (outline only). `1.5`, `2` and `2.5` are the authored Light, Regular and Bold layers; other values render the Regular geometry at that stroke width. The files carry a `style.outline.width` token for it; until dotLottie players apply stroke-width tokens, the runtime writes the width into the file before loading it |
| Motion | `light`, `strong` |
| Theme | `light`, `dark` |
| Reduced motion | Follows the OS preference; override with a boolean |

- **State:** read `startState` and `endState` from the descriptor. Use `handle.setState(name)` or the React `state` prop to keep app state. `play('reverse')` runs the authored reverse action.
- **Preview:** `preview()` is for demos and may reset a stateful icon afterwards.
- **Reduced motion** keeps the chosen state without playback. Continuous icons stop after one cycle unless you start another.
- `handle.update()` changes paint and motion without reloading. `stop()` returns to the rest pose. Pass an `AbortSignal` while loading and call `destroy()` on cleanup.
- `wasmUrl` points to a self-hosted WASM file from the pinned dotLottie player.

**Shared defaults:** edit `defaults` in `gen-animated-icons.json` and run `add` again. Import `defaults` from the generated `gen-animated-icons/settings.mjs` and spread it into runtime options (map `style` to `variant` in React).

Runtime color settings are applied at playback and are not baked into the file. Downloaded native files keep their authored themes and style slots.

## iOS and Android

The same commands work in iOS and Android apps: `add` installs the `.lottie` file into the app's assets and a JSON descriptor into `gen-animated-icons/descriptors/`. Play the files with dotLottie for iOS or Android; the installed skill's runtime reference has SwiftUI, Compose and Views examples. Native players show the 1.5, 2 and 2.5 weights; `add --bake-weight <w>` bakes any other weight into the file.

## Missing an icon?

Create it with your coding AI in your own project (`gen-animated-icons create`, then `build`; building needs a LottieFiles sign-in), or [request it](https://github.com/LottieFiles/gen-animated-icons/issues/new?template=icon_request.yml). See [creating icons](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/creating-icons.md).


---

## Source: docs/package-contract.md

# Package contract

The `@lottiefiles/generative-icons*` packages share one version and are released together. Schema version 1 is the current contract. Any schema change needs an explicit compatibility decision and a new schema version; matching filenames do not imply compatibility.

| Surface | Contract |
| --- | --- |
| Icon descriptor | Stable `id`; readable `label` and `intent`; family and category; start and end state; Light/Strong actions and optional authored reverse; supported styles and authored weights (any weight from 1 to 3 renders through the `style.outline.width` token); native asset path, bytes and SHA-256 |
| Catalog | `schemaVersion`, package `version`, categories and the complete descriptor list |
| Project (`gen-animated-icons.json`, written by the CLI) | `schemaVersion`, platform, detected framework and package manager, relative asset and descriptor folders, public URL base (web), default appearance, agent skill folders and local icon settings. `schemas/project.schema.json` describes only the original web fields; it does not yet cover the newer fields, native configs or weights between 1 and 3. |
| Lock (`.gen-animated-icons/lock.json`) | Selected IDs and versions, installed paths and hashes of managed files; local edits block replacement |
| Release (`release.json`) | Source commit and dirty status, catalog/asset/docs/spec hashes, runtime, CLI and player versions, and the `exporter` note (export is needed for authoring only and is not bundled) |
| Specs (`specs/<id>.icon.json`) | The editable spec (`gai-icon/2`) of every icon reanimated as one, hashed in `release.json`; `gen-animated-icons create <new-id> --from <id>` starts a new icon from it. Icons without a spec have none. |

## How packages are built

`gen-animated-icons library packages` builds the catalog and assets from registered source sets. It rejects duplicate IDs, checks source package hashes and confirms the required animation IDs inside each native bundle. Each icon gets its own descriptor module. A Strong bundle contains both Light and Strong actions, plus reverse actions where applicable; the runtime selects the requested action. Native bundles are copied unchanged.

The runtime package ships the gallery's controller as its canonical implementation; the build copies that exact file and parity tests guard it. Apps should use the runtime package rather than copying the controller.

The website consumes the published packages and validates the release hashes when importing the catalog, assets and documentation. No engine code is included in the packages or the CLI: native export runs on LottieFiles' hosted engine.

Projects set up by the earlier `lottie-icons` command (`lottie-icons.json` and `.lottie-icons/lock.json`) keep working with `gen-animated-icons add`, `info`, `diff` and `remove`, which write the same bytes there; `gen-animated-icons init --migrate` moves them to the current files.


---

## Source: docs/development.md

# Development

This repository holds the icon library and the `gen-animated-icons` CLI, a Rust workspace in `crates/`. The CLI builds, checks and packages the whole catalog byte for byte the way the earlier JavaScript and Python pipeline did. That pipeline (`motion/`, `scripts/` and each set's `build.mjs`, `verify.mjs` and `package.py`) stays in the repository as legacy: the Rust parity tests run it as a reference, but new work goes through the CLI.

## Requirements

- **Rust** 1.89 or newer. CI lints and tests with Rust 1.93.1.
- **clang** for C and C++. The vendored QuickJS compiles V8's `ieee754.cc`, which must be built by clang; on Linux set `CC=clang CXX=clang++`. The renderer's bindings also need libclang (`libclang-dev` on Debian and Ubuntu).
- **`CXXFLAGS` with `-ffp-contract=off`.** The renderer must not fuse multiply-adds, or its pixels drift from the browser player's. `.cargo/config.toml` sets it, but a `CXXFLAGS` already in your environment replaces that value, so add the flag to it.
- **x86_64 needs FMA** (Intel Haswell, AMD Piledriver or newer). On Apple silicon, use the arm64 build: Rosetta has no FMA.
- **Node 22+ and Python 3** for the local gallery, the web packages and the repository checks, and for the legacy pipeline.

## Build the CLI

```sh
cargo build --release -p gen-animated-icons
target/release/gen-animated-icons --version
```

Put `target/release` on your `PATH`, or set `GEN_ANIMATED_ICONS_BINARY` to the binary so the npm shim runs it (`node npm/gen-animated-icons/bin/gen-animated-icons.js` in this checkout; `npx gen-animated-icons` once the package is published at launch). The [CLI reference](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/cli.md) lists every command.

## Test

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                                       # unit tests, plus the oracle tests whose inputs exist
cargo test --workspace --release -- --ignored --skip live_   # whole-catalog parity; takes several minutes
```

Oracle tests compare the CLI with committed files, or with the legacy tools run side by side, and skip when an input is missing:

- intermediate `motion/icon-sets/*/scenes/` and `native/` folders, which Git ignores;
- the built `packages/*/dist` folders (`npm run build:packages`);
- `node` with `motion/node_modules` (`npm ci --prefix motion`), and `python3`, for the side-by-side runs.

Tests never touch your sign-in. They use a temporary `GEN_ANIMATED_ICONS_HOME`, a fake keychain, or `GAI_ENGINE_TOKEN` with a local mock engine. Never run `login` or `logout` from a test or script. Only the two live tests use a real sign-in: `live_exports_match_the_committed_natives_except_path_names` (gai-engine) and `live_build_through_the_hosted_engine` (the CLI). They are ignored by default, and `--skip live_` keeps the whole-catalog run above from starting them. They export on the hosted engine with your session and refresh it, so run them only on purpose, after signing in yourself with `gen-animated-icons login`:

```sh
cargo test --release -p gai-engine --test live -- --ignored live_
cargo test --release -p gen-animated-icons --test engine -- --ignored live_
```

The repository's other checks still run on Node and Python:

```sh
npm ci --prefix motion   # dependencies of the legacy pipeline
npm test                 # gallery controller, generation harness and set scaffold tests
npm run check:docs       # links, publication hygiene, retired names and guide hashes
```

## Author and build a library set

New sets go through the CLI. Building exports on the hosted engine, so sign in first with `gen-animated-icons login`; the hosted engine is internal until launch.

```sh
gen-animated-icons library new-set landscaping --name "Landscaping"
# draw and animate in motion/icon-sets/NN-landscaping/definitions.mjs and batches/
gen-animated-icons library preflight NN-landscaping
gen-animated-icons library build NN-landscaping
gen-animated-icons library verify NN-landscaping
gen-animated-icons library verify-visual NN-landscaping
gen-animated-icons library register NN-landscaping
gen-animated-icons library package NN-landscaping   # repackage with the final evidence
gen-animated-icons library package --icon-library   # refresh icon-library.zip and delivery.json
```

- Registration needs a built, nonempty manifest, a passing preflight and passing verification. Registering a missing manifest would stop the catalog checks from running.
- Sets 01 (Heart/Bell) and 02 (Search) are frozen: their assets are verified as they are and never rebuilt. Sets 03–06 build through built-in legacy profiles.
- Builders replace a set's outputs. Build complete sets and confirm that unrelated asset hashes are unchanged. Do not rebuild only to preview existing files: `gen-animated-icons preview <id>` renders any icon.
- Intermediate `scenes/` and `native/` folders are rebuildable and ignored by Git; final assets under `prototype/icons/NN/assets/` are tracked.

The [creating icons](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/creating-icons.md) guide covers the design loop; [verification](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/verification.md) covers the checks.

## Reanimate a library icon with a spec

A library icon can be replaced by a spec v2 icon of the same id. Start it in any project with `create <id> --replace`, author and build it there, then adopt the build into its set:

```sh
gen-animated-icons create airplane --replace       # in a scratch project: the icon's spec, or its v1 drawing ported
# write the motion in gen-animated-icons/icons/airplane.icon.json
gen-animated-icons check airplane && gen-animated-icons draft airplane
gen-animated-icons build airplane --no-install
gen-animated-icons library adopt 14 path/to/project/.gen-animated-icons/build/airplane
gen-animated-icons library verify 14
gen-animated-icons library verify-visual 14
```

- `create --replace` keeps the library id. An icon already reanimated starts from its own spec. Any other starts from its v1 drawing, ported from the Light animation at frame 0: one part per outline layer, and the tint and Solid layers as the part's own where the geometry matches, else as parts that draw only in that style, so every style keeps v1's shapes. The port has no motion (`check` reports an empty `motion`). It carries rest opacity, Solid-only strokes, per-weight geometry and the reverse name, and its warnings name what it could not: geometry only another weight draws, a Solid stroke other than 2.6, an even-odd Solid fill, a paint with no style slot, a trim at frame 0, and keys or shape items the format lacks.
- `adopt` copies the spec to `motion/icon-sets/NN-…/specs/<id>.icon.json` and the actions into the set's assets, replaces the icon's `set.json` entry (keeping its number, category and state labels) and repackages the set. The build must have the icon's actions: the same reverse or reset actions for a stateful icon.
- The rest pose and, for a stateful icon, the end state must stay the library drawing, so apps that switch versions see no jump.
- From then on the spec is the icon's source. `library build` keeps adopted icons as they are (their old definitions stay for the legacy checks but no longer export), and `library verify` checks them against their spec: hashes, format, paint, themes and markers. Their motion checks run in `build`, and `verify-visual` renders them like every other action.
- Frozen sets 01 and 02 can adopt: they are still never rebuilt, so adopting only replaces the icon's assets and entry, and their legacy checks skip it.
- A new icon joins a set the same way with `library adopt NN --add <build>`: `create <new-id>` (not a library id), build it, then add it. It takes the set's category and the number after the library's last icon, and `library build` keeps it. New icons also change the catalog size that some tests pin (see [crates/README.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/crates/README.md)), so update those counts in the same commit.

What the library-wide reanimation (Oct 2026, 412 icons) learned about porting, beyond [authoring-v2.md](https://github.com/LottieFiles/gen-animated-icons/blob/main/crates/gen-animated-icons/assets/skill/references/authoring-v2.md):

- Decide first: keep an icon whose motion already is its meaning, with rhythm in Strong, readable at 24 px (a dial indexing to its next stop, a refresh arrow's full turn, a toggle knob landing). Reanimate generic nudges, lifts and tilts, and moves under a pixel at 24 px.
- Port the rest drawing exactly, vertex for vertex, and match the icon's Solid (`"solid": "fill"` or `"ring"`): the default Solid adds a rim the library drawing does not have.
- A stateful icon ends on the library's end state (compare the build's last frame with the old one) and keeps its animation ids: a reverse apps play under another name (`lock-unlock`, `reconnect-disconnect`, `suitcase-retract`, `menu-open`, `check-reset`) keeps it with `state.reverseName`, which `create --replace` fills in.
- A `previewReset` icon uses `"reverse": "fade"` with states `prepared` → `result`.
- Some sets protect openings (counters, ring holes, Solid openings) in every frame: read the set's entry in `crates/gai-render/src/rules.rs` before designing, so the motion never shrinks, fills or covers one.
- `build` runs the visual rules every set shares; a set's own rules run only in `library verify-visual NN`, so run it after adopting and before you commit.
- The port carries what v1 drew: a part resting translucent takes `"opacity"` (gradient-stops' ramp bands), a stroke drawn only in Solid `"outline": "none"`, and geometry that changes with the weight `"weights"` (more-options' dots). Continuous loaders stay v1.

## Local gallery

`npm run dev` (or `python3 scripts/serve.py`) serves `prototype/` on port 4189; the gallery is at `/icons/01/`. Pass `-- --port 4190` to change the port. The gallery vendors its player and fonts and needs no build step.

## Packages

```sh
gen-animated-icons library packages    # packages/*/dist and the search sidecar
npm run test:packages                  # package unit tests
gen-animated-icons library release --output dist/packages-vX.Y.Z
npm run test:consumers -- dist/packages-vX.Y.Z   # install the tarballs into vanilla, React and Next.js fixtures
```

`npm run build:packages` and `npm run pack:release` are the legacy equivalents. See the [package contract](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/package-contract.md) and the [release runbook](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/maintainers/releasing.md).

## AI guide

`npm run docs:build` regenerates `prototype/icons/guide/` (the downloadable AI operating guide and its source hashes) from the canonical documents. Edit the canonical source, never the generated copy, then run `npm run check:docs`.
