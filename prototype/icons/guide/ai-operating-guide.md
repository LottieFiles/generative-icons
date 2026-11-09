# Generative Icons — AI operating guide

Give this file to your coding agent to create or revise icons that match the Generative Icons family by LottieFiles. It bundles the canonical workflow, rules and contract from the source repository into one file.

Repository: https://github.com/LottieFiles/generative-icons

## Quickstart

In an app, `npx generative-icons init` installs the generative-icons agent skill, which covers creating a missing icon in the app itself. This guide is the full family workflow used for the library's own sets; give it to your agent for deeper reference.

1. Search existing icons: `generative-icons search "<meaning>"`, then `generative-icons context "<meaning>"` for the reference packet.
2. Read the direction: the motion principles (events, not nudges; draw the subject once), the motion system, the drawing principles and the polish review, then the spec format. They follow the workflow below.
3. Fill in the task brief below and follow the workflow in the first source section.
4. Write each icon as a spec: `generative-icons create <id>`, then `check`, `draft`, `build` and `preview`. In the library repository, build it in a scratch project and add it with `generative-icons library adopt SET-ID --add <build>` (`create <library-id> --replace` and `library adopt` reanimate one), then `library verify` and `verify-visual`; a new set starts with `library new-set` and ends with `register`.

Building exports native Lottie on Creator MCP, LottieFiles' hosted engine, so it needs a LottieFiles sign-in (`generative-icons login`).

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
metadata:
  internal: true
---

# Animated icon workflow

## Quickstart

This skill is for library sets in this repository. In an app, the `generative-icons` skill that `generative-icons init` installs covers the same loop for local icons.

Run everything from the repository root with the `generative-icons` CLI (build it with `cargo build --release -p generative-icons`). Replace `SET-ID` with the set directory, for example `39-food-drink`.

