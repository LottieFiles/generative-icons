# Health & Fitness (36): design and review record

Single-agent workflow (design → author → source checks → native render → visual review → correction), following the Weather set's pipeline and lessons. Self-review only; no independent approval or user acceptance is implied. Package hashes: see `prototype/icons/36/assets/set.json` and `browser-verification.json`.

Shared helpers: `batches/sky.mjs` (copied from Weather; adds `trace`, `ellipse`, `lerp`). Existing meanings checked against the catalog: no heart-rate, step, dumbbell, running, pill or DNA icons existed. `heart`, `clock`/`stopwatch` and `thermometer`-style temperature are distinct.

## Icons, state policy and motion

| Icon | Policy | Main part | Light | Strong |
| --- | --- | --- | --- | --- |
| heart-rate | result + reset | pulse trace | trace draws in 0.64 s; heart beats once at the spike | trace in 0.34 s; lub-dub beat to 114% |
| step-counter | gesture | bare footprints | each print lifts 0.9 and plants | push-off, lift 1.4 with a twist, contact dip |
| dumbbell | gesture | whole dumbbell | curl up 1.6, tilt 4° | brace dip, drive up 4, tilt 8°, counter-swing |
| running | gesture | limbs | one easy stride, 0.5 bob | 6° lean, two quick strides, 1-unit bob |
| water-intake | result + reset | water surface | level rises with small slosh | droplet falls in, 0.32 s rise, hard slosh |
| sleep-tracking | gesture | Z marks | Zs breathe smaller and back | moon rocks; Zs pop from nothing to 116% |
| calories | gesture | flame | flicker, tip sway ≈1 | gutter, flare 112%, 2-unit lick |
| stethoscope | gesture | chest piece | pendulum swing 10° | back 6°, out 18°, damped |
| pill | gesture | capsule halves | open 0.9 along axis | twist 10°, halves pull 2 apart, snap shut |
| syringe | result + reset | plunger | pushes 3 in | draws back, driven home in 0.16 s, drop leaves the needle |
| first-aid-kit | gesture | whole kit | lifted 1, swings 2.5° | jerked up 2.2, swings −4.5/3/−1.5° |
| bandage | gesture | strip | stretches 4% | pinched, pulled 12%, snaps back |
| weight-scale | result + reset | needle | sweeps to the reading | body compresses 5%, needle overshoots 38° and rings down |
| blood-pressure | result + reset | needle (driven by bulb) | one squeeze, needle rises | two squeezes step the needle, 8° overshoot |
| yoga | result + reset | mat roll | the roll travels right, paying out the mat (radius shrinks with rolled length) | drawn back, pushed, rolls past its mark and rocks back |
| meditation | gesture | lotus petals | petals open 7° as the centre lifts | draw in, bloom 11°, settle |
| nutrition | gesture | apple and leaf | rock 3°, leaf flutters 10° | rock −2/7/−4°, leaf whips 16° |
| jump-rope | gesture | rope | one turn over and under | wind back, two fast turns, handles jerk |
| hospital | gesture | sliding doors | part 0.7 and close | jolt, part 1.5, cross pulses 18% |
| dna | gesture (whole-period twist) | strands | one full twist; rungs fade where strands close | wind back 24°, two turns with overrun |

Constructions:
- Arcs, pulse traces and water surfaces keep constant topology. Traces collapse unreached vertices onto the pen.
- Limbs (running, yoga) interpolate joint or polar angles, never positions, so their lengths don't shrink mid-swing.
- The stethoscope tube, blood-pressure hose and water surfaces are rebuilt each frame from their attachments.

## Observed failures and corrections

1. **Clearance contract rules on intended joints.** Rules for door edges on their lintel, needle into barrel, and legs sharing the hip measured joints as gaps. Those rules were removed. Real faults were fixed:
   - Bandage dots touched the pad lines: dots moved to 2.8 spacing, pad lines to 11.8/20.2.
   - Yoga's resting right arm crossed the bent knee.
   - Nutrition's leaf flutter (0.39), the stethoscope swing (0.55), the sleep Z beside the moon (0.71), and meditation's Strong sink onto its shadow (0.6): amplitudes reduced or parts moved.
