# Education (38): design and review record

Single-agent workflow (design → author → source checks → native render → visual review → correction). Self-review only; no independent approval or user acceptance is implied. Package hashes: `prototype/icons/38/assets/set.json`. The final `browser-verification.json` covers all 68 actions and matches every current package sha256.

References: Weather (35) construction and lessons. Avoided duplicates: `book`, `document`, `edit` (pencil line), `compass` (navigation), `calculator`, `checkbox`, `multiple-choice`, `progress-steps`, `translate`. Letters are vector strokes (no fonts). Assembly helpers: `batches/edu.mjs` (`container` = tint + frame + Solid ring, `pen`, `reveal`, `drawPolyline`).

## Icons, state policy and motion

| Icon | Policy | Main part | Light | Strong |
| --- | --- | --- | --- | --- |
| graduation-cap | gesture | board | lift 1.4, tilt 4°, tassel swings | dip, toss 3.1 with 11° tilt, bounce; tassel swings wide |
| chalkboard | result + reset | chalk line | writes a wave, then underlines | writes in 0.22 s, underline snaps, board jolts |
| flashcards | result + reset | card | flips edge-on from "?" to "A" | lifts off the deck, flips in 0.26 s, lands |
| quiz | result + reset | answer marker | a dot grows in the first answer's ring | dot snaps in, marker pops 12% |
| certificate | result + reset | seal | seal grows on; tails unfurl | seal slams from 1.9×, paper jolts, tails snap out |
| notebook | result + reset | note lines | three lines written in turn | picked up and set down; lines dashed off in 0.09 s each; taps down |
| pencil | result + reset | pencil + ink | writes a wave; tip rides the ink | lift and drop, writes in 0.3 s, flicks up |
| ruler | gesture | ticks | measuring pulse, 0.6 per tick | faster pulse of 1.5 while the ruler rocks 2.5° |
| lab-flask | gesture | bubbles | three bubbles rise up the neck; surface ripples | flask swirled ±5°, slosh, bubbles race up in 0.34 s |
| microscope | gesture | tube | racks 0.4 toward the stage and back | lifts, drives 0.8 with a hard stop, eases back |
| abacus | result + reset | capsule beads | four beads counted across in turn | each flicked across in 0.1 s, knocks to a stop with recoil |
| school-bell | gesture | bell | shaken ±8° from the grip; clapper lags | back-swing, hard ±15°, clapper rings wide and damps |
| course-progress | result + reset | scrubber | advances to 75% | play button pressed first, then drives in 0.24 s |
| lecture | result + reset | pointer | raises to the screen; two bullets appear | two sharp taps, each snapping a bullet in |
| library-shelf | gesture | middle book | drawn 2.2 up and slid back (right book leans 8°) | yanked 3.6, pushed home with a knock and rebound |
| language-learning | result + reset | bubble | bubble with "A" grows from the book | pages flap; bubble pops wide (120%), tilts, settles |
| alphabet-blocks | result + reset | A block | lowered onto the L and T blocks | falls turning 10°, double bounce, blocks below thud |
| homework | result + reset | tick | tick drawn on the worksheet | sheet slapped down, tick in 0.14 s, sheet hops |
| exam-grade | result + reset | ring | hand-drawn ring circles A+ | ring struck in 0.26 s, grade pops 6% |
| geometry-compass | result + reset | arc | swivels 6° on its needle while an arc is drawn beneath | cocks 9°, holds, snaps upright as the arc is struck in 0.16 s |

## Observed failures and corrections

