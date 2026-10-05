# Author an icon as a spec

A v2 icon is one JSON file, `gen-animated-icons/icons/<id>.icon.json`, with `"format": "gai-icon/2"`. It is data: shapes, roles, rigs and eased keys. Nothing in it is a function of time, so there is no pose math to write and no SVG to hand-build. The hosted engine builds every motion from it in one call: native shapes, rigs chained by layer parenting, eased keys, masks, mattes, trims and markers. `create` writes one by default. v1 JS batches (`create <id> --batch`, `batches/*.mjs`) keep working for what a spec cannot express yet (the last section); a command takes the spec path only when every icon it names has a spec.

```sh
gen-animated-icons create <id>            # a starter spec that checks clean
gen-animated-icons create <id> --from <library-id>   # start from a library icon's spec
gen-animated-icons check <id>              # offline: structure, names, motion, lints (milliseconds)
gen-animated-icons draft <id>              # one engine call: a review sheet, nothing stored
gen-animated-icons build <id>              # one engine call: export, slots, themes, .lottie, verify, install
```

Fix every `check` error before you draft. A draft is about 4 s and stores nothing, so draft after each change you cannot judge from the numbers; build once the draft looks right. `check` and `draft` never need a build first.

Over 400 library icons are specs that passed every check below. When one is close to what you need (the same object, or the same kind of event), start from it: `create <id> --from <library-id>` copies its parts, rigs, motion, state and reverse under your id. `search` marks those icons `"spec": true`, and every library carries their specs. Redraw what the new meaning needs, rewrite the intent, rename the state and labels, and keep the craft: the event, the beats, the clearance and the endings.

## Conventions

