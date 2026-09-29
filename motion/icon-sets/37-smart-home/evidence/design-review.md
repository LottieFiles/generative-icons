# Smart Home (37): design and review record

Single-agent workflow (design → author → source checks → native render → visual review → correction), following the Weather (35) pipeline and its lessons. Self-review only; no independent approval or user acceptance is implied. Package hashes: see `prototype/icons/37/assets/set.json` and `browser-verification.json`.

These are device icons, not duplicates of the existing lock, plug, home, settings or volume icons: each one shows a smart-home device performing its own job. Containers that hold moving detail use Solid rings (`ringRect`, `annulus`), per the Weather lesson.

## Icons, state policy and motion

| Icon | Policy | Main part | Light | Strong |
| --- | --- | --- | --- | --- |
| smart-bulb | result + reset | rays | four rays grow in a calm stagger | glass swells, rays burst to 1.4× and settle |
| thermostat | result + reset | setpoint arc | arc sweeps 200°→330° with the knob riding it | turned up in 0.26 s; the hub pulses as heat engages |
| smart-lock | result + reset | lever handle | lever pressed 30° down and back; status light comes on | yanked 45° with a hit, rebounds; status light pops on |
| door-sensor | result + reset | magnet | slides 4.5 away; two alert arcs open | yanked 5.5 and settles; arcs snap in |
| garage-door | result + reset | door panels | panels roll up into the header | small dip, then rolls up in 0.3 s to a stop |
| ceiling-fan | gesture | blades (side view) | half a revolution shown by blade foreshortening | wind-back, 1.5 revolutions, fan wobbles 3° on its rod |
| air-conditioner | gesture | airflow waves | drift 2.2 down, fade, recycle | vent kicks, waves blast 3.2 in a tight burst |
| smart-speaker | gesture | sound arcs | light ring flickers, arcs above swell 0.8 | body squats and springs, ring flashes twice, arcs 1.4 with a second pulse |
| video-doorbell | gesture | button | press to 84% | press to 70% with a snap back; the bell rocks 4° on its base |
| security-camera | gesture | camera body | tilt 6° down, 4° up | dip 10°, hold, sweep 9° up, settle |
| window-blinds | result + reset | slats | cord pulled, blind rises one slat (top slat folds into the rail), tassel comes down | yanked up past its stop, drops back, settles |
| robot-vacuum | result + reset | vacuum | drives 2.6 into the dust, which vanishes; reverses | backs up 0.5, charges 3.6, rebounds, reverses |
| washing-machine | gesture, cycle 180° | laundry swirl | half turn | rocks back, 1.5 turns, machine shudders |
| refrigerator | gesture | lower door | swings past edge-on to hang open at the left (105°), shelves show, closes | flung open to 108° with a bounce, holds, slammed shut with a small rebound |
| oven | result + reset | heat waves | display lights, two heat waves rise in turn | display snaps on, waves shoot up and shimmer |
| smoke-alarm | gesture | alarm arcs | LED blinks, arcs swell 0.8 | puck buzzes, LED flashes three times, arcs 1.6 with a second pulse |
| water-leak | gesture | drop | drop falls, puddle spreads 1.2, new drop forms | drop swells heavy, falls fast, puddle splashes 2.6, new drop overshoots |
| power-outlet | result + reset | button | 0.6 press, status light on | hard 1.2 press with rebound, light pops to 1.2× |
| energy-usage | result + reset | bars | four bars rise in turn | the bolt flashes and jolts, then the bars shoot up in quick sequence |
| motion-sensor | result + reset | beams | two beams open in turn; a person rises into range | beams burst open; the person rises fast, overshoots and settles |

Mechanisms that avoid hiding outlines: blind slats fold into the head rail by fading below their rest position; the refrigerator's shelves are drawn only right of the swinging door's edge; the garage panels fade out as they reach the header; the water drop grows from its tip under the coupling.

## Observed failures and corrections

1. **Refrigerator handle (design review before build):** a handle at a fixed fraction of the door width crowded the hinge once the door squashed. It now rides 3.5 units inside the door's free edge, and Strong opens only to 45% (the handle keeps a ≥1 Bold gap to the hinge).
2. **Robot vacuum (design review):** the dust specks faded after the disc had already overlapped them. Their fade was moved earlier so they vanish as the bumper arrives.
3. **Clearance gate** (painted margin ≥2 and contract gaps), first build:
   - The motion-sensor target at Strong's 1.4× pop reached 0.53 from the outer beam. It moved right to x26, and the ripple radius was cut so it keeps the margin.
   - The oven heat waves sat 0.9 from the window. They were shortened (4.2 → 3.8) and centred: 1.1 each end.
   - The power-outlet LED crowded the button at its 1.5× pop (0.31). The button was shortened, the LED moved and resized, and the pop reduced to 1.2×.
   - The smart-speaker arcs' inward prep reached 0.9 from the body, and the Strong squash pinched the grille lines to 0.83. The prep was removed and the grille respaced 3.8.
   - The smoke-alarm arcs were staggered, so the inner arc caught the outer (0.23). Both now pulse together with no inward prep.
   - The window-blinds Strong downward tug reached 0.61 from the frame (the sill has only a 1-unit gap). The tug was removed; Strong's character is the fling past the stop.

