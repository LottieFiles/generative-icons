<!-- Generated from .agents/skills/animated-icon-workflow/SKILL.md by `npm run docs:build`. Edit the source, not this copy. -->

# Drawing and motion craft

> These are the Generative Icons library's own rules, shipped with the CLI so the icons you make match the family. Where they name repository files, the gallery or `library` commands, use this skill's loop instead: `context`, `create`, `check`, `draft`, `build` and `preview` ([SKILL.md](../SKILL.md)).

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

### Continuous indicators

For an indeterminate loader or active voice indicator, author a complete periodic cycle without the gesture family's end hold. Declare `continuous: true` on each applicable icon; a category can also contain gestures and retained states. Use set-owned export support for this mixed metadata and native player looping while hover, focus, or Loop all remains active. Finish on the next cycle boundary when the pointer leaves; touch remains one cycle and reduced motion stays at a readable static pose. Test profile switches and stale loop listeners.

Validate position and velocity at the analytic cycle endpoint, then render the playable frames and compare the wrap against ordinary adjacent-frame motion. Check for unwanted still runs. A part can recycle only inside a verified zero-opacity interval; the whole indicator must stay visible. Do not force exact first/last playable-frame equality because adjacent frames contain different phase positions. Keep the old still-hold checks for gesture and state icons. Light and Strong may use different readable phase poses when they are continuous loaders.
