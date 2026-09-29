# Food & Drink (39): design and review record

Single-agent workflow (design → author → source checks → native render → visual review → correction). This is self-review only; it implies no independent approval or user acceptance. Package hashes: see `prototype/icons/39/assets/set.json` and `browser-verification.json` (full PASS, 50 actions, current build after the coordinator redraws).

Following the Weather (35) lessons. Shared helpers are in `batches/kitchen.mjs`: fixed-topology arcs, a Hermite curve builder, steam wisps, and a rise-and-recycle driver. No liquid, steam or toast relies on a transparent outline hiding another outline:

- Liquid surfaces end on the vessel's centreline.
- Toast is drawn only above the toaster top.
- The menu's right-page items appear only once the cover is past the fold.

## Icons, state policy and motion

| Icon | Policy | Main part | Light | Strong |
| --- | --- | --- | --- | --- |
| coffee-cup | gesture | steam wisps | rise 1.6, fade, recycle | shoot 2.4 in 0.3 s, lean as they rise |
| tea-cup | gesture | spoon | one stirring loop (foot slides along the rim) | backstroke, two quick loops, cup jiggles |
| wine-glass | result + reset | wine level | stream pours; level rises with a small slosh | fills in 0.28 s, glass dips 0.5, harder slosh |
| beer-mug | gesture | foam head | head swells 14% | mug raised 6° in a toast, head surges 16% |
| cocktail | gesture | glass tilt | tips 6°; drink stays level | draw-back, 14° hard stop, drink sloshes |
| cooking-pot | gesture | lid | bumps 0.9, tips 4° | jumps 2.2, tips 8°, double rattle, pot shivers |
| frying-pan | gesture | pancake disc | tossed 4.6, turns over once in the air | pan dips and flicks, toss 7.2, turns over twice, hard landing and bounce |
| kettle | gesture | steam jet | wisp rises 1.8 from the spout | kettle rocks on its base, jet 3.2 |
| toaster | result + reset | toast | slices rise, lever springs | body jolts, slices pop 1.8 past rest, bounce |
| blender | gesture | cross blade and vortex | 2 blade turns, shallow vortex | 4 turns, deeper vortex (1.2), appliance buzzes |
| chef-hat | gesture | hat | tips 4° off its corner | wind-up, hop 1.8 with 9° tip, squash landing |
| cutlery | gesture | fork and knife | tip 3.5° toward each other | spread, swing in, clink, rebound |
| pizza | gesture | slice tip | tip sags 1.2 | slice lifted 2, tip droops 3, rebounds |
| burger | gesture | top bun | hops 1.1 and lands | lifts 1.6, slams; patty squashes to 82% and widens |
| cake | result + reset | flame | flame grows from the wick | flares to 120%, sways, settles |
| ice-cream | gesture | scoop | stretch and squash | crouch, stretch 10% with a small hop, 84% squash, wobble (volume kept) |
| noodle-bowl | gesture | chopsticks | lift 1.6; noodles stretch from rim to pinch | dip, pull 2.6, bob |
| bread | result + reset | loaf | dough rises to a domed loaf; scores open | springs 10% past full and settles |
| recipe-book | gesture | ribbon | book taps; ribbon swings 14° | book hops 1.4, ribbon swings 30° with damped return |
| restaurant-menu | result + reset | cover | cover swings open about the centre fold | lifts, swings 8% past flat, settles |

## Observed failures and corrections

1. **Source verify: still after rest.** Wine-glass Strong's windowed slosh ended at 0.72 s, after its 0.66 s active time. Strong's active time went to 0.74 s. (The first `sed` edit silently did not match. The gate caught it on the rerun.)
2. **Clearance gate** (painted margin ≥2 and contract gaps):
   - The blender blade sat 0.5 from the jar's bottom edge, so it was raised to y17. The Strong vortex went 4 → 3.2 so it clears the blade.
   - The bread scores overlapped the dome at rest and when squashed, so they moved inward and now grow from their midpoints as the loaf rises.
   - The coffee wisps crossed the top margin on the Strong rise, so the rise went 3 → 2.4.
   - The ice-cream scoop crossed the top margin (stretch plus hop), so the stretch went to 1.1 and the hop to 0.5.
   - The pizza toppings sat 0.47–0.95 from the slice edges, so they moved inward.
   - The burger's gaps dipped to 0.93, so the slam and rebound were reduced.
   - The wine stream touched the sloshing surface, so it now stops 3.4 above it.
   - The cutlery rule measured an intended tine joint, so the rule was removed.
3. **Native render: "Duotone policy disagrees with the full action"** (frying-pan). At Bold 24 px, the 2.4-unit pancake's tint was entirely under its stroke, so Duotone equalled Outline. The pancake is now 3.4 tall.
4. **Visual review of native styles and pose sheets:**
   - The coffee wisps read as S-blobs at Bold and Solid. The mug moved down 1 unit, and the wisps are longer (4.6) and calmer (0.9 wiggle).
   - Ice-cream's waffle line, 2.7 below the cone's top edge, fused with it at Bold. It moved to 3.6.
5. **Solid planned from the Weather lessons:**
   - Blender base, menu pages and recipe-book cover are rings, so the button, items and spoon stay visible.
   - Pizza and the burger top bun cut toppings and seeds as real counters.
   - Beer ridges, bread scores, crease, icing, crust and waffle lines are Outline-only detail.
6. **Collector port:** 4196 was already held by another process, so evidence was not saved. The collector moved to 4206.

7. **Coordinator review of the styles sheet (redraws):**
   - **frying-pan:** the pan was tiny and shallow, and the pancake read as a pill. The pan was redrawn at family scale: a 10-unit-deep body across x 3.8–21.2 with its handle to x 28. The pancake is now a side-on disc (8.1 × 3.5) that turns a full revolution in the air, twice on Strong. It turns only while high enough to clear the pan, and the angle snaps to the exact start drawing at whole turns (a float residue from a 360° turn failed "reset changes visible geometry").
   - **blender:** at rest it read as a "T" in a box. It is now a tapered jar under a lid band, on a filled motor base with a button counter, with a cross (X) blade low in the jar. The Strong vortex was reduced to 1.2 so it clears the blade.
   - **bread Solid:** the scores vanished. Solid now cuts them as full-length capsule counters. The loaf's own squash keeps them inside it, while Outline scores still grow in as the loaf rises.
   - All gates were rerun after the redraw; the final native PASS covers 50 actions on current hashes.

## Reusable lessons

- **Duotone needs a surface wider than the Bold stroke.** An enclosed shape under about 3 units tall renders identical to Outline at Bold, and the native Duotone check rejects it. Size small tinted parts (pancakes, pellets) at least 3.4 across.
- **A liquid surface must end on the vessel's centreline and be drawn inside that vessel's frame.** Its tint region is then the vessel segment below the surface. Keep a stream or pick ≥1 away from the sloshing surface, not just its rest line.
- **A decorative stroke parallel to an edge needs a centreline spacing of 3.5** (Bold 2.5 + 1). Otherwise it fuses at Bold.
- **Details that squash with their parent** (scores on rising dough) should grow in with the parent, not scale with it.
- **Check that a scripted substitution matched** (`grep -c`) before rebuilding.
- **Check that the collector port is free** before relying on saved evidence.

## Remaining concerns

- The closed restaurant menu, its prepared state, sits right of centre, because the cover opens about the centre fold.
- Normal-speed viewing and user acceptance are outstanding.