4. **Native render, first full run:** PASS on all 62 actions. The composed styles and pose sheets then showed recognition faults that every numeric gate had passed:
   - **Smart lock** read as a remote control: the bar and 2×2 dots had no lock semantics. Turning the thumb-turn now throws a deadbolt out of the body's side.
   - **Ceiling fan:** three wide blades round a dot read as a radiation trefoil, and the Solid blades lost their shape. Now four slender capsule blades, cycle 90°.
   - **Garage door:** the top panel started at 67% opacity, because its fade window overlapped its start position. Panels respaced 1.25 below the header.
   - **Security camera:** a tall wall line plus the elbow arm read as a "4". Now a short wall plate centred on the arm.
   - **Window blinds:** the raised blind above the pane cross read as "±". The horizontal mullion was dropped.
   - **Robot vacuum:** Strong's full turn swung the bumper arc under the centre dot and drew a smiley face; the rest pose was ambiguous. Now a bumper chord and a lidar ring, and Strong's character is a charge, rebound and reverse.
   - **Refrigerator:** squashing the door toward its hinge hardly showed the interior. The door now swings past edge-on to hang open at the left, showing both shelves; the handle shows only while the door front faces us.
   - **Motion sensor:** dome, arcs and a dot read as a speaker broadcasting. The detected target is now a person, and two beams make room for it.
5. **Rebuild failures from those changes:**
   - The refrigerator's Solid door switched between a filled rectangle and a ring (MORPH_TOPOLOGY). It is now always a ring, with a clamped counter.
   - The motion-sensor person scaled from zero, which closed its head–shoulder gap (−0.82), and its 1.25× pop broke the margin. It now rises in rigidly and fades in.
6. **Second native render (the eight revised icons, PASS on 26 actions) and review:**
   - The ceiling fan's 3-wide capsule counters closed to slits at Bold. The blades are now 4.2 wide (axis r6.6–10.6), with the hub gap and margin kept.
   - Window blinds without the horizontal mullion then read as a "T" in a box. The blind now covers the top half with one slat line, and daylight (a sun) appears in the uncovered pane instead of mullions.
   - Smart lock, garage door, security camera, robot vacuum, refrigerator and motion sensor were confirmed readable.

## Reusable lessons

- **Recognition is a gate of its own.** Six of twenty icons passed every numeric and native gate yet read as something else: a remote, a radiation trefoil, "4", "±", a smiley, a speaker. Read the composed styles sheet at rest, and the Strong poses, asking what else it could be.
- **A rotating detail can assemble a face** with a fixed dot (an arc under a dot). Check mid-rotation poses as well as the rest.
- **Reveal, don't occlude:** draw background detail only where a moving panel has uncovered it (shelves right of a door edge, a sun below a blind edge), and hide it inside a fade window that clears the gap contract.
- **Scale-in from zero closes internal gaps** of multi-part marks (head and shoulders). Bring them in rigidly (rise plus fade) instead.
- **A Solid ring must stay a ring:** switching between a filled shape and a holed shape during motion changes topology. Clamp the counter instead.
- **A fade threshold must not overlap the rest position**, or the part starts partly transparent.
- **Hollow capsules need width ≥ Bold stroke + 1.5** to keep an open counter.

## Coordinator review round (v3)

The parent review of the final styles sheet found six icons that still failed "what else could this read as?". All six were redrawn to its spec:
- **smart-lock:** now a door-lock escutcheon. A tall rounded plate, a 2×2 keypad, and a lever handle projecting from a round rose. The lever presses down and back, and a status light comes on.
- **ceiling-fan:** now a side elevation (ceiling, downrod, motor, two long blades, light bowl). Rotation is shown by blade foreshortening (reach 8·|cos φ|). Strong spins 1.5 turns and wobbles on the rod.
- **video-doorbell / smart-speaker:** the doorbell is a tall narrow pill with a ring-and-dot lens and a large ring button (Solid ring). The speaker is a squat cylinder in side view, whose elliptical top is the light ring, with sound arcs above. The silhouettes are now clearly different.
- **security-camera:** a bullet body angled 14° down, a lens disc at the front, a round joint, and a straight short arm to a small wall plate.
- **window-blinds:** v3 (boxed frame, slats, side cord, sun) read as a document, and v4 (boxed frame, four slats, sun) as a tablet with a home button. v5 has no box: a head rail, five hanging slats and a side pull cord with a tassel. Raising folds the top slat into the rail as the tassel comes down; four slats stay down at rest.
- **Gates during the round:**
  - The camera lens sat 0.25 from its housing, so it moved inward.
  - The keypad dots were 0.7 apart, so they were respaced to 4.
  - The status light and the doorbell button overshoot were trimmed to keep their gaps.
  - The blinds cord and tassel sat 0.35–0.5 from the slats, so the slats were shortened.
- **Lesson:** a boxed frame with horizontal lines plus a bottom-centre dot reads as a device screen. A head rail with hanging slats and a side cord reads as a blind.
