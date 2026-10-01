---
version: "1.0"
system-version: "0.1.0"
name: Generative Animated Icons
register: expressive
overshoot: spatial-only
summary: Physical, causal motion. Something causes the action, the icon responds with material-true movement, then holds a meaningful final state.
description: Motion system for the Generative Animated Icons family of 32 × 32 animated icons, exported as dotLottie with Light and Strong profiles, light/dark themes and style slots. It describes the shipped library. The numeric source of truth is motion/icon-sets/library/family-contract.json.

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
    allow: [shape-layers, path-morph, markers, rounded-corners]
    deny: [expressions, effects, masks, mattes, image-layers, text-layers, 3d, audio, precomps, gradients-animated, time-remap, motion-blur]
  assets:
    naming: "<name>.lottie"
    targets: [dotlottie]
  reduced-motion:
    strategy: segment
    segment: rest
---

# Generative Animated Icons — Motion

## Overview

This file is a compact, machine-readable summary of the motion system. The full rules are in [docs/motion-principles.md](docs/motion-principles.md) and the [animated icon workflow](.agents/skills/animated-icon-workflow/SKILL.md).

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

## Durations

| Token | Value | Use |
|---|---|---|
| timeline | 2400ms | the fixed timeline of every asset (144 frames at 60 fps); the upper bound of any action |

Action length is authored per icon and recorded as the `action` marker's duration. Loading indicators use the full timeline.

## Easing

The engine's Lottie export writes motion as sampled path keyframes joined by straight lines, so easing lives in the sampled shapes, not in keyframe tangents. That is why only `linear` is declared. The export writes linear as tangents `o(1,1) i(0,0)`, which `easing-map` records. Timing and easing are designed in the authoring helpers (`motion/icon-sets/library/direction.mjs`).

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
- **Features:** shape layers and path animation only. No expressions, effects, masks, mattes, precomps, image or text layers.
- **Authoring:** don't hand-write Lottie JSON. Author the icon's source with the authoring kit, build it with `gen-animated-icons` (which exports on LottieFiles' hosted engine) and run its verification (see [creating icons](docs/creating-icons.md)).

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
