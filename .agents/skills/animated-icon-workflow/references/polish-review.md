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
