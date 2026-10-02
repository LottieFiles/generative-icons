# Author an icon as a spec (v2, prototype)

A v2 icon is one JSON file, `gen-animated-icons/icons/<id>.icon.json`, with `"format": "gai-icon/2"`. It is data: shapes, roles, rigs and eased keys. Nothing in it is a function of time, so there is no pose math to write and no SVG to hand-build. The hosted engine builds every motion from it in one call: native shapes, rigs chained by layer parenting, eased keys, masks, mattes, trims and markers. v1 icons (`batches/*.mjs`) keep working; a command takes the v2 path only when every icon it names has a spec.

```sh
gen-animated-icons create <id> --spec      # a starter spec that checks clean
gen-animated-icons check <id>              # offline: structure, names, motion, lints (milliseconds)
gen-animated-icons draft <id>              # one engine call: a review sheet, nothing stored
gen-animated-icons build <id>              # one engine call: export, slots, themes, .lottie, verify, install
```

Fix every `check` error before you draft. A draft is about 4 s and stores nothing, so draft after each change you cannot judge from the numbers; build once the draft looks right. `check` and `draft` never need a build first.

## Conventions

| Item | Rule |
|---|---|
| Grid | 32 units, y down. A point is `[x, y]`, an anchor name, or `{"from": P, "angle": deg, "len": r}`. |
| Angles | Degrees, positive = clockwise on screen. Polar and arc angles: 0 = +x (3 o'clock), -90 = up. |
| Times | Seconds on the 2.4 s timeline. Fractional times are kept. |
| Values | `x`, `y`, `rotate`, `r`, `grow`, `amp` are offsets from the drawn pose (rest 0). `opacity` and `draw` are absolute 0–1 (rest 1). `scale` and `squash` are factors (rest 1). |
| Draw order | The order of `parts`, back to front. |
| Comments | A `"//"` key on the spec, a part, a rig, a track (a `follow` included) or `reverse`. |

The drawing is the rest pose. A gesture ends where it starts; a stateful icon (`state`) ends changed.

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

`mirror` copies another part across x = 16 (or `"x"`/`"y"`), with its options unless you set them. Anchors: `"anchors": {"name": [x, y]}`, plus derived ones on every part: `<part>.c`, `.top`, `.bottom`, `.left`, `.right`, `.tl`, `.tr`, `.bl`, `.br` (bounding box at rest) and `.start`, `.end` (open shapes). Polar points replace hand trigonometry: a hand at 2 o'clock is `{"from": "dial", "angle": -30, "len": 4}`.

### Roles: one drawing, every style

Each part expands into the style layers the runtime switches: outline at 1.5 / 2 / 2.5, Solid, and the Duotone tint. Never draw a style twice.

| Role | Outline | Tint | Solid (`auto`) |
|---|---|---|---|
| `stroke` (default) | stroked | none | the centerline at 2.6 |
| `surface` | the frame stroked; `"outline": "open"` leaves out the closing edge, `"none"` draws none | the interior (`"tint": false` for none) | filled, plus a 2.6 rim |
| `fill` | filled in every weight | none | filled |

`"solid"` also takes `"ring"` (the outline at 2.6, hollow), `"fill"`, `"none"`, `{"grow": d}` (fill plus a 2d rim; d > 0) or a shape of its own.

### Holes, knockouts and gaps

- `"holes": [S, …]`: real counters. They are stroked in Outline and cut from tint and Solid by subtract masks, so overlapping counters stay open. On a part that `scale` or `squash` reshapes, the masks follow.
- `"knockout": ["hand", {"part": "knob", "gap": 0.8}]` on a surface: in Solid, those parts are cut out of it with a clearance (default 0.6, or the part's `"gap": number`) instead of drawn. A part that sits still on the surface's rig is cut by masks. One that moves on another rig, is reshaped, draws on, fades or starts hidden is cut by a matte that follows it, trim included, so the cut-out draws on with the part. A surface's moving knockouts must share one rig (`KNOCKOUT_RIGS`), and a part with its own trim or visibility must be its surface's only moving knockout (`KNOCKOUT_TRACKS`).
- In Solid every part is one colour: a detail drawn inside a filled Solid body disappears unless it is knocked out (`SOLID_HIDDEN`), or the body's Solid is `"ring"`.
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

Presets expand to keys:

| Preset | Form |
|---|---|
| `{"hold": t}` | Stay at the value just reached until `t` |
| `{"settle": t, "to"?, "amp"?, "decay"?, "swings"?}` | Damped swings about `to` (default: the value just reached), ending at `t` |
| `{"bounce": t, "amp"?}` | A rebound of `amp` × the arriving move at the midpoint, back with `hit` |
| `{"wobble": at, "amp": a, "until": t, "swings"?}` | A damped shake from rest |
| `{"pulse": [start, duration], "r"?: d, "grow"?: d}` | Show, swell and fade a part (on `<part>.r`, `.grow` or `.opacity`); the part is hidden at rest |
| `{"loop": KEYS, "period": p, "until"?}` | Repeat keys that end where they start |
| `{"follow": "<target>", "lag"?, "scale"?, "offset"?}` | value = scale × source + offset, delayed by lag. `scale` is from -4 to 4 (default 1; -1 mirrors), `lag` from 0 to 1 s. As a whole track it applies to both motions. |

Timing is derived: the action ends at the motion's last key, and rest comes one rounded frame after `active + 0.04 s`. `"timing": {"light": s, "strong": s, "reverse": s, "reverseStrong": s}` overrides it (at most 2.3 s), but no key may come after it (`MOVES_AFTER_REST`). A rotation may end a whole turn (or a multiple of `"cycle": deg`) from where it started.

## State and reverse

```json
"state": {"from": "locked", "to": "open", "labels": ["Unlock", "Lock"], "reverse": "auto"},
"reverse": {"strong": {"door.rotate": [[0.04, 100], [0.26, 0, "hit"]]}}
```

- `auto` (default): each track's beats replay in mirrored order. Inside a beat, values reflect about its ends, and times and eases are kept. A wind-up stays a wind-up the other way. It reproduces v1's authored toggle-switch reverse key for key.
- `mirror`: an exact time reversal. A hold or step jumps up to one frame later.
- `fade`: the preview reset. States must be `prepared` → `result` and labels `["Play action", "Reset preview"]`; the animations are `<id>-reset` and `<id>-reset-strong`.
- `reverse.light` / `reverse.strong` replace any track of a reverse with explicit keys: use them where a reflection would pass through another part, or a reaction must not replay.

## What check and draft tell you

| Code | Level | Meaning |
|---|---|---|
| `V2_SCHEMA` | error | Structure: types, ranges, unknown fields (with "did you mean"), JSON Pointer to the spot |
| `ONE_SUBPATH` | error | A `path` with a second M or an early Z; use a list of shapes |
| `UNKNOWN_NAME`, `RIG_CYCLE`, `CARRIED_TWICE`, `ANCHOR_CYCLE` | error | Names and hierarchy |
| `KEY_ORDER`, `TIMELINE`, `NO_MOTION`, `MOVES_AFTER_REST` | error | Times |
| `REST_VALUE` | error | A track that does not start (or, for a gesture, end) at rest; `scale` keyed as 0 is the classic one |
| `PIVOT`, `MORPH_TOPOLOGY`, `KNOCKOUT_RIGS`, `KNOCKOUT_TRACKS`, `COMBINED_PATH_TRACKS`, `PARAM_UNSUPPORTED`, `FADE_STATE` | error | Things the engine cannot build as written |
| `CANVAS_MARGIN` | error / warning | Paint past the canvas in any of 145 frames per motion / within 0.5 of the edge |
| `SMALL_AT_24`, `LIGHT_STRONG`, `TRAILING_HOLD`, `LIVE_ANCHOR` | warning | Too small at 24 px; Strong not stronger than Light; a last key that only stretches the action; an anchor resolved at rest |
| `SOLID_HIDDEN`, `HOLE_OUTSIDE` | warning | A part drawn inside another part's Solid fill, which vanishes in Solid; a hole or knockout that misses its surface at rest |
| `NOT_IMPLEMENTED` | error | A term of the format this prototype cannot build yet (below) |

With MCP, the `draft` tool returns the same sheets as images. `draft` writes `.gen-animated-icons/drafts/<id>.png` (rows: Light, then Strong in Regular, Bold, Solid and Duotone, then the reverses; columns: the beats of each motion) and `<id>-24px.png` (the real size, enlarged). The engine's renderer is not the player: `build` ends with dotlottie-rs sheets of every style and both themes in `.gen-animated-icons/previews/<id>/`. Look at both. Solid cut-outs made by a matte (a knockout that moves, draws on or fades) are where they differ: the draft can leave a drawn-on cut-out out entirely or show a matte beside a mask as a different shape. Judge Solid cut-outs on the build's sheets; `build <id> --no-install` renders them without installing.

## Not in this prototype

`swing`, `flip`, `minus`, a keyed wave `phase` and connectors (anchors that follow another rig) stop with `NOT_IMPLEMENTED`. Also not supported yet: `scale` or `squash` on a rig that carries rigs, two path-changing tracks on one shape, a negative Solid `grow`, and one pulse per track and motion. Parts that collapse are not hidden automatically: key their `opacity`. Clearance and counter-width lints are not run yet, so check counters at Bold in the draft. Continuous loaders (motion through the whole 2.4 s, trim start or offset) are not in the format yet: keep them in v1.
