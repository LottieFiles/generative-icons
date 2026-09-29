# Video editing repair and review

Scope: finish the existing 20 drawings and their Light/Strong forward, reverse and reset actions. Preserve icon IDs, timings and geometry unless a source or native check identifies a fault. No catalog change.

Family references: video-file (play mark inside a document), play-pause (common forward/reverse endpoints), microphone (attached capsule and cradle). Current packages and source geometry were read through generation-context before changes. The existing source comments describe each named part, pivot and motion driver.

## Observed failures and corrections

- Native export stopped at cut-clip: "cannot insert a MoveTo path command". Its blade contained two disconnected subpaths. Split the handle and stem into named parts, retaining the shared rigid transform and visibility. Export video's arrow had the same construction; split its stem and head. Preserve the original error in failed-export-initial.log.
- Source verification rejected voiceover-level-0 as fixed. That bar is intentionally animated. Correct the source constraint and independently test vertical bars, their fixed centre line and painted spacing in both profiles/directions. Do not change verifier thresholds.
- Native Cut clip styles showed a filled Solid panel hiding the completed cut and leaving two cap bumps. Use the same open panel construction as the other detail-bearing frames so the cut remains visible in Solid.
- Native Clapperboard styles showed no stripes in Solid. Keep its slate filled and use a ring plus solid stripe strokes for the clapper, retaining all Outline geometry and timing.
- Add exported-geometry checks for blade contact, trim-handle contact, export-arrow contact/clearance, waveform clearance and monitor-stand contact. Test every sampled frame in Bold and Solid, including reverse/reset actions.

## Per-icon motion record

| Icon | Directed Light and Strong action | State policy | Review scope |
| --- | --- | --- | --- |
| cut-clip | Light: the razor glides down through the clip and leaves a cut mark joined to both edges. Strong: it lifts, slams through with the clip dipping on impact, and the mark stays. | Retained result with preview reset | Both profiles; all actions and styles |
| trim-clip | Light: the trim handle drags the clip end 7 units to the left. Strong: a short backswing, then a fast pull with one bounded settle. The reverse drags it back out. | Toggle with explicit reverse | Both profiles; all actions and styles |
| transition | Light: the second frame slides up over the first into a dissolve overlap. Strong: it draws back, drives in with a hard stop and settles. Reverse separates the frames into a hard cut. | Toggle with explicit reverse | Both profiles; all actions and styles |
| speed-ramp | Light: the bead rides the speed curve steadily to the top. Strong: it hesitates, then accelerates up the ramp and arrives in 0.4 s. The ramped position is retained; preview reset rearms. | Retained result with preview reset | Both profiles; all actions and styles |
| reverse-clip | Light: the play triangle flips through its axis to point left. Strong: it stretches forward first, snaps through with a hard stop past the mirror and settles. Reverse flips it forward again. | Toggle with explicit reverse | Both profiles; all actions and styles |
| rotate-clip | Light: the 20×12 frame and its play mark turn a quarter turn clockwise into portrait. Strong: a small counter-wind, a fast turn past 90° and a settle back. Reverse turns it back. | Toggle with explicit reverse | Both profiles; all actions and styles |
| green-screen | Light: the backdrop frame retracts into corner brackets and its tint fades, leaving the keyed subject. Strong: the arms stretch out, snap in past the mark and settle. Reverse restores the backdrop. | Toggle with explicit reverse | Both profiles; all actions and styles |
| stabilize | Light: the frame jitters half a unit and settles around the fixed lock ring. Strong: it shakes 1.6 units with a 3.5° wobble, snaps level with a hit and locks still. | Gesture returning to rest | Both profiles; all actions and styles |
| thumbnail | Light: the thumbnail card lifts 0.7 and settles. Strong: it dips, lifts 1.5 with a 3° tilt about its bottom-left corner, lands with a hit and settles. | Gesture returning to rest | Both profiles; all actions and styles |
| export-video | Light: the export arrow hops 1.5 and eases back onto the clip. Strong: it presses down, launches 3.5, lands with the clip squashing 0.9 and settles. | Gesture returning to rest | Both profiles; all actions and styles |
| resolution | Light: the extra grid lines grow from the centre lines until the frame holds a 4×4 grid. Strong: verticals snap out first, then horizontals. Reverse retracts them to the 2×2 grid. | Toggle with explicit reverse | Both profiles; all actions and styles |
| aspect-ratio | Light: the widescreen frame narrows to 4:3 while its diagonal follows the corners. Strong: it stretches slightly, snaps narrow with a hit and settles. Reverse widens it again. | Toggle with explicit reverse | Both profiles; all actions and styles |
| mute-track | Light: the waveform bars sink to dots on the centre line. Strong: they swell 12% then slam flat. Reverse raises them, overshooting 15% before settling. | Toggle with explicit reverse | Both profiles; all actions and styles |
| voiceover | Light: the three level bars rise in a rolling wave and settle. Strong: the microphone bobs and the bars surge 4–5 units in a staggered snap before settling. | Gesture returning to rest | Both profiles; all actions and styles |
| screen-record | Light: the record dot grows to fill the ring. Strong: it pops past size with the ring flashing wider, then both settle. Reverse shrinks the dot away. | Toggle with explicit reverse | Both profiles; all actions and styles |
| clapperboard | Light: the clapper opens 8° on its hinge and drops shut. Strong: it opens 18°, holds, slams shut with the slate dipping on contact and rebounds once. | Gesture returning to rest | Both profiles; all actions and styles |
| merge-clips | Light: the two clips slide together and join. Strong: each draws back, drives in, bumps on contact and settles joined. Reverse pulls them apart. | Toggle with explicit reverse | Both profiles; all actions and styles |
| add-caption | Light: two caption lines extend smoothly along the bottom of the frame. Strong: they type in word by word with short holds. The caption is retained; preview reset clears it. | Retained result with preview reset | Both profiles; all actions and styles |
| video-filter | Light: the wipe line sweeps 14 units across the frame and the filtered region follows it. Strong: a short pull back, a fast sweep with a hard stop and a settle. Reverse sweeps it back. | Toggle with explicit reverse | Both profiles; all actions and styles |
| render-progress | Light: the render bar fills its track steadily. Strong: it fills in three decisive chunks with short holds. The finished bar is retained; preview reset empties it. | Retained result with preview reset | Both profiles; all actions and styles |

## Evidence boundary

Native build and verification results are recorded separately from sampled image review. Final asset hashes are in build.json and browser-verification.json. User visual acceptance is not claimed. No new shared design rule is needed: the existing single-MoveTo and Solid-interior rules cover these faults.