| Item | Rule |
|---|---|
| Grid | 32 units, y down. A point is `[x, y]`, an anchor name, or `{"from": P, "angle": deg, "len": r}`. |
| Angles | Degrees, positive = clockwise on screen. Polar and arc angles: 0 = +x (3 o'clock), -90 = up. |
| Times | Seconds on the 2.4 s timeline. Fractional times are kept. |
| Values | `x`, `y`, `rotate`, `r`, `grow`, `amp` are offsets from the drawn pose (rest 0). `opacity` and `draw` are absolute 0–1 (rest 1; a part's `"opacity"` sets the opacity it rests at, `"hidden": true` rests it at 0). `scale` and `squash` are factors (rest 1). |
| Draw order | The order of `parts`, back to front. |
| Comments | A `"//"` key on the spec, a part, a rig, a track (a `follow` included) or `reverse`. |

The drawing is the rest pose. A gesture ends where it starts; a stateful icon (`state`) ends changed.

## Motion that reads

Give every action an event, not a nudge that comes back:

- Animate what the object does for the user: a microphone takes sound, send leaves, Wi-Fi gains signal, a bell rings, a download lands.
- Show the cause briefly with parts hidden at rest (`"hidden": true`, then key their `opacity`): sound arcs, sparks, speed lines, a trail, a ripple. Hide them again before the action ends.
- Draw marks that mean done, created or blocked (`draw`) instead of moving them in.
- Hit, rebound, settle: two or three beats, each smaller (`hit`, `settle`, `bounce`). Make the return quicker than the action.
- Move the driver first and followers 60–120 ms later (`follow` with `lag`).
- Light plays the event once; Strong adds beats or travel. Think 1.5–4 units of travel or 10–40° of turn: a move you cannot see at 24 px does not count.
- Keep every part at least 1.4 units inside the canvas at Bold in every frame, travel and overshoot included: at 24 px, ink on the outer pixel fails the visual check `build` runs. `check` warns within 1.4.
- A reading stops dead on its value: a gauge needle, a level, a score or a chart point never overshoots what it reads. Strong adds a step (a second click, a second milestone) instead of travel; `LIGHT_STRONG` may warn, and that is expected.
- A turn of a shape that looks the same turned (a hexagon, a gear, a ring of dots) is invisible: give it a press, a pop or a cue around it instead.
- A squash about a line reads as a turn or a flip (a key, a coin) at large sizes, and as a squeeze at 24 px: confirm it on the 24 px sheet.
- A part that leaves and comes back (flies off, then a new one arrives) is two parts: cross-fade the leaving copy and the arriving one, so no frame is empty.
- Transient parts must read when they show: a spark, drop or arc under 2 px at 24 px is a speck (`SMALL_AT_24`).
- Icons that do the same thing get the same treatment: every arrow of a set, every lock, every selector.

## Parts

```json
"parts": {
  "body":  {"shape": {"circle": [[16, 17.5], 8.5]}, "role": "surface", "solid": "ring"},
  "hand":  {"shape": {"line": ["dial", {"from": "dial", "angle": -30, "len": 2.3}]}},
  "feet":  {"shape": [{"line": [[10.5, 23.5], [10.5, 26]]}, {"line": [[21.5, 23.5], [21.5, 26]]}]},
  "bell-r": {"mirror": {"of": "bell-l"}}
}
```

| Shape | Form |
|---|---|
| `line` | `{"line": [P, P]}` |
| `polyline` | `{"polyline": [P, …], "r": corner}` (up to 96 points) |
| `rect` | `{"rect": [x, y, w, h], "r": r or [tl, tr, br, bl]}`. Starts at the bottom-left corner, clockwise. |
| `circle`, `ellipse` | `{"circle": [P, r]}`, `{"ellipse": [P, rx, ry]}` |
| `arc` | `{"arc": [P, r, from, to], "closed": true}` adds the chord |
| `polygon` | `{"polygon": [V, …], "r": corner}`; a vertex is a point or `{"p": P, "r": corner, "curve": [P, P]}` (up to 96) |
| `wave` | `{"wave": {"x": [x0, x1], "y": y, "cycles": n, "amp": a, "phase": p}}` |
| `path` | `{"path": "M… Z"}`: SVG data with ONE subpath (one M, Z only at the end) |
| `morph` | `{"morph": [A, B], "by": "<driver>"}`: two shapes with the same command list; key the driver 0 → 1 |
| list | `"shape": [S, S, …]`: several subpaths painted as one layer per style |

A part rests visible, translucent (`"opacity": 0.3`, which its opacity track starts and, for a gesture, ends at) or hidden (`"hidden": true`). `"weights": {"light": S, "bold": S}` gives the Light and Bold outlines shapes of their own, for a dot that grows with the weight (`shape` draws Regular, Solid and tint; such a part moves only with its rig).

`mirror` copies another part across x = 16 (or `"x"`/`"y"`), with its options unless you set them. Anchors: `"anchors": {"name": [x, y]}`, plus derived ones on every part: `<part>.c`, `.top`, `.bottom`, `.left`, `.right`, `.tl`, `.tr`, `.bl`, `.br` (bounding box at rest) and `.start`, `.end` (open shapes). Polar points replace hand trigonometry: a hand at 2 o'clock is `{"from": "dial", "angle": -30, "len": 4}`.

### Roles: one drawing, every style

Each part expands into the style layers the runtime switches: outline at 1.5 / 2 / 2.5, Solid, and the Duotone tint. Never draw a style twice.

| Role | Outline | Tint | Solid (`auto`) |
|---|---|---|---|
| `stroke` (default) | stroked; `"outline": "none"` draws none, so the part shows only in Solid | none | the centerline at 2.6 |
| `surface` | the frame stroked; `"outline": "open"` leaves out the closing edge, `"none"` draws none | the interior (`"tint": false` for none) | filled, plus a 2.6 rim |
| `fill` | filled in every weight | none | filled |

`"solid"` also takes `"ring"` (the outline at 2.6, hollow), `"fill"`, `"none"`, `{"grow": d}` (fill plus a 2d rim; d > 0) or a shape of its own.

### Holes, knockouts and gaps

- `"holes": [S, …]`: real counters. They are stroked in Outline and cut from tint and Solid by subtract masks, so overlapping counters stay open. On a part that `scale` or `squash` reshapes, the masks follow.
- `"knockout": ["hand", {"part": "knob", "gap": 0.8}]` on a surface: in Solid, those parts are cut out of it with a clearance (default 0.6, or the part's `"gap": number`) instead of drawn. A part that sits still on the surface's rig is cut by masks. One that moves on another rig, is reshaped, draws on, fades or starts hidden is cut by a matte that follows it, trim included, so the cut-out draws on with the part. A surface's moving knockouts must share one rig (`KNOCKOUT_RIGS`), and a part with its own trim or visibility must be its surface's only moving knockout (`KNOCKOUT_TRACKS`).
- A morphing knockout is cut by a matte that follows the morph, when it is its surface's only moving knockout. If another part already holds the matte, it is cut at its rest shape and `MORPH_KNOCKOUT` warns.
- In Solid every part is one colour: a detail drawn inside a filled Solid body disappears unless it is knocked out (`SOLID_HIDDEN`), or the body's Solid is `"ring"`.
- A surface's default Solid is filled with a 2.6 rim, so it reads a little bigger than its outline. `"solid": "fill"` fills the outline's shape exactly.
- `"gap": {"other": d}`: cut this part wherever it comes within `d` of another part's shape at rest, in every style.

## Rigs

```json
"rigs": {
  "clock":  {"pivot": {"contact": ["leg-l.end", "leg-r.end"]}, "carries": ["body", "bell-l", "bell-r"]},
  "bell-l": {"pivot": [10, 11.5]}
}
```

- A rig named like a part moves that part. `carries` lists the parts and rigs that move with it: a rig in `carries` is a child rig, at any depth. A part carried by a rig is rigid with it.
- `pivot` is a constant point. `{"contact": [A, B]}` tips about A for negative angles and about B for positive ones (a clock rocking on its feet). It compiles to two rigs, one per sign, split exactly at the zero crossings.
- Any rotation needs a declared pivot (`PIVOT`). `x`, `y` and `scale` on a part without a rig use an implicit rig at the part's centre.
- `scale` and `squash` (`"squash": {"base": y, "widen": 0.5}`) reshape the carried parts' points, so strokes keep their weight. They are exact, and only for a rig that carries parts, not other rigs.

## Motion

```json
"motion": {
  "bell-l.rotate": {
    "light":  [[0, 0], [0.1, -6], [0.42, 0, "inOut"]],
    "strong": [[0, 0], [0.07, 5], [0.17, -20, "hit"], {"settle": 0.68, "to": 0}]
  },
  "bell-r.rotate": {"follow": "bell-l.rotate", "scale": -1}
}
```

A target is `<part or rig>.<prop>`, with props `x y rotate scale squash opacity draw r grow amp`, or a bare morph driver. Each track has `light` and `strong` keys; a motion that leaves a track out holds it at rest. Every icon has Light and Strong.

A key is `[t, value]` or `[t, value, ease]`. The ease says how the motion arrives at that key:

| Ease | Meaning |
|---|---|
| `auto` (default) | Smooth through same-direction keys, stops at turns and at the ends (v1 `flow`, exact) |
| `linear`, `in`, `out`, `inOut` (= `smooth`) | Polynomial curves, exact |
| `sine` | v1's cosine in-out |
| `hit` | Arrive at full speed and stop dead; a hold may follow (hit-then-hold) |
| `launch`, `pass` | Leave at full speed; keep speed through the key |
| `step` | Hold the previous value, jump at this key |
| `[x1, y1, x2, y2]` | A cubic-bezier, x in 0–1 |

Presets expand to keys. `hold`, `settle`, `bounce` and `wobble` are items in a key list; `pulse`, `loop` and `follow` replace the whole list (`"light": {"loop": …}`):

| Preset | Form |
|---|---|
| `{"hold": t}` | Stay at the value just reached until `t` |
| `{"settle": t, "to"?, "amp"?, "decay"?, "swings"?}` | Damped swings about `to` (default: the value just reached), ending at `t` |
| `{"bounce": t, "amp"?}` | A rebound of `amp` × the arriving move at the midpoint, back with `hit` |
| `{"wobble": at, "amp": a, "until": t, "swings"?}` | A damped shake from rest |
| `{"pulse": [start, duration], "r"?: d, "grow"?: d}` | Show, swell and fade a part (on `<part>.r`, `.grow` or `.opacity`); the part is hidden at rest |
| `{"loop": KEYS, "period": p, "until"?}` | Repeat keys that end where they start |
| `{"follow": "<target>", "lag"?, "scale"?, "offset"?}` | value = scale × source + offset, delayed by lag. `scale` is from -4 to 4 (default 1; -1 mirrors), `lag` from 0 to 1 s. As a whole track it applies to both motions. |

Timing is derived: the action ends at the motion's last key, and rest comes one rounded frame after `active + 0.04 s`. `"timing": {"light": s, "strong": s, "reverse": s, "reverseStrong": s}` overrides it (at most 2.3 s), but no key may come after it (`MOVES_AFTER_REST`). A gesture's rotation may end a whole turn (or a multiple of `"cycle": deg`, such as 60 on a hexagon) from where it started. The same pose rasterises a little differently there, and the build compares the rest frame with the first pixel for pixel, so the compiler adds an invisible step back to 0 one frame after the turn's last key. Every other track must end exactly on its rest value, in Light and Strong alike.

## State and reverse

```json
"state": {"from": "locked", "to": "open", "labels": ["Unlock", "Lock"], "reverse": "auto"},
"reverse": {"strong": {"door.rotate": [[0.04, 100], [0.26, 0, "hit"]]}}
```

- `auto` (default): each track's beats replay in mirrored order. Inside a beat, values reflect about its ends, and times and eases are kept. A wind-up stays a wind-up the other way. It reproduces v1's authored toggle-switch reverse key for key.
- `mirror`: an exact time reversal. A hold or step jumps up to one frame later.
- `fade`: the preview reset. States must be `prepared` → `result` and labels `["Play action", "Reset preview"]`; the animations are `<id>-reset` and `<id>-reset-strong`. Only the parts the action leaves changed fade out and back; a part that moved and came back stays.
- `reverse.light` / `reverse.strong` replace any track of a reverse with explicit keys. A track that follows an overridden one follows the override.
- Look at the reverse in the draft every time. `auto` replays a landing reaction (a `hit`, `bounce` or `settle` at the end state) at the start of the reverse and reflects it to the other side: a seat that dips before it moves, a flag that flutters below the pole. Override those tracks so the reverse has its own event (a lock pops open, a lid slams shut) and reacts where it lands. Most stateful icons with a landing need this.
- State names are kebab-case (`checkpoint-saved`). Labels are short verbs (`["Lock", "Unlock"]`). The animations are `<id>`, `<id>-strong` and `<id>-reverse`, `<id>-reverse-strong` (or the two `-reset` ones); `"reverseName": "<id>-unlock"` in `state` keeps a name apps already play (Strong's adds `-strong`).

## What check and draft tell you

| Code | Level | Meaning |
|---|---|---|
| `V2_SCHEMA` | error | Structure: types, ranges, unknown fields (with "did you mean"), JSON Pointer to the spot |
| `ONE_SUBPATH` | error | A `path` with a second M or an early Z; use a list of shapes |
| `UNKNOWN_NAME`, `RIG_CYCLE`, `CARRIED_TWICE`, `ANCHOR_CYCLE` | error | Names and hierarchy |
| `KEY_ORDER`, `TIMELINE`, `NO_MOTION`, `MOVES_AFTER_REST` | error | Times |
| `REST_VALUE` | error | A track that does not start (or, for a gesture, end) at rest; `scale` keyed as 0 is the classic one |
| `PIVOT`, `MORPH_TOPOLOGY`, `KNOCKOUT_RIGS`, `KNOCKOUT_TRACKS`, `COMBINED_PATH_TRACKS`, `PARAM_UNSUPPORTED`, `FADE_STATE` | error | Things the engine cannot build as written |
| `CANVAS_MARGIN` | error / warning | Paint past the canvas in any of 145 frames per motion / within 1.4 of the edge at Bold while at least 30% opaque, where `build`'s visual check would see it on the outer pixel at 24 px |
| `SMALL_AT_24`, `LIGHT_STRONG`, `TRAILING_HOLD`, `LIVE_ANCHOR` | warning | Under 3 px at 24 px (a part hidden at rest: under 2 px); Strong not stronger than Light; a last key that only stretches the action; an anchor resolved at rest |
| `SOLID_HIDDEN`, `HOLE_OUTSIDE`, `MORPH_KNOCKOUT` | warning | A part drawn inside another part's Solid fill, which vanishes in Solid; a hole or knockout that misses its surface at rest; a morphing knockout cut at its rest shape |
| `NOT_IMPLEMENTED` | error | A term of the format that cannot be built yet (below) |

With MCP, the `draft` tool returns the same sheets as images. `draft` writes `.gen-animated-icons/drafts/<id>.png` (rows: Light, then Strong in Regular, Bold, Solid and Duotone, then the reverses; columns: the beats of each motion) and `<id>-24px.png` (the real size, enlarged). The engine's renderer is not the player: `build` ends with dotlottie-rs sheets of every style and both themes in `.gen-animated-icons/previews/<id>/`. Look at both. Solid cut-outs made by a matte (a knockout that moves, draws on or fades) are where they differ: the draft can leave a drawn-on cut-out out entirely or show a matte beside a mask as a different shape. Judge Solid cut-outs on the build's sheets; `build <id> --no-install` renders them without installing.

## What build's visual check fails

`build` renders every action in dotlottie-rs at every size, style, weight and theme, frame by frame, and fails the icon on any of these. `check` cannot see most of them, so design against them from the start.

| Failure | Cause | Avoid it |
|---|---|---|
| Clipped icon | Ink on the outermost pixel, in any frame | Keep paint 1.4 inside the canvas at Bold, overshoot included (`CANVAS_MARGIN` warns) |
| Invisible main action | A frame with nothing drawn | Cross-fade a part that leaves and returns; never fade everything at once |
| Loop or state boundary is incorrect | A gesture's last frame differs from its first, or a toggle ends where it started | End every track exactly on rest; whole turns step back (above). Light and Strong end on the same pixels |
| Insufficient animation states | Too few distinct frames for the action's length (half its frames, 6 to 16) | Ease keys instead of `step`-only motion; a typing caret slides each stroke in 40–50 ms |
| Hold is not still | Something moves after rest | No key after the action ends |

## Not in the format yet

`swing`, `flip`, `minus`, a keyed wave `phase` and connectors (anchors that follow another rig) stop with `NOT_IMPLEMENTED`. Also not supported yet: `scale` or `squash` on a rig that carries rigs, two path-changing tracks on one shape, a negative Solid `grow`, and one pulse per track and motion. Parts that collapse are not hidden automatically: key their `opacity`. Clearance and counter-width lints are not run yet, so check counters at Bold in the draft. Continuous loaders (motion through the whole 2.4 s, trim start or offset) are not in the format yet. Several cut-outs moving on different rigs in one Solid surface cannot all follow (`KNOCKOUT_RIGS`): one surface has one matte. For an icon that needs one of these, write a JS batch: `create <id> --batch`.