2. **`verify.mjs`: stroke presets missing.** Step-counter was drawn entirely with filled marks, so no paint followed the 1.5/2/2.5 Outline presets. The soles are now outlined. Only the special-cased ellipsis may be fill-only.
3. **Still after rest:** nutrition's Light leaf flutter ended after its active time, so the active time was lengthened.
4. **Collapsed marks:** sleep-tracking's regrowing Z showed as a dot at 10–20% scale, so the hide threshold was raised to 35%.
5. **Visual review of native styles and poses:** failures that all numeric gates had passed.
   - Yoga v1 looped the arms concentrically around the head and read as a keyhole. The arms now rise from outstretched to a pointed arch, widened after clearance measured 0.45 at its middle.
   - Meditation v1 (armless torso arch over a lens) read as a UFO; v2 (arch plus arms) read as a bell. Replaced with a lotus, the common meditation mark.
   - Step-counter's ringed sole and heel read as "0 o", so the heel is now a filled pad.
   - Jump-rope's thin handle lines read as a bare U, so the grips are now outlined capsules.
   - DNA rungs shrank into blobs at the crossings, so there are fewer rungs, shown only when longer than about 2 units.
   - Blood-pressure's elliptical bulb read as a ring beside a magnifier, so it is now a teardrop whose tip takes the hose.

6. **Coordinator review of the styles sheet** (after a full native PASS): three icons still did not read.
   - Step-counter's ovals plus dots read as "!!" or eyes. It is now two shoe prints (a wide-ball, narrow-waist outlined sole plus a separate filled heel pad), offset like walking steps.
   - Yoga read as a flower, key or lollipop through four drawings: arch loop, shoulder-bar pentagon, head-hugging diamond, then the final one. The final pictogram has a filled head, horizontal upper arms to wide elbows, forearms meeting high overhead, a torso, a standing leg and a bent-leg triangle. The arms sweep from the sides on side-bowed paths (a linear path cut through the head).
   - DNA: crossings blobbed and end rungs read as lids. Now 0.7 period with exactly two crossings and open ends. The strand profile sin(π/2·cos θ) flattens the wide middle lobe. Three rungs there have a half-length computed numerically so every rung end keeps a Bold gap ≥1.1 to every strand sample. A slope-only inset failed clearance at 0.65 because of curvature.

7. **Second coordinator review:**
   - Step-counter's sole plus separate heel pad read as "!!". It is now bare footprints: one closed sole outline (heel, arch, ball) with three filled toe dots. The toes were respaced after clearance measured 0.68 at the 0.82 print scale.
   - Yoga (`yoga`, "Yoga") read as a warning triangle on a signpost. The figure is replaced by a yoga mat unrolling in side view: a flat mat line and a roll (circle plus inner spiral turn) that travels right. Its radius shrinks with rolled length and the spiral turns with rolling contact.
     - At a 4.6 rest radius the inner turn blobbed at Bold (snail), so the rest radius became 5.6.
     - Strong's pull-back crossed the edge margin and grew the roll, so the pull-back was shortened and the radius clamped.

## Reusable lessons

- Write gap contracts only between parts that should be separate. A joint (hinge, lintel, hip, needle socket) is not a gap; measure the free ends instead.
- Every icon needs at least one Outline stroke that follows the weight presets. Filled marks can carry detail but not the whole drawing.
- A figure's silhouette reads before its parts. Seated or lotus figures made from a lens and an arch read as objects (UFO, bell) at 24 px, so prefer an established symbol (a lotus).
- Concentric strokes around a head or dot read as a target or keyhole. Keep limbs or rings offset.
- Hide scaled strokes well before zero (about 35% for small glyphs). A collapsed stroke still paints a round dot.
