# Design and review record

Scope: a new set of 20 emoji, chosen from Unicode's emoji frequency ranking (2021), with duplicates in a one-colour set skipped (♥️, ☺️). No library icon drew any of them; `emoji-picker` (set 33) supplied the face construction.

Method: the lead drew and built a calibration face (`smiling-face-with-smiling-eyes`) and a kit sample (grin and tears) to fix the shared geometry and the Solid construction, then four agents authored five icons each as gai-icon/2 specs with `context`, `check`, `draft`, `build` and the build's player sheets, and the lead reviewed every Strong sheet and asked for fixes. The builds were adopted with `library adopt 64 --add`.

| Icon | Light | Strong | Notes |
| --- | --- | --- | --- |
| 😂 `face-with-tears-of-joy` | The head kicks up with a laugh and lands, and a droplet spurts from each tear down to the corners. | A dip, then two laugh beats with the head tipping; the grin opens and each beat throws a pair of droplets. | Tears moved out and the grin narrowed slightly so they stay 0.64 apart at Bold. |
| ❤️ `red-heart` | A lub-dub: the heart squeezes, swells, swells again smaller, and three rays flick out above it. | A deeper squeeze, a bigger beat and a third smaller one, with the rays reaching farther. |  |
| 🤣 `rolling-on-the-floor-laughing` | The tilted face rocks back and rolls one full turn, flinging a droplet off each tear. | A wind-up, a faster turn that hops and travels, a bouncing landing, and droplets flung to all four corners. |  |
| 👍 `thumbs-up` | The fist dips and tips back, then jabs up with the thumb leading, rebounds and settles. | A deeper wind-up, a higher jab tilting forward, two rebounds, and two sparks drawing on beside the thumb. |  |
| 😭 `loudly-crying-face` | The mouth opens in one wail while a drop runs down each tear stream and drips off the jaw. | Two sob beats dip the head, the mouth wails wider on each, and two waves of tears drip off. | In Solid the drop running down the stream is hidden; the drip off the jaw shows in every style. |
| 🙏 `folded-hands` | The palms press together, bow forward, and three glow rays appear above the fingertips. | A deeper bow and a second smaller one, with the rays rising farther. | At small sizes the outline is close to a pointed arch; the sleeves and rays carry it. |
| 😘 `face-blowing-a-kiss` | The lips push and the heart floats up and away as a new one fades in at the lips. | The face draws back, leans 7° into the kiss, and the heart flies higher on an arc with a turn. | Face r 10 shifted left. In Solid the flying heart shows only past the rim; the rim gap is empty about 0.16 s. |
| 🥰 `smiling-face-with-hearts` | Three hearts pop in turn round the head as the smile widens. | Each heart dips and pops bigger, the ring drifts round the head and back, and the face tilts with a double settle. | Face r 9.5. At 24 px the small hearts are 3–4 px. |
| 😍 `smiling-face-with-heart-eyes` | The heart eyes beat once (a swell and a smaller second swell) as the grin opens. | The eyes draw in, pop out bigger and beat twice while the face lifts and the grin opens wider. | Solid: the eyes beat, the grin stays still (one moving cut-out per face). |
| 😊 `smiling-face-with-smiling-eyes` | The smile widens and blush fades in on the cheeks, then out. | The smile widens further as the face lifts and tips 6°, the blush lingers, then a small settle. | Blush is filled ovals, Outline and Duotone only. |
| 🎉 `party-popper` | The cone kicks back, fires its confetti out along arcs and the streamer draws out again. | The cone aims, kicks harder and rebounds; more pieces fly farther, with a second smaller burst. |  |
| 😁 `beaming-face-with-smiling-eyes` | The wide grin spreads from its centre and the face lifts a beat later, then relaxes. | The face dips, pops up and lands with a short settle while the grin spreads wider. | No teeth line: it closes at Bold and 24 px, so the grin is wide and flat instead. |
| 💕 `two-hearts` | The big heart beats, then the small one rises, tips and lands. | The big heart dips and rises with a beat; the small one floats higher, swinging, and lands softly. |  |
| 🥺 `pleading-face` | The big eyes swell and the brows lift, then settle. | The eyes squeeze, then swell bigger with the brows higher; a tear wells under one eye and the face quivers. | The frown can't tremble on its own (one moving cut-out); the tear doesn't show in Solid. |
| 😅 `grinning-face-with-sweat` | A sweat drop runs down the temple and drips off as a new bead fades in, with two nervous chuckles. | The face winces first, then the drop runs farther round to the cheek and drips off, with three shrinking chuckles. | In Solid the running drop shows as a bump on the face's edge. |
| 🔥 `fire` | The inner flame surges, the three tongues lick up in turn, and an ember rises off the top. | A bigger surge, the whole flame leans and licks twice, and two embers rise. |  |
| 🤦 `person-facepalming` | The hand lifts off the face and slaps back; the head nods back with the impact. | The hand presses in, lifts further and slaps back hard; the head recoils and two impact ticks flash off the hand. | Face r 10.3 moved down. Solid shows a plain mitten; Duotone leaves a thin untinted band at the thumb. |
| 🤷 `person-shrugging` | Both palms pop up as the head dips and tilts. | The arms dip first, the palms rise higher with a bounce, the head tilts further and the eyebrows lift. | Face r 7.8 so the kaomoji arms fit. |
| 🙄 `face-with-rolling-eyes` | The pupils roll one full circuit round the eyes as the head tips back. | The pupils wind back, roll past a full turn and settle; the head tips further and the mouth slides aside in a sigh. |  |
| 😆 `grinning-squinting-face` | The chevron eyes squeeze shut as the face hops once and lands. | Three shrinking laugh hops with the eyes squeezed and laugh lines flicking out at the landings. | The grin opens on each hop in Outline and Duotone; Solid keeps a still mouth. |

Construction findings:

- Solid allows one group of moving cut-outs per face (`KNOCKOUT_RIGS`, `KNOCKOUT_TRACKS`). Where a face needs two moving features, the moving one is drawn only in Outline and Duotone over a still Solid-only cut-out (`smiling-face-with-heart-eyes`, `grinning-squinting-face`).
- A knocked-out part is not drawn in Solid. Tears, hands, hearts and sweat on the rim are filled parts with a `gap` cut in the face, which is computed at rest, so the moving copies are hidden-at-rest parts that cross-fade with the resting one.
- The kit's first grin and tears overlapped at Regular and Bold. The tears moved out 1.3 and the grin narrowed 4% (0.64 painted gap at Bold), within the 1.4 margin through `rolling-on-the-floor-laughing`'s Strong roll.
- A teeth line inside the beaming grin closes at Bold and 24 px; that grin is wide and flat instead.
- Flung tears drawn as outlined drops blob at Bold; they are filled droplets flown to the canvas corners.

Review: every icon's build sheets (Light and Strong, Outline 1.5/2/2.5, Solid, Duotone, both themes) were opened by its author and by the lead. `library preflight`, `library verify` (20 adopted spec icons, 40 actions) and `library verify-visual` (40 actions at every size, style, weight and theme) pass. User acceptance is not claimed.
