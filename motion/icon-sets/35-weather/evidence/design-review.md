# Weather (35): design and review record

Single-agent workflow (design → author → source checks → native render → visual review → correction). Self-review only; no independent approval or user acceptance is implied. Package hashes: see `prototype/icons/35/assets/set.json` and `browser-verification.json`.

References: `cloud`, `sun`, `moon` (05-interface), `cloud-sync`. The family cloud path is reused unchanged in shape, placed by one uniform scale and offset (`batches/sky.mjs`).

## Icons, state policy and motion

| Icon | Policy | Main part | Light | Strong |
| --- | --- | --- | --- | --- |
| partly-cloudy | gesture | ray crown | rays brighten 0.35 outward; cloud drifts 0.2 | rays flare 0.8 and turn 16°; cloud edges in 0.4 and settles |
| rain | gesture | streaks | fall 2.6, fade, recycle while invisible | cloud lifts (wring), streaks shoot 4.4 |
| snow | gesture | flakes | drift 2.2 with sway | fall 3.0 with wider, faster sway |
| hail | gesture | pellets | drop to ground, rebound 1 | 0.14 s drop, rebound 2.7 |
| thunderstorm | gesture | bolt | double flicker, dip 0.5 | hard double flash, strike 1.1, cloud recoil |
| snowflake | gesture, cycle 60° | whole flake | 60° turn | −10° wind-up, 126°, settle at 120° |
| wind | gesture | gust lines | push 1.1 in stagger | draw back 0.6, drive 3, damped return |
| fog | gesture | layers | parallax 0.8 | pull back, slide 2 |
| tornado | gesture | funnel bands | one travelling sway, tip 1 | wider 1.5-cycle wave, tip 2.5 |
| hurricane | gesture, cycle 180° | whole storm | half turn CCW | 14° wind-up, full turn CCW, settle |
| rainbow | result + reset | bands | draw 0.30 s each, calm stagger | draw 0.16 s each, tight stagger, bands pop 0.6 |
| sunrise | result + reset | sun cap | rises from below a real horizon; rays grow | overshoots 1.1, rays flare 1.3× |
| sunset | result + reset | sun cap | sinks from high to half; rays shorten | lifts first, drops 0.9 past, settles |
| moon-phase | result + reset | terminator | crescent → gibbous | 0.28 s sweep while the moon swings 9° |
| temperature | result + reset | mercury | rises steadily | bulb swells, column shoots up in 0.24 s |
| humidity | result + reset | water surface | level rises, small slosh | 0.34 s rise, strong slosh (windowed to end) |
| uv-index | result + reset | meter dashes | fill 0.16 s each; rays lengthen | pulse, 0.09 s steps, rays flare |
| umbrella | gesture | canopy | flexes in and springs back | half closes, snaps open 6% wide |
| windsock | gesture | sock | lifts 12° with ripple | sags, blown horizontal with travelling ripple |
| weather-radar | result + reset | sweep arm | one turn; cell appears as the arm passes | kick-back, faster sweep, cell pops with ripple |

Sunrise and sunset draw only the part of the sun above the horizon (a fixed-topology arc whose ends ride the horizon). Nothing relies on a transparent outline hiding another outline.

## Observed failures and corrections

1. **Snowflake:** at Bold, branch legs fused each arm end into a blob (native styles sheet). Branch vertex moved from r6.4/±42° to r5.4/±48° (painted gaps ≈1.1 and 1.3). Gap contract added.
2. **Symmetric index turns:** `verify.mjs` rejected snowflake's 60° end ("reset changes visible geometry"). Declared `cycleDegrees` (60 snowflake, 180 hurricane), as sun and gear already do.
3. **Clearance gate** (`verify-clearance.mjs`, painted margin ≥2 and contract gaps): fog layers 0.5 apart at Bold, so spacing went to 3.6. In partly-cloudy, Strong's gust closed the sun–cloud gap to 0.15 and flared rays reached a 1.1 margin, so the sun got smaller, the drift gentler, and the rays a turn. Rain's cloud rebound dip reached 0.86, so the dip was removed.
4. **Still after rest:** snow and hail finished their last fade-in after rest; sunrise's staggered rays and humidity's exponential slosh never reached exactly zero. Active times were lengthened, and the slosh is windowed to end at its active time.
5. **Light/Strong too similar** (`measure-motion.mjs` ratio < 1.5): rainbow, uv-index and hail. The Strong main drive was made decisively faster, with real preparation and response.
6. **Native render: "Invisible main action"**. Rainbow's prepared state painted nothing, so its reset had no visible colour override. A fixed horizon now keeps the prepared state visible and grounds the drawing.
7. **Visual review of native styles:** fixes found by eye after all numeric checks had passed.
   - Hail's ring pellets closed at Bold, so they became filled marks.
   - The thunderstorm's filled polygon bolt was too small and closed at Bold, so it became a zigzag stroke.
   - Moon-phase ended on a straight terminator that read as a divided circle, so it now ends waxing gibbous.
   - Windsock's Solid stripe caps poked outside the sock, so the stripes are now Outline-only.
   - Weather-radar's Solid filled the whole scope and hid the arm and cell, so Solid uses a ring.
   - At rest, weather-radar read as a clock face (hand at 12 plus a dot). A fixed range ring now carries the storm cell.

## Reusable lessons for new sets

- A stateful prepared state must still paint visible housing (a horizon, rim or container). Every forward frame must paint.
- A container that holds moving detail needs a Solid **ring** (`annulus`), not a filled disc.
- Small closed marks (bolts, pellets) close at Bold. Use centreline strokes or filled marks.
- Stroke detail inside a filled Solid silhouette should have no Solid variant, or its caps poke out.
- An exponential or staggered follower must end inside the active time. Window decays.
- Keep centrelines within 3.3–28.7 (Bold radius 1.25, Solid 1.3, margin 2), including Strong's overshoot.
- Fix a `measure-motion` Light≈Strong flag by changing the main drive, not the followers.
- Numeric gates pass faults that the native styles sheet shows immediately. Always compose and read the styles and pose sheets.
