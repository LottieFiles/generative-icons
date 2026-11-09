<!-- Generated from docs/motion-principles.md by `npm run docs:build`. Edit the source, not this copy. -->

# Motion principles

Design **cause → action → response → final state** before writing keyframes. Define the stationary structure, main driver, followers, anchors, hinges and protected gaps.

## Events, not nudges

An icon that moves without anything happening reads as decoration: the microphone that tilts a few degrees and comes back. Every action needs an event.

- **Find the event.** Ask what the object does for the user and animate that, not the object's mass: a microphone takes sound, send leaves, Wi-Fi gains signal, a shield blocks, a bell rings, a download lands.
- **Show the cause, briefly.** Sound arcs, sparks, steam, speed lines, trails and ripples may appear during an action when they are hidden at rest and gone when it ends.
- **Draw the meaning.** Marks that mean done, created or blocked (a check, a slash, a plus) draw on instead of moving in.
- **Rhythm, not a round trip.** Hit, rebound, settle: two or three beats, each smaller. The return is quicker and smaller than the action, never its mirror.
- **Sequence the parts.** The driver moves first and followers 60–120 ms later; repeated parts arrive in order.
- **Draw the subject once.** The motion transforms the object the rest pose shows (it moves, turns, scales, squashes or draws) and brings it back without leaving the canvas. Never swap it for a copy that fades in where the first was: the eye reads the object jumping back. Parts hidden at rest are cues, never the subject again, and no frame may jump.

## Light and Strong

Light plays the icon's event once, with one beat, and must read at 24 px. Strong must visibly change the primary action at normal speed and small sizes through travel, rotation, preparation, more beats, decisive timing and material response. A speed multiplier, extra pause or decorative follower is not sufficient.

Compare profiles at equal elapsed seconds and at 24/32 px. Keep common meaningful endpoints so switching profiles does not reverse the user's state. The current family uses 60 fps and a 2.4-second timeline; active duration is authored per icon.

## Material and mechanics

Hardware remains rigid around its pivots; the events around an object (sound, sparks, wind) are not material and may appear and fade. Paper feeds without arbitrary stretching. Cloth carries a traveling deformation. Ink follows its pen. Followers react to a cause; they do not all repeat a generic pulse.

Keep ordinary travel continuous through intermediate waypoints. Reserve full stops for deliberate contact, inspection or state changes. A physical impact can break velocity and start a follower response. Data values remain monotonic and must not overshoot for decoration.

## State and looping

Forward actions such as charging, writing and scanning retain their result. Author a separate preview reset or real reverse action when required. The gallery's loop is a preview policy, not permission to undo the product's meaning.

Continuous indicators need a complete periodic cycle, with position and velocity continuity and a useful reduced-motion pose. Do not insert a gesture end hold into an indeterminate loader. Check the playable-frame wrap; identical first/last playable frames are not required.

## Review

Watch native motion at normal speed before frame stepping. Inspect preparation, main action, contact, response, hold and reset. Verify attached joints and clear counters throughout the swept motion. Transparent resets must not inflate visible travel/speed measurements.