1. **Import:** `arcN` is not exported by `helpers.mjs`; import it from `sky.mjs`.
2. **Margin (preflight hull warning):** Strong's 125% bubble pop in language-learning would leave the canvas. It now pops wide (120% × 102%), the tilt is 3°, and the bubble moved down 0.5.
3. **Clearance gate:**
   - Abacus: Strong's bead overshoot reached 0.53 from the frame, so the right slots moved in 0.5.
   - Alphabet-blocks: the falling block became visible above the 2-unit margin, so its fade-in starts lower.
   - Exam-grade: the ring grazed the A (0.38, then −0.41 in Strong), so the grade is drawn at 88% and pops 6%.
   - Flashcards: foreshortened marks crossed the card edges, so the marks show only while the card is at least 60% wide.
   - Graduation-cap: the tassel swung into the band because a positive angle is clockwise. Strong's big swing now goes outward.
   - Lecture: the bullets were 0.5 from the screen edges, so they were shortened.
   - Library-shelf: Strong's 6° book tilt reached its neighbour (−0.25), so the book now moves straight.
   - School-bell: the clapper reached 0.36 from the body, so its swing was reduced.
4. **Still after rest:** school-bell's Light clapper lag ran past rest, so Light's active time was extended to 0.64.
5. **Light/Strong:** alphabet-blocks was flagged (speed ratio 1.09). Light is now a smooth lower; Strong is a fast drop with a bounce.
6. **Native render:** "Insufficient animation states: 15" for Light alphabet-blocks at 24 px, because the block was hidden for too much of its drop. Light now fades in from −4.5; Strong keeps −3.5 because its turning block needs the headroom.
7. **Visual review of native sheets:**
   - Flashcards ended as a square with a check, which reads as the existing checkbox. It is now a landscape card whose answer side shows "A".
   - Alphabet-blocks: the small C and I closed into blobs at Bold, so the letters are now L and T, open strokes with no counters.
   - Graduation-cap: the cord fused with the board edges near the tip at Bold. The cord was removed and the tassel hangs from the board's edge.
   - Geometry-compass ended tilted with the arc hugging the pencil leg, which read as tweezers. It now inks the arc, stands back up, and the legs are spread to R12 for a longer arc.

8. **Coordinator review of the styles sheet: four icons redrawn before registration.**
   - **quiz:** the buzzer dome read as a hat or bowl. It is now a quiz card: a stroke "?" above two answer rows (ring plus line). The first answer's dot grows in and is retained; Strong pops the marker. Gates added: rows 6.7 apart, and marker, row and card gaps ≥1 including the pop. The card was widened after a 0.4 ring-to-card gap.
   - **abacus:** the round r1.7 beads fused onto the rods at Bold and read as a list. Beads are now 3.6 × 5.5 capsules standing 1.5 clear of the Bold rod stroke on each side, spaced 4.6 (≥1 painted gap) with ≥1 to the frame, on 3 rods with 3/3/2 beads. Strong's knock recoils away from the frame instead of overshooting into it. Strong's active time is now 0.52.
   - **geometry-compass:** it read as a bent stick or hook. It is now a drafting compass: hinge disc and handle, straight legs spread 40°, a needle point, and a collared pencil tip. The arc is a separate stroke on the paper beneath both tips, at radius 20.6 about the hinge, ≥1 clear of each tip. Lesson: an arc concentric with the needle is nearly tangent to the pencil leg at its tip, so it reads as part of the leg.
   - **library-shelf:** the spine bands fused at Bold and Solid. The books are now 4.6 wide with one band each, 3.5 below the top, and the right book leans 8° on its corner.

## Reusable lessons

- Positive rigid angles are clockwise (y down), so a hanging follower's positive swing moves its lower end left. Check the swing direction against nearby parts.
- A card flip (scaleX) squeezes its face marks toward its edges. Hide face content while it is foreshortened below about 60%.
- The render's minimum distinct-state count counts only visible change. A part hidden for most of a short Light action fails it.
- A rest pose can collide with an existing icon's meaning (a square with a check is a checkbox). Compare the rest pose against the catalog, not just the motion.
- Scale-up pops need headroom. Near the canvas edge, pop wide (anisotropic squash-stretch) instead of tall.

## Remaining concerns

- Geometry-compass: the arc sits beneath the tips rather than exactly on the pencil's path. That is an iconic simplification chosen for legibility.
- School-bell is a hand bell. Its handle distinguishes it from the notification `bell`, but the families are related.
