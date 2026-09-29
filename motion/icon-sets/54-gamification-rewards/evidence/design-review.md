# Design and review record

Scope: complete the ten existing reward icons with ten new meanings. All drawings use the 32-unit family grid, the three native Outline weights, Solid construction, and tint on enclosed surfaces. The existing public IDs stay unchanged. The exact-ID manifest search found no matches outside set 54 for any of the twenty IDs before integration.

References: set 39 assembly and verification; `star` for rounded star points, `gift` for exposed rim/lid mechanics, `countdown-timer` for a retained time state, and `coffee-cup` for attached rigid parts. Bonus uses a multiplier ticket to distinguish it from Gift. Round countdown depicts a round-start clock and an elapsed hand state. Geometry is authored in this set.

| Icon | Drawing and driver | Light / Strong plan | State and protected geometry |
| --- | --- | --- | --- |
| trophy | Cup, ears, stem and base; pivot at base centre | 1.2-unit hoist / 2.6-unit hoist, 3-degree rock and landing | Gesture; keep ear and stem contacts |
| medal | Ribbon and star disc; pivot at ribbon top | 7-degree swing / 18-degree pendulum response | Gesture; star is a Solid counter |
| level-up | Fixed stairs and hopping ball | 7.6-unit rise / 8.6-unit spring and contact bounce | Retained level; explicit geometric preview reverse |
| xp-gain | Track, growing fill, falling star | Absorption then 40–85% fill / faster drop and drive | Retained progress; reverse; keep fill inside track |
| streak-flame | Outer flame and inner tongue anchored at base | 3-unit tip sway / 4-unit lean and 10% flare | Gesture; continuous topology |
| badge-unlock | Shield, lock, hinged shackle, star | 1.6-unit lift and 25-degree opening / snap and overshoot | Locked/unlocked; separately directed reverse |
| leaderboard | Stepped podium, dividers and crown star | Third, second, first rise / quicker rise and hard stops | Retained podium; geometric preview reverse |
| dice | Rounded die and three pips; centre pivot | 10-degree rock / 55-degree tumble and 2.6-unit jump | Gesture; exact zero-degree rest; pips rotate with die |
| game-controller | Rigid gamepad and separately pressed buttons | Button press and dip / decaying rumble and two presses | Gesture; Solid controls retain real counters |
| coin | Coin and star; vertical flip axis | One flip with 1.5-unit toss / two flips and 4-unit toss | Gesture; complete face returns |
| gem | Stone outline and facets; lower-tip pivot | 6-degree rock / back-rock and 13-degree turn | Gesture; facet joints follow rigid stone |
| treasure-chest | Fixed body, complete rim, lid hinged at (7,18) | 23-degree opening / 27-degree overshoot | Closed/open; directed close; hinge checked every frame |
| quest | Folded map and travelling marker | 0.78-second route / 0.42-second route plus 3-degree map response | Started/reached; reverse route; interior corridor stays open |
| achievement | Rosette, check, attached ribbon tails | 1.3-unit lift and stamp / larger tilted stamp | Gesture; one rigid assembly |
| star-rating | Large star at (16,13.5), three rating dots | 5-degree rock and sequential dots / 16-degree rock | One/three selected; reverse clears dots in reverse order |
| high-score | Crown and framed number-one arcade display; shared base pivot | 5-degree rock / prepared 11-degree rock and response | Gesture; crown/case gap is rigid; number stays legible in Solid ring |
| bonus | Notched multiplier ticket; centre pivot | 6-degree tilt / prepared 14-degree flick | Gesture; all glyphs follow the ticket |
| power-up | Fixed capsule, moving lightning bolt | 1.2-unit rise / charge dip and 3.2-unit surge | Gesture; minimum one-unit painted bolt/shell gap |
| lives | Main heart at (16,12), two fixed reserve hearts | One 8% beat / 16% beat, rise and smaller beat | Gesture; reserve hearts stay separate |
| round-countdown | Case, start tab, hand and hub at (16,17.5) | 270-degree sweep in 0.85 seconds / decisive 0.4-second sweep | Start/elapsed; directed rewind; hand/hub contact checked |

Construction review: preserve whole visible housings in each initial state. Solid containers use open rings where they hold moving marks. Closed cutouts use the family winding helper. All reverse endpoints must match their neighbouring forward states. Retained results are not undone inside a forward action.

Independent checks in `verify.mjs` preserve all inherited assertions and add chest-hinge contact, hand/hub contact plus case clearance, XP-bar inset, bolt/shell clearance, Quest marker/map clearance and crown/case clearance. They inspect exported world geometry across forward and reverse actions.

Verification evidence is stored beside this record. Source and native checks are separate from visual review and user acceptance. Final native package hashes are in the generated set manifest and browser report. User acceptance is not claimed.

First complete native review: 54 of 56 actions passed. Dice failed exact rendered rest equality because the 360-degree native matrix differed from zero at raster edges. The correction ends a 55-degree tumble at exactly zero; no tolerance or assertion was changed. The incomplete earlier run lost the server connection and is preserved separately as infrastructure failure evidence.

The saved style sheets also exposed crowding that passed the numeric gates. Quest now uses a filled traveller inside a wider map corridor. Achievement has an open ribbon attachment and a larger Solid window around a smaller check. High-score moves its crown and case with one rigid pivot, preventing contact; its crown is a filled symbol without small closed channels. The chest latch is a clean Solid counter. Before images remain in `round2-drawing-review/`.

Final status: all 20 icons and 56 native actions pass the requested source, build, native, token and full browser gates. The browser report matches every current package hash. Agent inspection covered native small-size sheets in both themes, the 192 px style sheets, the reverse-state sheets and equal-time Light/Strong poses. Exact evidence and limits are recorded in `visual-review.md`. The set download retains the stroke-width token documentation and includes the direct helper imports needed to rebuild its editable source.

The closed Solid chest review exposed a notch between separately rounded parts. Its body now has a full flat rim with a small overlap under the lid. The hinge contract also tests contact with the actual Outline boundary and containment within the Solid paint. The previous image and passing browser proof are preserved under `round4-drawing-review/` and `passing-browser-round4.json`.