1. **Check tools:** `generative-icons doctor`. At the repository root its app checks fail because the repository is not an app; the cpu, quickjs, renderer, sign-in and engine checks are the ones that matter here.
2. **Read the direction:** [motion principles](https://github.com/LottieFiles/generative-icons/blob/main/docs/motion-principles.md), [drawing principles](https://github.com/LottieFiles/generative-icons/blob/main/docs/design-principles.md), the [motion system](https://github.com/LottieFiles/generative-icons/blob/main/MOTION.md), [polish-review.md](https://github.com/LottieFiles/generative-icons/blob/main/.agents/skills/animated-icon-workflow/references/polish-review.md) and the spec format, [authoring-v2.md](https://github.com/LottieFiles/generative-icons/blob/main/crates/generative-icons/assets/skill/references/authoring-v2.md) (its "Motion that reads" and "What build's visual check fails" sections above all).
3. **Search:** `generative-icons context "<meaning>" --search`, then rerun without `--search` and with `--neighbor <id>` (up to four) to get the reference packet.
4. **Brief:** per icon, write meaning, states, neighbors and a **cause → action → response → final state** plan for Light and Strong: the event, the driver and its followers, the cues hidden at rest, and how the one drawing moves.
5. **Author each icon as a spec** (`gai-icon/2`) in a scratch project with this checkout as its library (`--library <checkout>`): `generative-icons create <id>` for a new icon, `create <library-id> --replace` to reanimate one. Then `check`, `draft`, `build <id> --no-install` and `preview`, until the sheets read. Write a JS batch only for what specs cannot express yet (continuous loaders; see [production.md](https://github.com/LottieFiles/generative-icons/blob/main/.agents/skills/animated-icon-workflow/references/production.md)).
6. **Adopt:** `generative-icons library adopt SET-ID <build>` replaces an icon; `--add` adds a new one. A new set starts with `generative-icons library new-set <slug> --name "Display name"` and can be made of specs only. Give every new icon search terms in the set's `search-terms.json`.
7. **Verify and inspect:** `generative-icons library verify SET-ID`, `generative-icons library verify-visual SET-ID` (it renders every action and writes review sheets) and `generative-icons library verify --paint`. Review native playback in the local gallery (`npm run dev`, `/icons/01/`).
8. **Register** a new set: `generative-icons library register SET-ID` after its first complete verification.
9. **Finish:** `generative-icons library packages`, `npm test` and `cargo test --workspace` (new icons change the pinned catalog counts; see [crates/README.md](https://github.com/LottieFiles/generative-icons/blob/main/crates/README.md)), then open a pull request.

**Building needs the user's LottieFiles sign-in.** Export runs on Creator MCP, LottieFiles' hosted engine. On `NOT_LOGGED_IN` or `SESSION_EXPIRED`, ask the user to run `generative-icons login` in their terminal; never sign in yourself. Meanwhile complete the source checks (`check` for specs, `library preflight` for JS batches) and report the native gates as not run.

## Scope and roles

Apply this workflow to the requested scope and complete the authorized work without adding approval steps for routine drawing, animation, fixes or local review. The latest user direction wins over past accepted designs or these defaults. For a critique-only request, deliver findings and motion briefs without editing icons. For an explicitly static icon, apply the drawing and export checks without adding animation.

**One agent, separate stages.** One developer-chosen coding agent runs the whole loop: retrieve references → design → author → check → render and inspect → correct → deliver. No second model, supervisor or handoff is required; invoke another model only when the developer asks. Self-review is not independent approval, a passing check does not prove artistic quality, and a text-only agent must not claim it inspected an image. This is the canonical statement of these limits; the references do not repeat it.

## Where things live

- Gallery: `prototype/icons/01/index.html`, served by `npm run dev` on port 4189 at `/icons/01/`. All icons belong on that page; it starts each icon in **Strong** and Light stays selectable.
- Sources: `motion/icon-sets/SET-ID/`: `specs/<id>.icon.json` for spec icons, `definitions.mjs` and `batches/` for JS ones. Catalog: `prototype/icons/catalog.json`; each set is registered in a named browse category.
- Rules: [polish-review.md](https://github.com/LottieFiles/generative-icons/blob/main/.agents/skills/animated-icon-workflow/references/polish-review.md) (rejection criteria and per-icon record), [generation-harness.md](https://github.com/LottieFiles/generative-icons/blob/main/.agents/skills/animated-icon-workflow/references/generation-harness.md) (reference retrieval and preflight), [production.md](https://github.com/LottieFiles/generative-icons/blob/main/.agents/skills/animated-icon-workflow/references/production.md) (build and evidence), `docs/design-principles.md`, and `motion/icon-sets/library/motion-directions.json` for existing motion intent.

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

- Find the event, not a nudge: what the object does for the user (a bell rings, send leaves, a download lands). Show its cause briefly with parts hidden at rest (sparks, arcs, speed lines, a trail), and draw marks that mean something.
- Draw the subject once. The motion transforms the drawing the rest pose shows (moves, turns, scales, squashes or draws it) and brings it back without leaving the canvas. Never swap it for a copy that fades in where the first was; parts hidden at rest are cues, never the subject again, and no frame may jump.

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

Read [references/production.md](https://github.com/LottieFiles/generative-icons/blob/main/.agents/skills/animated-icon-workflow/references/production.md) for the current build and evidence procedure. Export only through the CLI on the hosted engine: `build` for a spec, then `library adopt`; `library build` for a set's JS batches. Do not claim one tool produced an asset when another did.

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


---

## Source: docs/motion-principles.md

# Motion principles

Design **cause → action → response → final state** before writing keyframes. Define the stationary structure, main driver, followers, anchors, hinges and protected gaps.

## Events, not nudges

An icon that moves without anything happening reads as decoration: the microphone that tilts a few degrees and comes back. Every action needs an event.

- **Find the event.** Ask what the object does for the user and animate that, not the object's mass: a microphone takes sound, send leaves, Wi-Fi gains signal, a shield blocks, a bell rings, a download lands.
- **Show the cause, briefly.** Sound arcs, sparks, steam, speed lines, trails and ripples may appear during an action when they are hidden at rest and gone when it ends.
- **Draw the meaning.** Marks that mean done, created or blocked (a check, a slash, a plus) draw on instead of moving in.
- **Rhythm, not a round trip.** Hit, rebound, settle: two or three beats, each smaller. The return is quicker and smaller than the action, never its mirror.
- **Sequence the parts.** The driver moves first and followers 60–120 ms later; repeated parts arrive in order.
- **Draw the subject once.** The motion transforms the object the rest pose shows (it moves, turns, scales, squashes or draws) and brings it back without leaving the canvas. Never swap it for a copy that fades in where the first was: the eye reads the object jumping back. Parts hidden at rest are cues, never the subject again, and no frame may jump.

## Light and Strong

Light plays the icon's event once, with one beat, and must read at 24 px. Strong must visibly change the primary action at normal speed and small sizes through travel, rotation, preparation, more beats, decisive timing and material response. A speed multiplier, extra pause or decorative follower is not sufficient.

Compare profiles at equal elapsed seconds and at 24/32 px. Keep common meaningful endpoints so switching profiles does not reverse the user's state. The current family uses 60 fps and a 2.4-second timeline; active duration is authored per icon.

## Material and mechanics

Hardware remains rigid around its pivots; the events around an object (sound, sparks, wind) are not material and may appear and fade. Paper feeds without arbitrary stretching. Cloth carries a traveling deformation. Ink follows its pen. Followers react to a cause; they do not all repeat a generic pulse.

Keep ordinary travel continuous through intermediate waypoints. Reserve full stops for deliberate contact, inspection or state changes. A physical impact can break velocity and start a follower response. Data values remain monotonic and must not overshoot for decoration.

## State and looping

Forward actions such as charging, writing and scanning retain their result. Author a separate preview reset or real reverse action when required. The gallery's loop is a preview policy, not permission to undo the product's meaning.

Continuous indicators need a complete periodic cycle, with position and velocity continuity and a useful reduced-motion pose. Do not insert a gesture end hold into an indeterminate loader. Check the playable-frame wrap; identical first/last playable frames are not required.

## Review

Watch native motion at normal speed before frame stepping. Inspect preparation, main action, contact, response, hold and reset. Verify attached joints and clear counters throughout the swept motion. Transparent resets must not inflate visible travel/speed measurements.


---

## Source: MOTION.md

---
version: "1.0"
system-version: "0.1.0"
name: Generative Icons
register: expressive
overshoot: spatial-only
summary: Physical, causal motion. Something causes the action, the icon responds with material-true movement, then holds a meaningful final state.
description: Motion system for the Generative Icons family of 32 × 32 animated icons, exported as dotLottie with Light and Strong profiles, light/dark themes and style slots. It describes the shipped library. The numeric source of truth is motion/icon-sets/library/family-contract.json.

durations:
  timeline: 2400ms

easings:
  linear: linear

roles:
  icon-play:
    use: "an icon performs its one action and holds the result (send, save, check, unlock)"
    avoid: "continuous activity such as loading or recording (use loop); decorative pulses with no cause"
    duration: "{durations.timeline}"
    easing: "{easings.linear}"
    properties: [path, x, y, scale, rotate, opacity, fill, stroke-dashoffset]
    reduced: remove
  loop:
    use: "an indicator of ongoing activity: loading, syncing, recording, playing audio"
    avoid: "anything that finishes; attention-getting; one-shot confirmations"
    duration: "{durations.timeline}"
    easing: "{easings.linear}"
    properties: [path, x, y, scale, rotate, opacity]
    max-run: 30s
    pausable: true
    reduced: remove

defaults: { enter: icon-play, exit: icon-play, loop: loop }

reduced-motion:
  policy: remove

bindings:
  roles:
    icon-play: { marker: action }
    loop: { marker: showcase }

constraints:
  duration-range: { max: "{durations.timeline}" }
  loop-max-run: 30s

vector:
  fps: 60
  canvas: { width: 32, height: 32 }
  easing-map:
    linear: { o: [1, 1], i: [0, 0] }       # the exporter's linear form; same curve as o(0,0) i(1,1)
  loop: { default: false, idle-segment: showcase }
  markers:
    enter: action
    idle: showcase
    reduced: rest
  state-machine:
    states: [action, rest, showcase]
  budgets:
    max-layers: 60
    max-file-kb: 128
  slots:
    naming: "<group>.<token>"
    types: [color]
    themes: [light, dark]
    required: [color.primary, style.outline.light, style.outline.regular, style.outline.bold, style.solid, style.tint, style.outline.width]
  features:
    allow: [shape-layers, path-morph, markers, rounded-corners, subtract-masks, alpha-inverted-mattes]
    deny: [expressions, effects, image-layers, text-layers, 3d, audio, precomps, gradients-animated, time-remap, motion-blur]
  assets:
    naming: "<name>.lottie"
    targets: [dotlottie]
  reduced-motion:
    strategy: segment
    segment: rest
---

# Generative Icons — Motion

## Overview

This file is a compact, machine-readable summary of the motion system. The full rules are in [docs/motion-principles.md](https://github.com/LottieFiles/generative-icons/blob/main/docs/motion-principles.md) and the [animated icon workflow](https://github.com/LottieFiles/generative-icons/blob/main/.agents/skills/animated-icon-workflow/SKILL.md).

Each icon tells one small physical story: **cause → action → response → final state**. A bell swings because it was struck, a page feeds because the printer pulled it, and the icon then holds its result. Every icon ships in two profiles. **Light** makes one quiet, readable action. **Strong** visibly changes the main action through travel, rotation, anticipation and material response. Strong is not just Light played faster.

Every icon is 32 × 32 at 60 fps on a fixed 2400 ms timeline. Each icon sets its own active duration inside that timeline.

## Principles

- **P1: Cause before motion.** Every movement has a driver; followers react to it instead of repeating a generic pulse.
- **P2: Material stays true.** Hardware stays rigid around its pivots, paper doesn't stretch, cloth carries a travelling wave, ink follows its pen.
- **P3: Forward actions keep their result.** A check stays checked. The gallery's loop is a preview policy, not permission to undo meaning.
- **P4: No decorative overshoot on data.** Values such as progress and counts are monotonic. Overshoot is for physical, spatial motion only.
- **P5: Loops are only for ongoing activity.** Loading, recording and playback use `loop`; everything else uses `icon-play`.
- **P6: The rest pose is the reduced-motion pose.** Every asset has a `rest` marker showing the meaningful final state.
- **P7: Readable at 24 px.** Light and Strong are compared at equal elapsed time and at 24 and 32 px before approval.
- **P8: Every action has an event.** Animate what the object does for the user, not a generic nudge. Show the cause briefly (sound, sparks, wind, a trail: hidden at rest), draw marks that mean something, and move in decaying beats with parts in order. See [motion principles](https://github.com/LottieFiles/generative-icons/blob/main/docs/motion-principles.md#events-not-nudges).
- **P9: The subject is drawn once.** The motion transforms the drawing the rest pose shows and brings it back; it is never swapped for a copy. Parts hidden at rest are cues, never the subject again. See [motion principles](https://github.com/LottieFiles/generative-icons/blob/main/docs/motion-principles.md#events-not-nudges).

## Durations

| Token | Value | Use |
|---|---|---|
| timeline | 2400ms | the fixed timeline of every asset (144 frames at 60 fps); the upper bound of any action |

Action length is authored per icon and recorded as the `action` marker's duration. Loading indicators use the full timeline.

## Easing

The engine's Lottie export of the `definitions.mjs` sets writes motion as sampled path keyframes joined by straight lines, so easing lives in the sampled shapes, not in keyframe tangents. That is why only `linear` is declared. Icons adopted from spec v2 (`specs/<id>.icon.json`) are built from eased keyframes instead and keep the curves themselves. The export writes linear as tangents `o(1,1) i(0,0)`, which `easing-map` records. Timing and easing are designed in the authoring helpers (`motion/icon-sets/library/direction.mjs`).

## Roles

| Role | When | Not for |
|---|---|---|
| icon-play | an icon performs its action and holds the result | loading, recording or other ongoing activity |
| loop | ongoing activity: loading, syncing, recording, playback | anything that finishes |

## Vector Animation

- **Frame rate 60, canvas 32 × 32, 144 frames** for every shipped animation.
- **Markers:** `action` covers the icon's active motion, `rest` is one frame on the final pose, and `showcase` covers the whole timeline for the gallery preview loop. These are the only marker names allowed.
- **Profiles:** each `.lottie` holds a Light animation (`<name>`) and a Strong one (`<name>-strong`), with `dark` and `light` themes.
- **Slots:** `color.primary` plus the five style slots (`style.outline.light`, `style.outline.regular`, `style.outline.bold`, `style.solid`, `style.tint`) and the `style.outline.width` stroke-width token bound to every Regular outline stroke, which switch weight and style at runtime.
- **Features:** shape layers and path animation. Solid cut-outs (counters and knockouts) use subtract masks, and an inverted alpha matte when the cut-out moves, draws on or fades; the compiler adds both from the spec. No expressions, effects, precomps, image or text layers.
- **Authoring:** don't hand-write Lottie JSON. Author the icon's source with the authoring kit, build it with `generative-icons` (which exports on Creator MCP, LottieFiles' hosted engine) and run its verification (see [creating icons](https://github.com/LottieFiles/generative-icons/blob/main/docs/creating-icons.md)).

## Accessibility & Reduced Motion

Under reduced motion, players show the `rest` frame and don't animate. Loops stop on their rest pose. Nothing flashes.

## Performance

Budgets: at most 60 layers and animation JSON at most 128 KB. Sampled path keyframes are the main cost, so the largest files are the best candidates for reducing samples.

## Naming

Assets are `<name>.lottie`, with animations `<name>` (Light) and `<name>-strong` (Strong). Layers are named `<name>-<part>` with a style or weight suffix where one applies (`heart-body-bold`). Layers carry no role prefix, because every layer in an asset shares the asset's role.

## Decision Procedure

1. Does the icon show something still in progress (loading, recording, playing)? → `loop`.
2. Otherwise → `icon-play`: design the cause, the action and the final state, then author Light and Strong.

## Do's and Don'ts

- Do give every movement a cause and keep the final state meaningful.
- Do ship `action`, `rest` and `showcase` markers in every animation.
- Don't use a loop for something that finishes.
- Don't overshoot data values or opacity.
- Don't make Strong by speeding up Light.


---

## Source: docs/design-principles.md

# Drawing principles

## Shared construction

Use a 32 × 32 vector coordinate system. Outline weights are 1.5, 2 and 2.5 units; Regular is 2. The three weights share one geometry, and every Regular stroke also carries the `style.outline.width` token, so a runtime can render any weight from 1 to 3 without new artwork. Keep caps, joins and useful corners rounded. Sharp geometry is appropriate when required for recognition or a supplied brand shape.

The numeric source of truth is [family-contract.json](https://github.com/LottieFiles/generative-icons/blob/main/motion/icon-sets/library/family-contract.json). Do not scale a whole group to simulate weight or quantize animated coordinates to the grid. Matching numbers do not guarantee equal visual weight.

## Styles

- **Outline:** use the shared stroke presets and preserve open counters at Bold.
- **Solid:** construct the intended filled silhouette. Where a stroke joins a filled surface, bury its cap inside the surface to prevent raster seams.
- **Duotone:** apply 18% tint only to a real enclosed surface. Open lines, brackets and signal arcs remain unfilled and intentionally match Outline. Declare `duotoneFill: false` when applicable.

## Geometry that survives motion

Draw the sections motion exposes: complete rims, hidden page edges, hinges and supporting rails. Parts that meet should share anchors or geometry. Background-colored patches are not occlusion. Transparent outlines cannot hide other outlines.

Preserve meaningful negative space throughout the full action, especially at Bold and in Solid. Inspect arrow junctions as painted shapes; a barb approaching a tight curve may close the opening even when centerlines look separated. Opaque painted surfaces may need slight overlap at a shared edge to prevent an antialiased seam.

## Optical review

Compare the icon with relevant neighbors at 24, 32 and 48 px. Review at 192 px for construction faults. Check actual measured CSS and backing dimensions, not just labels. Small-size readability, visual center and perceived stroke density require rendered judgment.

Keep source findings, rendered measurements and visual preferences separate. Record a reusable correction in these principles or the [polish checklist](https://github.com/LottieFiles/generative-icons/blob/main/.agents/skills/animated-icon-workflow/references/polish-review.md) only after verifying it. Asset-specific dimensions and timing belong with the asset.


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
| The subject leaves and a copy fades in where it was | Draw the subject once: transform the original (move, turn, scale, squash, draw) and bring it back without leaving the canvas. Show the cause with cues hidden at rest; a cue never has the subject's shape. `check` warns `DUPLICATE_DRAWING`. |

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

### Lessons from the Emoji set and website generation (October 2026)

- **One construction for a family of similar objects.** Fix the shared parts first (Emoji: one face circle, dot and arc eyes, one grin and one smile) and build a calibration icon before briefing the rest; a set drawn from one kit reads as a family.
- **Check clearances at Regular and Bold, not only Light.** The first emoji grin and tears touched at 2 and 2.5 though they cleared at 1.5; the fix moved the tears out and narrowed the grin. A teeth line inside a grin closes at Bold and 24 px.
- **Solid faces and cards.** A filled surface can move only one group of its cut-outs. Parts that should read as their own filled shapes on its rim (tears, a hand, a heart) are filled and gap-cut, not knocked out. The authoring-v2 reference has the constructions.
- **Sibling icons must not share one performance.** Five laughing faces each laugh differently (tears spurt, a roll, squeeze-hops, a spreading grin, running sweat); review the family together.
- **Read it blind.** Ask a reviewer who has not been told the subject what the rest pose shows at 24 px and what happens in the frames. The website's gate rejects icons misread blind (a balloon read as a light bulb, a bee as a pretzel); a reviewer who can see the icon's name is not blind.
- **Draw the subject once.** Thirteen library icons swapped their subject for a cross-faded copy; the copy reads as the object jumping back. They were reanimated to transform the drawing they have.


---

## Source: crates/generative-icons/assets/skill/references/authoring-v2.md

# Author an icon as a spec

A v2 icon is one JSON file, `generative-icons/icons/<id>.icon.json`, with `"format": "gai-icon/2"`. It is data: shapes, roles, rigs and eased keys. Nothing in it is a function of time, so there is no pose math to write and no SVG to hand-build. Creator MCP, LottieFiles' hosted engine, builds every motion from it in one call: native shapes, rigs chained by layer parenting, eased keys, masks, mattes, trims and markers. `create` writes one by default. v1 JS batches (`create <id> --batch`, `batches/*.mjs`) keep working for what a spec cannot express yet (the last section); a command takes the spec path only when every icon it names has a spec.

```sh
generative-icons create <id>            # a starter spec that checks clean
generative-icons create <id> --from <library-id>   # start from a library icon's spec
generative-icons check <id>              # offline: structure, names, motion, lints (milliseconds)
generative-icons draft <id>              # one engine call: a review sheet, nothing stored
generative-icons build <id>              # one engine call: export, slots, themes, .lottie, verify, install
```

Fix every `check` error before you draft. A draft is about 4 s and stores nothing, so draft after each change you cannot judge from the numbers; build once the draft looks right. `check` and `draft` never need a build first.

Over 400 library icons are specs that passed every check below. When one is close to what you need (the same object, or the same kind of event), start from it: `create <id> --from <library-id>` copies its parts, rigs, motion, state and reverse under your id. `search` marks those icons `"spec": true`, and every library carries their specs. Redraw what the new meaning needs, rewrite the intent, rename the state and labels, and keep the craft: the event, the beats, the clearance and the endings.

## Conventions

| Item | Rule |
|---|---|
| Grid | 32 units, y down. A point is `[x, y]`, an anchor name, or `{"from": P, "angle": deg, "len": r}`. |
| Angles | Degrees, positive = clockwise on screen. Polar and arc angles: 0 = +x (3 o'clock), -90 = up. |
| Times | Seconds on the 2.4 s timeline. Fractional times are kept. |
| Values | `x`, `y`, `rotate`, `r`, `grow`, `amp` are offsets from the drawn pose (rest 0). `opacity` and `draw` are absolute 0–1 (rest 1; a part's `"opacity"` sets the opacity it rests at, `"hidden": true` rests it at 0). `scale` and `squash` are factors (rest 1). |
| Draw order | The order of `parts`, back to front. |
| Comments | A `"//"` key on the spec, a part, a rig, a track (a `follow` included) or `reverse`. |

The drawing is the rest pose. A gesture ends where it starts; a stateful icon (`state`) ends changed.

## Motion that reads

Give every action an event, not a nudge that comes back:

- Animate what the object does for the user: a microphone takes sound, send leaves, Wi-Fi gains signal, a bell rings, a download lands.
- Show the cause briefly with parts hidden at rest (`"hidden": true`, then key their `opacity`): sound arcs, sparks, speed lines, a trail, a ripple. Hide them again before the action ends.
- Draw marks that mean done, created or blocked (`draw`) instead of moving them in.
- Hit, rebound, settle: two or three beats, each smaller (`hit`, `settle`, `bounce`). Make the return quicker than the action.
- Move the driver first and followers 60–120 ms later (`follow` with `lag`).
- Light plays the event once; Strong adds beats or travel. Think 1.5–4 units of travel or 10–40° of turn: a move you cannot see at 24 px does not count.
- Keep every part at least 1.4 units inside the canvas at Bold in every frame, travel and overshoot included: at 24 px, ink on the outer pixel fails the visual check `build` runs. `check` warns within 1.4.
- A reading stops dead on its value: a gauge needle, a level, a score or a chart point never overshoots what it reads. Strong adds a step (a second click, a second milestone) instead of travel; `LIGHT_STRONG` may warn, and that is expected.
- A turn of a shape that looks the same turned (a hexagon, a gear, a ring of dots) is invisible: give it a press, a pop or a cue around it instead.
- A squash about a line reads as a turn or a flip (a key, a coin) at large sizes, and as a squeeze at 24 px: confirm it on the 24 px sheet.
- Draw the subject once and transform it: it moves, turns, scales, squashes or draws (`draw` off and back on), and comes back to rest without leaving the canvas. Never swap it for a copy: a part hidden at rest with the subject's shape that takes over as the subject fades reads as the object jumping back (`DUPLICATE_DRAWING`). Parts hidden at rest are cues (sparks, speed lines, a wake, a ripple, a smaller droplet), never the subject again. A send, launch or drip goes a short way with a cue and springs back; a result that differs from the rest pose is a state with a real reverse.
- No frame may jump: never start the action by snapping away what the rest pose shows.
- Transient parts must read when they show: a spark, drop or arc under 2 px at 24 px is a speck (`SMALL_AT_24`).
- Icons that do the same thing get the same treatment: every arrow of a set, every lock, every selector.

## Parts

```json
"parts": {
  "body":  {"shape": {"circle": [[16, 17.5], 8.5]}, "role": "surface", "solid": "ring"},
  "hand":  {"shape": {"line": ["dial", {"from": "dial", "angle": -30, "len": 2.3}]}},
  "feet":  {"shape": [{"line": [[10.5, 23.5], [10.5, 26]]}, {"line": [[21.5, 23.5], [21.5, 26]]}]},
  "bell-r": {"mirror": {"of": "bell-l"}}
}
```

| Shape | Form |
|---|---|
| `line` | `{"line": [P, P]}` |
| `polyline` | `{"polyline": [P, …], "r": corner}` (up to 96 points) |
| `rect` | `{"rect": [x, y, w, h], "r": r or [tl, tr, br, bl]}`. Starts at the bottom-left corner, clockwise. |
| `circle`, `ellipse` | `{"circle": [P, r]}`, `{"ellipse": [P, rx, ry]}` |
| `arc` | `{"arc": [P, r, from, to], "closed": true}` adds the chord |
| `polygon` | `{"polygon": [V, …], "r": corner}`; a vertex is a point or `{"p": P, "r": corner, "curve": [P, P]}` (up to 96) |
| `wave` | `{"wave": {"x": [x0, x1], "y": y, "cycles": n, "amp": a, "phase": p}}` |
| `path` | `{"path": "M… Z"}`: SVG data with ONE subpath (one M, Z only at the end) |
| `morph` | `{"morph": [A, B], "by": "<driver>"}`: two shapes with the same command list; key the driver 0 → 1 |
| list | `"shape": [S, S, …]`: several subpaths painted as one layer per style |

A part rests visible, translucent (`"opacity": 0.3`, which its opacity track starts and, for a gesture, ends at) or hidden (`"hidden": true`). `"weights": {"light": S, "bold": S}` gives the Light and Bold outlines shapes of their own, for a dot that grows with the weight (`shape` draws Regular, Solid and tint; such a part moves only with its rig).

`mirror` copies another part across x = 16 (or `"x"`/`"y"`), with its options unless you set them. Anchors: `"anchors": {"name": [x, y]}`, plus derived ones on every part: `<part>.c`, `.top`, `.bottom`, `.left`, `.right`, `.tl`, `.tr`, `.bl`, `.br` (bounding box at rest) and `.start`, `.end` (open shapes). Polar points replace hand trigonometry: a hand at 2 o'clock is `{"from": "dial", "angle": -30, "len": 4}`.

### Roles: one drawing, every style

Each part expands into the style layers the runtime switches: outline at 1.5 / 2 / 2.5, Solid, and the Duotone tint. Never draw a style twice.

| Role | Outline | Tint | Solid (`auto`) |
|---|---|---|---|
| `stroke` (default) | stroked; `"outline": "none"` draws none, so the part shows only in Solid | none | the centerline at 2.6 |
| `surface` | the frame stroked; `"outline": "open"` leaves out the closing edge, `"none"` draws none | the interior (`"tint": false` for none) | filled, plus a 2.6 rim |
| `fill` | filled in every weight | none | filled |

`"solid"` also takes `"ring"` (the outline at 2.6, hollow), `"fill"`, `"none"`, `{"grow": d}` (fill plus a 2d rim; d > 0) or a shape of its own.

### Holes, knockouts and gaps

- `"holes": [S, …]`: real counters. They are stroked in Outline and cut from tint and Solid by subtract masks, so overlapping counters stay open. On a part that `scale` or `squash` reshapes, the masks follow.
- `"knockout": ["hand", {"part": "knob", "gap": 0.8}]` on a surface: in Solid, those parts are cut out of it with a clearance (default 0.6, or the part's `"gap": number`) instead of drawn. A part that sits still on the surface's rig is cut by masks. One that moves on another rig, is reshaped, draws on, fades or starts hidden is cut by a matte that follows it, trim included, so the cut-out draws on with the part. A surface's moving knockouts must share one rig (`KNOCKOUT_RIGS`), and a part with its own trim or visibility must be its surface's only moving knockout (`KNOCKOUT_TRACKS`).
- A morphing knockout is cut by a matte that follows the morph, when it is its surface's only moving knockout. If another part already holds the matte, it is cut at its rest shape and `MORPH_KNOCKOUT` warns.
- In Solid every part is one colour: a detail drawn inside a filled Solid body disappears unless it is knocked out (`SOLID_HIDDEN`), or the body's Solid is `"ring"`.
- A surface's default Solid is filled with a 2.6 rim, so it reads a little bigger than its outline. `"solid": "fill"` fills the outline's shape exactly.
- `"gap": {"other": d}`: cut this part wherever it comes within `d` of another part's shape at rest, in every style.
- A knocked-out part is not drawn in Solid; only its hole is. A part that should read as its own filled shape on or over a surface (a tear on a face's rim, a hand over it, a badge on a card) is not a knockout: give it `"solid": "fill"` and cut the surface around it with `"gap": {"<part>": 1.1}`. The gap is cut at rest, so keep that part where it rests; animate a cue instead of moving it away.
- One surface has one matte, so only one group of its knockouts can move. When two features of a filled surface must move on their own (a face's eyes and mouth), keep one as the moving knockout and split the other in two: a still Solid-only knockout (`"outline": "none"`) and a moving copy drawn only in Outline and Duotone (`"solid": "none"`). Solid then keeps that feature still; say so in the intent.
- A cue that appears inside a filled Solid surface (blush, a sparkle, a laugh line) vanishes into the fill: give it `"solid": "none"`, place it outside the surface, or knock it out, and make sure Solid still reads the event through the other motion.
- Small flying marks (droplets, confetti, embers) are filled (`"role": "fill"`): a hollow outlined drop under about 4 units blobs at Bold.

## Rigs

```json
"rigs": {
  "clock":  {"pivot": {"contact": ["leg-l.end", "leg-r.end"]}, "carries": ["body", "bell-l", "bell-r"]},
  "bell-l": {"pivot": [10, 11.5]}
}
```

- A rig named like a part moves that part. `carries` lists the parts and rigs that move with it: a rig in `carries` is a child rig, at any depth. A part carried by a rig is rigid with it.
- `pivot` is a constant point. `{"contact": [A, B]}` tips about A for negative angles and about B for positive ones (a clock rocking on its feet). It compiles to two rigs, one per sign, split exactly at the zero crossings.
- Any rotation needs a declared pivot (`PIVOT`). `x`, `y` and `scale` on a part without a rig use an implicit rig at the part's centre.
- `scale` and `squash` (`"squash": {"base": y, "widen": 0.5}`) reshape the carried parts' points, so strokes keep their weight. They are exact, and only for a rig that carries parts, not other rigs.

## Motion

```json
"motion": {
  "bell-l.rotate": {
    "light":  [[0, 0], [0.1, -6], [0.42, 0, "inOut"]],
    "strong": [[0, 0], [0.07, 5], [0.17, -20, "hit"], {"settle": 0.68, "to": 0}]
  },
  "bell-r.rotate": {"follow": "bell-l.rotate", "scale": -1}
}
```

A target is `<part or rig>.<prop>`, with props `x y rotate scale squash opacity draw r grow amp`, or a bare morph driver. Each track has `light` and `strong` keys; a motion that leaves a track out holds it at rest. Every icon has Light and Strong.

A key is `[t, value]` or `[t, value, ease]`. The ease says how the motion arrives at that key:

| Ease | Meaning |
|---|---|
| `auto` (default) | Smooth through same-direction keys, stops at turns and at the ends (v1 `flow`, exact) |
| `linear`, `in`, `out`, `inOut` (= `smooth`) | Polynomial curves, exact |
| `sine` | v1's cosine in-out |
| `hit` | Arrive at full speed and stop dead; a hold may follow (hit-then-hold) |
| `launch`, `pass` | Leave at full speed; keep speed through the key |
| `step` | Hold the previous value, jump at this key |
| `[x1, y1, x2, y2]` | A cubic-bezier, x in 0–1 |

Presets expand to keys. `hold`, `settle`, `bounce` and `wobble` are items in a key list; `pulse`, `loop` and `follow` replace the whole list (`"light": {"loop": …}`):

| Preset | Form |
|---|---|
| `{"hold": t}` | Stay at the value just reached until `t` |
| `{"settle": t, "to"?, "amp"?, "decay"?, "swings"?}` | Damped swings about `to` (default: the value just reached), ending at `t` |
| `{"bounce": t, "amp"?}` | A rebound of `amp` × the arriving move at the midpoint, back with `hit` |
| `{"wobble": at, "amp": a, "until": t, "swings"?}` | A damped shake from rest |
| `{"pulse": [start, duration], "r"?: d, "grow"?: d}` | Show, swell and fade a part (on `<part>.r`, `.grow` or `.opacity`); the part is hidden at rest |
| `{"loop": KEYS, "period": p, "until"?}` | Repeat keys that end where they start |
| `{"follow": "<target>", "lag"?, "scale"?, "offset"?}` | value = scale × source + offset, delayed by lag. `scale` is from -4 to 4 (default 1; -1 mirrors), `lag` from 0 to 1 s. As a whole track it applies to both motions. |

Timing is derived: the action ends at the motion's last key, and rest comes one rounded frame after `active + 0.04 s`. `"timing": {"light": s, "strong": s, "reverse": s, "reverseStrong": s}` overrides it (at most 2.3 s), but no key may come after it (`MOVES_AFTER_REST`). A gesture's rotation may end a whole turn (or a multiple of `"cycle": deg`, such as 60 on a hexagon) from where it started. The same pose rasterises a little differently there, and the build compares the rest frame with the first pixel for pixel, so the compiler adds an invisible step back to 0 one frame after the turn's last key. Every other track must end exactly on its rest value, in Light and Strong alike.

## State and reverse

```json
"state": {"from": "locked", "to": "open", "labels": ["Unlock", "Lock"], "reverse": "auto"},
"reverse": {"strong": {"door.rotate": [[0.04, 100], [0.26, 0, "hit"]]}}
```

- `auto` (default): each track's beats replay in mirrored order. Inside a beat, values reflect about its ends, and times and eases are kept. A wind-up stays a wind-up the other way. It reproduces v1's authored toggle-switch reverse key for key.
- `mirror`: an exact time reversal. A hold or step jumps up to one frame later.
- `fade`: the preview reset. States must be `prepared` → `result` and labels `["Play action", "Reset preview"]`; the animations are `<id>-reset` and `<id>-reset-strong`. Only the parts the action leaves changed fade out and back; a part that moved and came back stays.
- `reverse.light` / `reverse.strong` replace any track of a reverse with explicit keys. A track that follows an overridden one follows the override.
- Look at the reverse in the draft every time. `auto` replays a landing reaction (a `hit`, `bounce` or `settle` at the end state) at the start of the reverse and reflects it to the other side: a seat that dips before it moves, a flag that flutters below the pole. Override those tracks so the reverse has its own event (a lock pops open, a lid slams shut) and reacts where it lands. Most stateful icons with a landing need this.
- State names are kebab-case (`checkpoint-saved`). Labels are short verbs (`["Lock", "Unlock"]`). The animations are `<id>`, `<id>-strong` and `<id>-reverse`, `<id>-reverse-strong` (or the two `-reset` ones); `"reverseName": "<id>-unlock"` in `state` keeps a name apps already play (Strong's adds `-strong`).

## What check and draft tell you

| Code | Level | Meaning |
|---|---|---|
| `V2_SCHEMA` | error | Structure: types, ranges, unknown fields (with "did you mean"), JSON Pointer to the spot |
| `ONE_SUBPATH` | error | A `path` with a second M or an early Z; use a list of shapes |
| `UNKNOWN_NAME`, `RIG_CYCLE`, `CARRIED_TWICE`, `ANCHOR_CYCLE` | error | Names and hierarchy |
| `KEY_ORDER`, `TIMELINE`, `NO_MOTION`, `MOVES_AFTER_REST` | error | Times |
| `REST_VALUE` | error | A track that does not start (or, for a gesture, end) at rest; `scale` keyed as 0 is the classic one |
| `PIVOT`, `MORPH_TOPOLOGY`, `KNOCKOUT_RIGS`, `KNOCKOUT_TRACKS`, `COMBINED_PATH_TRACKS`, `PARAM_UNSUPPORTED`, `FADE_STATE` | error | Things the engine cannot build as written |
| `CANVAS_MARGIN` | error / warning | Paint past the canvas in any of 145 frames per motion / within 1.4 of the edge at Bold while at least 30% opaque, where `build`'s visual check would see it on the outer pixel at 24 px |
| `SMALL_AT_24`, `LIGHT_STRONG`, `TRAILING_HOLD`, `LIVE_ANCHOR` | warning | Under 3 px at 24 px (a part hidden at rest: under 2 px); Strong not stronger than Light; a last key that only stretches the action; an anchor resolved at rest |
| `SOLID_HIDDEN`, `HOLE_OUTSIDE`, `MORPH_KNOCKOUT` | warning | A part drawn inside another part's Solid fill, which vanishes in Solid; a hole or knockout that misses its surface at rest; a morphing knockout cut at its rest shape |
| `NUDGE` | warning | A gesture where nothing draws on, fades, morphs or appears, and the whole drawing tilts under 20° and comes back without travelling, a hit or a hold: a nudge, not an event. Find the event ("Motion that reads") |
| `DUPLICATE_DRAWING` | warning | A part hidden at rest draws exactly the shape of a shown part that fades out: the subject swapped for a copy. Remove the copy and transform the original ("Motion that reads") |
| `NOT_IMPLEMENTED` | error | A term of the format that cannot be built yet (below) |

With MCP, the `draft` tool returns the same sheets as images. `draft` writes `.generative-icons/drafts/<id>.png` (rows: Light, then Strong in Regular, Bold, Solid and Duotone, then the reverses; columns: the beats of each motion) and `<id>-24px.png` (the real size, enlarged). The engine's renderer is not the player: `build` ends with dotlottie-rs sheets of every style and both themes in `.generative-icons/previews/<id>/`. Look at both. Solid cut-outs made by a matte (a knockout that moves, draws on or fades) are where they differ: the draft can leave a drawn-on cut-out out entirely or show a matte beside a mask as a different shape. Judge Solid cut-outs on the build's sheets; `build <id> --no-install` renders them without installing.

## What build's visual check fails

`build` renders every action in dotlottie-rs at every size, style, weight and theme, frame by frame, and fails the icon on any of these. `check` cannot see most of them, so design against them from the start.

| Failure | Cause | Avoid it |
|---|---|---|
| Clipped icon | Ink on the outermost pixel, in any frame | Keep paint 1.4 inside the canvas at Bold, overshoot included (`CANVAS_MARGIN` warns) |
| Invisible main action | A frame with nothing drawn | Keep the subject drawn: transform it rather than fading it out; never fade everything at once |
| Loop or state boundary is incorrect | A gesture's last frame differs from its first, or a toggle ends where it started | End every track exactly on rest; whole turns step back (above). Light and Strong end on the same pixels |
| Insufficient animation states | Too few distinct frames for the action's length (half its frames, 6 to 16) | Ease keys instead of `step`-only motion; a typing caret slides each stroke in 40–50 ms |
| Hold is not still | Something moves after rest | No key after the action ends |

Before rendering, `build`'s paint check fails an icon drawn only with fills (`paint: <id>: outline presets []`): the Outline weights need at least one stroked part. For small dots, stroke a circle over a filled core (braille, app-grid); a stroked circle alone shows pinholes in the player at small sizes.

## Not in the format yet

`swing`, `flip`, `minus`, a keyed wave `phase` and connectors (anchors that follow another rig) stop with `NOT_IMPLEMENTED`. Also not supported yet: `scale` or `squash` on a rig that carries rigs, two path-changing tracks on one shape, a negative Solid `grow`, and one pulse per track and motion. Parts that collapse are not hidden automatically: key their `opacity`. Clearance and counter-width lints are not run yet, so check counters at Bold in the draft. Continuous loaders (motion through the whole 2.4 s, trim start or offset) are not in the format yet. Several cut-outs moving on different rigs in one Solid surface cannot all follow (`KNOCKOUT_RIGS`): one surface has one matte. For an icon that needs one of these, write a JS batch: `create <id> --batch`.


---

## Source: crates/generative-icons/assets/skill/references/review.md

# Review previews

Look at the PNGs `preview` writes: one sheet per motion, rows for the looks (Outline at 1.5, 2 and 2.5, Solid, Duotone on dark, then Regular on light), columns for the same elapsed times in both sheets, ending on the rest pose. Use `--times` to put columns on the preparation, peak, contact, settle, hold and reset moments. A passing `check` or `build` proves the file is valid, not that the icon is good.

## Reject and fix

| What you see | Fix |
| --- | --- |
| Sharp or hooked contours despite round caps | Author the curve in the path; check inner and outer turns in Solid and at 2.5 weight |
| Equal stroke numbers but uneven visible weight | Use a single centerline for simple bars, or respace; never scale a stroked part |
| A lid, handle or flap shows loose ends when it moves | Draw the rim, hinge and contact surfaces; attach endpoints to the moving boundary |
| A counter or gap fills in at 24 px | Measure the swept paint at the widest weight; widen the opening or move the pivot |
| A Solid detail vanishes or a joined fill shows a seam | Give Solid its own contour; overlap paints that form one surface |
| Duotone tints an open tray, doorway or gap | Tint only enclosed surfaces; with none, set `duotoneFill = false` |
| An arrow barb merges into its curve or leaves a dot | Give the head a clear straight approach; hide zero-length round-cap paths |
| Parts drift apart, rigid hardware bends, or the path jumps | Share one driver; move rigid parts with `rigid`; keep path commands identical over time |

## Strong against Light

- Compare both at the same elapsed times. If you cannot tell them apart without their labels, revise Strong.
- Strong needs a stronger primary action: more travel, a wider turn or a more decisive stroke. Reject a difference that is only a follower, a pause, extra keys or a longer file.
- Check the first visible change. Do not spend the opening on a part that is already complete.
- Keep progress monotone, keep rigid parts rigid, and keep holds where the meaning needs them.

## Report

For each icon, state: its meaning and state policy, what differs between Light and Strong, which previews you opened, the findings you fixed, and anything left unresolved or unchecked.


---

## Source: .agents/skills/animated-icon-workflow/references/generation-harness.md

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

The numeric contract is [family-contract.json](https://github.com/LottieFiles/generative-icons/blob/main/motion/icon-sets/library/family-contract.json). The prompt and preflight read the same file. It holds the 32-unit grid, 1.5/2/2.5 Outline weights, 18% eligible Duotone, Strong default, and existing timeline. The CLI's tests check its agreement with every current catalog entry. Versioned exporters keep their constants to preserve prior provenance; a future numeric change requires coordinated authoring/export/paint/render validation. This file is not permission to rescale existing artwork.

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

Preflight is source screening only: painted contacts, closed Duotone surfaces, timing, continuous-loop velocity, player behavior and downloads are checked by the [production gates](https://github.com/LottieFiles/generative-icons/blob/main/.agents/skills/animated-icon-workflow/references/production.md). Generation-context hashes record inputs; final review evidence must still refer to the current final packages.


---

## Source: .agents/skills/animated-icon-workflow/references/production.md

# Production and verification

Run commands from the repository root with the `generative-icons` CLI and confirm paths against the checkout. Use [polish-review.md](https://github.com/LottieFiles/generative-icons/blob/main/.agents/skills/animated-icon-workflow/references/polish-review.md) for acceptance criteria and the per-icon record. Derive the expected icons and forward/reverse/reset actions from the current catalog and set metadata, never from a fixed count.

Follow [generation-harness.md](https://github.com/LottieFiles/generative-icons/blob/main/.agents/skills/animated-icon-workflow/references/generation-harness.md) before export. `generative-icons library build SET-ID` runs source preflight, then exports every action on LottieFiles' hosted engine, post-processes the native files, and packages the set: `.lottie` files, `set.json`, `evidence/build.json` and the set zip. A failing preflight stops it before anything is exported. A clean source report does not replace any native or delivery gate.

## Source layout

Every set lives in `motion/icon-sets/NN-slug/` and has its own `README.md` with its icon list and rebuild commands.

| Sets | Authoring | Build / verify |
| --- | --- | --- |
| `01-heart-bell`, `02-search` | Geometry inline in the set `build.mjs` (legacy; preflight cannot screen it) | Frozen: never rebuilt; `library verify` checks the committed assets |
| `03-essentials` to `06-tools` | `definitions.mjs` plus the matching `library/directed-*.mjs` helper | `library build` (built-in legacy profiles), then `library verify` |
| `07` to `10` (AI) | Set `batches/` and `definitions.mjs` with the shared builder settings | `library build`, then `library verify` |
| `11` onward | Set `batches/`, `definitions.mjs` and `set-config.json`; set-specific rules in `verify.rules.json` or `geometry-contract.json` where present | `library build`, then `library verify` |
| Spec icons, in any set | `specs/<id>.icon.json` (`gai-icon/2`), built in a project and added with `library adopt` (`--add` for a new icon). `64-emoji` is made of specs only | `library build` keeps them as adopted; `library verify` checks them against their spec |

The per-set `build.mjs`, `verify.mjs`, `verify-clearance.mjs` and `package.py` files, and the `build:*` and `verify:*` scripts in `motion/package.json`, are the legacy pipeline. The CLI reproduces their outputs byte for byte and its parity tests still run them; do not use them for new work.

`library/drawing.mjs` contains contour helpers; `library/direction.mjs` contains timing and result/reset helpers. Builds keep sampled scene points as an independent reference and export rigid parts as native transforms. Export needs the user's LottieFiles sign-in; if it is missing, stop at preflight and report the native gates as not run. Never substitute raster animation.

Build complete sets: a set build regenerates the set manifest and packages all its records. Rebuild the complete affected set and confirm that unaffected native asset hashes are unchanged.

Use `generative-icons library new-set <slug> --name "Display name"` to scaffold a new set. Supply unique icon names and custom geometry/motion. After a successful complete build and verification, run `generative-icons library register SET-ID` to check and register it. Do not hardcode an icon or action count into a new acceptance test.

## Paint and styles

Players and the CLI's renderer use the dotLottie core that dotLottie Web `0.80.0` ships. The project uses separate native stroke paints for Light/Regular/Bold, selected with opacity slots, because a scalar width override does not change rendered pixels in this player. Preserve the existing style contract unless a replacement is verified in the actual player.

Slots: `color.primary`, `style.outline.light`, `style.outline.regular`, `style.outline.bold`, `style.solid`, `style.tint`, plus the `style.outline.width` stroke-width token bound to every Regular outline stroke (players that ignore it show the Regular weight). Themeable colors remain solids. Duotone tint uses 18% only where a real enclosed surface exists. Compare the full action when that surface can leave the final frame; an open icon must match Outline at every frame. Do not add invisible dummy surfaces to satisfy a style-difference test.

For filled Outline marks, the weight is in the geometry. The More options check measures the exported dot diameters for each preset; native raster comparisons still must pass. Stroke-based icons retain the constant-width checks.

## Verification sequence

1. Snapshot affected current JSON, dotLottie, metadata, and pose images, plus hashes for untouched assets.
2. Build affected sets with `library build`, run `library verify SET-ID` for each, and run `library verify --paint` across the catalog. Source checks must compare actual exported geometry/paint with the intended constraints.
3. Run `generative-icons library verify-visual SET-ID` (`--icons name1,name2` for a focused run). It renders every action in a child process with the players' renderer and writes `evidence/browser-verification.json` plus rest, style and pose sheets to both the set's `evidence/` and its served `assets/`. A focused run keeps the earlier records of the other icons. Check results and errors; a failed aggregate stays failed.
4. Use the available browser skill to inspect the real gallery (`npm run dev`, `/icons/01/`). Do not control a second browser through an unrelated automation stack. Leave the user's gallery server running, and stop only processes you started.
5. Look at the sheets, and at `generative-icons preview <id> --times …` for the moments that matter. Feed observed flaws into the next correction.
6. Run `npm test` for catalog/state/controller changes, or a focused live interaction check for an asset-only correction. Keep new visual tests proportional to the actual risk.
7. Package the affected set after its final render proof with `generative-icons library package SET-ID`, then refresh the aggregate download with `generative-icons library package --icon-library`. Verify ZIP integrity, catalog action coverage, every selected initial animation, embedded sibling JSON, per-action hashes, and actual HTTP download bytes. Include current shared authoring dependencies, instructions and review evidence in source archives.

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

Add animated icons to any JavaScript app: install only the icons you need with the `generative-icons` CLI, and render them with the shared runtime or the React component.

| Package | Purpose |
| --- | --- |
| `generative-icons` | The CLI: project setup, search, icon installation, and creating icons your app is missing |
| `@lottiefiles/generative-icons-runtime` | Playback, paint, state, reduced motion and lifecycle |
| `@lottiefiles/generative-icons-react` | React and Next.js component |
| `@lottiefiles/generative-icons` | Per-icon descriptors, native dotLottie assets and the catalog, for apps that import icons without the CLI |

Keep the `@lottiefiles/generative-icons*` packages on the same version.

## Install

### 1. Install the runtime

```sh
npm install @lottiefiles/generative-icons-runtime
# React or Next.js, also:
npm install @lottiefiles/generative-icons-react
```

### 2. Set up the project

```sh
npx generative-icons init
```

In a monorepo, pass `--cwd apps/web` to target the app. If your app uses a base URL, include it in `publicPath` in `generative-icons.json`.

### 3. Add icons

```sh
npx generative-icons search heart
npx generative-icons add heart
```

Commit `generative-icons.json`, the added files and `.generative-icons/lock.json`.

### What gets installed

- `init` reads the app's package metadata and creates `generative-icons.json`, the agent skill in `.claude/skills/generative-icons/` and `.agents/skills/generative-icons/`, and a copy of the authoring kit. It does not edit `AGENTS.md`, global agent settings or components.
- `add` copies native files to `public/generative-icons/` and descriptors to `src/generative-icons/` (or `generative-icons/` without a `src` folder), and records their hashes in `.generative-icons/lock.json`.
- `diff` reports local changes, `add` refuses to overwrite modified managed files, and `remove` uninstalls icons whose files are unedited.

## Vanilla JavaScript

```js
import { createIcon } from '@lottiefiles/generative-icons-runtime';
import icon, { src } from './generative-icons/heart.mjs';

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
import icon, { src } from './generative-icons/heart.mjs';

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

**Shared defaults:** edit `defaults` in `generative-icons.json` and run `add` again. Import `defaults` from the generated `generative-icons/settings.mjs` and spread it into runtime options (map `style` to `variant` in React).

Runtime color settings are applied at playback and are not baked into the file. Downloaded native files keep their authored themes and style slots.

## Animated SVG

Where a player isn't wanted (a static page, Markdown, email or a design tool), export an icon as an SVG that the browser animates by itself with SMIL: no player, script or WASM, and about 1 KB gzipped for a typical icon.

```sh
npx generative-icons export bell --out public/icons/bell.svg   # plays on hover
npx generative-icons export menu --style solid --color '#008c76'   # prints the SVG
```

The same export in JavaScript, at build time or in the browser:

```js
import { dotLottieToSvg } from '@lottiefiles/generative-icons-runtime/svg';
import icon, { src } from './generative-icons/bell.mjs';

const svg = await dotLottieToSvg(icon, await (await fetch(src)).arrayBuffer(), { style: 'outline', weight: 2 });
```

`iconToSvg(icon, animations, options)` does the same from parsed Lottie JSON, keyed by animation name. Options are `motion`, `style`, `weight`, `color`, `trigger`, `state`, `size` and `id`.

| Trigger | Plays | Works in |
| --- | --- | --- |
| `auto` (default) | `toggle` for stateful icons, `loop` for loaders, `hover` otherwise | as below |
| `hover`, `click` | The action, then the rest pose, like the runtime's `play()` | Inline `<svg>` |
| `toggle` | Stateful icons: each click plays the other direction | Inline `<svg>` |
| `loop`, `load` | The whole timeline on repeat, or the action once | Inline `<svg>`, `<img>`, CSS backgrounds |
| `js` | `svg.getElementById('<id>_m').beginElement()`. Stateful icons: `<id>_a` leaves the first state and `<id>_b` returns to it | Inline `<svg>` |

- Color is `currentColor` unless you pass a hex value, so CSS `color` themes the icon. Any weight from 1 to 3 works.
- The static attributes hold the rest pose, so renderers without SMIL show the icon at rest.
- Element ids start with the icon id (`--id-prefix`, or the `id` option), with `-` written as `_`. Give each inline copy on a page its own.
- SMIL ignores `prefers-reduced-motion`. In apps that need reduced motion or app-driven state, use the runtime's player.

## iOS and Android

The same commands work in iOS and Android apps: `add` installs the `.lottie` file into the app's assets and a JSON descriptor into `generative-icons/descriptors/`. Play the files with dotLottie for iOS or Android; the installed skill's runtime reference has SwiftUI, Compose and Views examples. Native players show the 1.5, 2 and 2.5 weights; `add --bake-weight <w>` bakes any other weight into the file.

## Missing an icon?

Create it with your coding AI in your own project (`generative-icons create`, then `build`; building needs a LottieFiles sign-in), or [request it](https://github.com/LottieFiles/generative-icons/issues/new?template=icon_request.yml). See [creating icons](https://github.com/LottieFiles/generative-icons/blob/main/docs/creating-icons.md).


---

## Source: docs/package-contract.md

# Package contract

The `@lottiefiles/generative-icons*` packages share one version and are released together. Schema version 1 is the current contract. Any schema change needs an explicit compatibility decision and a new schema version; matching filenames do not imply compatibility.

| Surface | Contract |
| --- | --- |
| Icon descriptor | Stable `id`; readable `label` and `intent`; search terms (`aliases`: other words and short phrases that should find the icon); family and category; start and end state; Light/Strong actions and optional authored reverse; supported styles and authored weights (any weight from 1 to 3 renders through the `style.outline.width` token); native asset path, bytes and SHA-256 |
| Catalog | `schemaVersion`, package `version`, categories and the complete descriptor list |
| Project (`generative-icons.json`, written by the CLI) | `schemaVersion`, platform, detected framework and package manager, relative asset and descriptor folders, public URL base (web), default appearance, agent skill folders and local icon settings. `schemas/project.schema.json` describes only the original web fields; it does not yet cover the newer fields, native configs or weights between 1 and 3. |
| Lock (`.generative-icons/lock.json`) | Selected IDs and versions, installed paths and hashes of managed files; local edits block replacement |
| Release (`release.json`) | Source commit and dirty status, catalog/asset/docs/spec hashes, runtime, CLI and player versions, and the `exporter` note (export is needed for authoring only and is not bundled) |
| Specs (`specs/<id>.icon.json`) | The editable spec (`gai-icon/2`) of every icon reanimated as one, hashed in `release.json`; `generative-icons create <new-id> --from <id>` starts a new icon from it. Icons without a spec have none. |

## How packages are built

`generative-icons library packages` builds the catalog and assets from registered source sets. It rejects duplicate IDs, checks source package hashes and confirms the required animation IDs inside each native bundle. Each icon's `aliases` come from its set's `motion/icon-sets/NN-…/search-terms.json`, a build input hashed in `release.json`. Each icon gets its own descriptor module. A Strong bundle contains both Light and Strong actions, plus reverse actions where applicable; the runtime selects the requested action. Native bundles are copied unchanged.

The runtime package ships the gallery's controller as its canonical implementation; the build copies that exact file and parity tests guard it. Apps should use the runtime package rather than copying the controller.

The website consumes the published packages and validates the release hashes when importing the catalog, assets and documentation. No engine code is included in the packages or the CLI: native export runs on LottieFiles' hosted engine.


---

## Source: docs/development.md

# Development

This repository holds the icon library and the `generative-icons` CLI, a Rust workspace in `crates/`. The CLI builds, checks and packages the whole catalog byte for byte the way the earlier JavaScript and Python pipeline did. That pipeline (`motion/`, `scripts/` and each set's `build.mjs`, `verify.mjs` and `package.py`) stays in the repository as legacy: the Rust parity tests run it as a reference, but new work goes through the CLI.

## Requirements

- **Rust** 1.89 or newer. CI lints and tests with Rust 1.93.1.
- **clang** for C and C++. The vendored QuickJS compiles V8's `ieee754.cc`, which must be built by clang; on Linux set `CC=clang CXX=clang++`. The renderer's bindings also need libclang (`libclang-dev` on Debian and Ubuntu).
- **`CXXFLAGS` with `-ffp-contract=off`.** The renderer must not fuse multiply-adds, or its pixels drift from the browser player's. `.cargo/config.toml` sets it, but a `CXXFLAGS` already in your environment replaces that value, so add the flag to it.
- **x86_64 needs FMA** (Intel Haswell, AMD Piledriver or newer). On Apple silicon, use the arm64 build: Rosetta has no FMA.
- **Node 22+ and Python 3** for the local gallery, the web packages and the repository checks, and for the legacy pipeline.

## Build the CLI

```sh
cargo build --release -p generative-icons
target/release/generative-icons --version
```

Put `target/release` on your `PATH`, or set `GEN_ANIMATED_ICONS_BINARY` to the binary so the npm shim runs it (`node npm/generative-icons/bin/generative-icons.js` in this checkout; `npx generative-icons` once the package is published at launch). The [CLI reference](https://github.com/LottieFiles/generative-icons/blob/main/docs/cli.md) lists every command.

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

Tests never touch your sign-in. They use a temporary `GEN_ANIMATED_ICONS_HOME`, a fake keychain, or `GAI_ENGINE_TOKEN` with a local mock engine. Never run `login` or `logout` from a test or script. Only the two live tests use a real sign-in: `live_exports_match_the_committed_natives_except_path_names` (gai-engine) and `live_build_through_the_hosted_engine` (the CLI). They are ignored by default, and `--skip live_` keeps the whole-catalog run above from starting them. They export on the hosted engine with your session and refresh it, so run them only on purpose, after signing in yourself with `generative-icons login`:

```sh
cargo test --release -p gai-engine --test live -- --ignored live_
cargo test --release -p generative-icons --test engine -- --ignored live_
```

The repository's other checks still run on Node and Python:

```sh
npm ci --prefix motion   # dependencies of the legacy pipeline
npm test                 # gallery controller, generation harness and set scaffold tests
npm run check:docs       # links, publication hygiene, retired names and guide hashes
```

## Author and build a library set

New sets go through the CLI. Building exports on Creator MCP, LottieFiles' hosted engine, so sign in first with `generative-icons login`.

```sh
generative-icons library new-set landscaping --name "Landscaping"
# draw and animate in motion/icon-sets/NN-landscaping/definitions.mjs and batches/
generative-icons library preflight NN-landscaping
generative-icons library build NN-landscaping
generative-icons library verify NN-landscaping
generative-icons library verify-visual NN-landscaping
generative-icons library register NN-landscaping
generative-icons library package NN-landscaping   # repackage with the final evidence
generative-icons library package --icon-library   # refresh icon-library.zip and delivery.json
```

- Registration needs a built, nonempty manifest, a passing preflight and passing verification. Registering a missing manifest would stop the catalog checks from running.
- A set can be made only of spec icons. Scaffold it with `new-set`, leave its `definitions.mjs` exporting no icons, build each icon in a project and add it with `library adopt NN --add <build>` ([below](#reanimate-a-library-icon-with-a-spec)). The first addition writes the set's `set.json` from its `set-config.json`. Preflight passes the set once every icon in its manifest is adopted from `specs/`, `library build` keeps them, and verification and registration run as for any set.
- Sets 01 (Heart/Bell) and 02 (Search) are frozen: their assets are verified as they are and never rebuilt. Sets 03–06 build through built-in legacy profiles.
- Builders replace a set's outputs. Build complete sets and confirm that unrelated asset hashes are unchanged. Do not rebuild only to preview existing files: `generative-icons preview <id>` renders any icon.
- Intermediate `scenes/` and `native/` folders are rebuildable and ignored by Git; final assets under `prototype/icons/NN/assets/` are tracked.

The [creating icons](https://github.com/LottieFiles/generative-icons/blob/main/docs/creating-icons.md) guide covers the design loop; [verification](https://github.com/LottieFiles/generative-icons/blob/main/docs/verification.md) covers the checks.

## Reanimate a library icon with a spec

A library icon can be replaced by a spec v2 icon of the same id. Start it in any project with `create <id> --replace`, author and build it there, then adopt the build into its set:

```sh
generative-icons create airplane --replace       # in a scratch project: the icon's spec, or its v1 drawing ported
# write the motion in generative-icons/icons/airplane.icon.json
generative-icons check airplane && generative-icons draft airplane
generative-icons build airplane --no-install
generative-icons library adopt 14 path/to/project/.generative-icons/build/airplane
generative-icons library verify 14
generative-icons library verify-visual 14
```

- `create --replace` keeps the library id. An icon already reanimated starts from its own spec. Any other starts from its v1 drawing, ported from the Light animation at frame 0: one part per outline layer, and the tint and Solid layers as the part's own where the geometry matches, else as parts that draw only in that style, so every style keeps v1's shapes. The port has no motion (`check` reports an empty `motion`). It carries rest opacity, Solid-only strokes, per-weight geometry and the reverse name, and its warnings name what it could not: geometry only another weight draws, a Solid stroke other than 2.6, an even-odd Solid fill, a paint with no style slot, a trim at frame 0, and keys or shape items the format lacks.
- `adopt` copies the spec to `motion/icon-sets/NN-…/specs/<id>.icon.json` and the actions into the set's assets, replaces the icon's `set.json` entry (keeping its number, category and state labels) and repackages the set. The build must have the icon's actions: the same reverse or reset actions for a stateful icon.
- The rest pose and, for a stateful icon, the end state must stay the library drawing, so apps that switch versions see no jump.
- From then on the spec is the icon's source. `library build` keeps adopted icons as they are (their old definitions stay for the legacy checks but no longer export), and `library verify` checks them against their spec: hashes, format, paint, themes and markers. Their motion checks run in `build`, and `verify-visual` renders them like every other action.
- Frozen sets 01 and 02 can adopt: they are still never rebuilt, so adopting only replaces the icon's assets and entry, and their legacy checks skip it.
- A new icon joins a set the same way with `library adopt NN --add <build>`: `create <new-id>` (not a library id), build it, then add it, and give it search terms (below). It takes the set's category and the number after the library's last icon, and `library build` keeps it. New icons also change the catalog size that some tests pin (see [crates/README.md](https://github.com/LottieFiles/generative-icons/blob/main/crates/README.md)), so update those counts in the same commit.

What the library-wide reanimation (Oct 2026, 412 icons) learned about porting, beyond [authoring-v2.md](https://github.com/LottieFiles/generative-icons/blob/main/crates/generative-icons/assets/skill/references/authoring-v2.md):

- Decide first: keep an icon whose motion already is its meaning, with rhythm in Strong, readable at 24 px (a dial indexing to its next stop, a refresh arrow's full turn, a toggle knob landing). Reanimate generic nudges, lifts and tilts, and moves under a pixel at 24 px.
- Port the rest drawing exactly, vertex for vertex, and match the icon's Solid (`"solid": "fill"` or `"ring"`): the default Solid adds a rim the library drawing does not have.
- A stateful icon ends on the library's end state (compare the build's last frame with the old one) and keeps its animation ids: a reverse apps play under another name (`lock-unlock`, `reconnect-disconnect`, `suitcase-retract`, `menu-open`, `check-reset`) keeps it with `state.reverseName`, which `create --replace` fills in.
- A `previewReset` icon uses `"reverse": "fade"` with states `prepared` → `result`.
- Some sets protect openings (counters, ring holes, Solid openings) in every frame: read the set's entry in `crates/gai-render/src/rules.rs` before designing, so the motion never shrinks, fills or covers one.
- `build` runs the visual rules every set shares; a set's own rules run only in `library verify-visual NN`, so run it after adopting and before you commit.
- The port carries what v1 drew: a part resting translucent takes `"opacity"` (gradient-stops' ramp bands), a stroke drawn only in Solid `"outline": "none"`, and geometry that changes with the weight `"weights"` (more-options' dots). Continuous loaders stay v1.

## Search terms

Each set's `search-terms.json` maps every icon id to the words and short phrases people type to find it: other names for what is drawn (bin, garbage), the action in UI words (delete, remove), button text (log out, add to cart), what an object stands for (doctor on stethoscope) and real abbreviations (2fa, otp). The packages ship them as the descriptor's `aliases`, so the website, `search` and apps all find the icon by them.

- Every library icon needs at least 3 terms, none of them its own label or id; a test checks the whole catalog.
- Terms are lowercase letters, digits, single spaces and hyphens, 1 to 40 characters, at most 24 per icon.
- Leave a term to the icon that is the better answer for it, and skip words that would match hundreds of icons (icon, button, outline) or describe the motion.
- Maintainers judge the website's search on the queries in the internal website repository (`search/queries.json`, `npm run search:eval`) and check each batch of new terms there before release.

## Local gallery

`npm run dev` (or `python3 scripts/serve.py`) serves `prototype/` on port 4189; the gallery is at `/icons/01/`. Pass `-- --port 4190` to change the port. The gallery vendors its player and fonts and needs no build step.

## Packages

```sh
generative-icons library packages    # packages/*/dist and the search sidecar
npm run test:packages                  # package unit tests
generative-icons library release --output dist/packages-vX.Y.Z
npm run test:consumers -- dist/packages-vX.Y.Z   # install the tarballs into vanilla, React and Next.js fixtures
```

`npm run build:packages` and `npm run pack:release` are the legacy equivalents. See the [package contract](https://github.com/LottieFiles/generative-icons/blob/main/docs/package-contract.md) and the [release runbook](https://github.com/LottieFiles/generative-icons/blob/main/docs/maintainers/releasing.md).

## AI guide

`npm run docs:build` regenerates `prototype/icons/guide/` (the downloadable AI operating guide and its source hashes) from the canonical documents. Edit the canonical source, never the generated copy, then run `npm run check:docs`.
