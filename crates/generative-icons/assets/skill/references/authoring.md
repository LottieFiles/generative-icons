# Author a local icon as a JS batch

An icon is a spec by default ([authoring-v2.md](authoring-v2.md)). For what a spec cannot express yet, `create <id> --batch` makes it JavaScript source in `generative-icons/icons/`, laid out like a library set (keep its files regular files: a symlinked source keeps the build `STALE_BUILD`, and `submit` refuses it):

| File | What it is |
| --- | --- |
| `batches/<id>.mjs` | One icon batch, written by `create`; yours to edit |
| `batches/_*.mjs` | Shared helpers, and library set sources that `create --from` copied; yours to edit |
| `batches/<folder>/*.mjs` | More helpers, if you want them in a subfolder; yours to edit |
| `definitions.mjs` | Imports every `batches/<id>.mjs`; `create` regenerates it until you edit it |
| `set-config.json` | The set's `id` and `name` (composition names use the name) |
| `evidence/design-review.md` | Your design notes |

`build` evaluates the definitions, samples each action at 60 fps, exports native Lottie on Creator MCP (LottieFiles' hosted engine), packages the `.lottie`, verifies it and installs it. Its intermediates live in `.generative-icons/build/<id>/`, its review sheets in `.generative-icons/previews/<id>/` (where `preview` also writes, unless you pass `--out`), with a snapshot of every source module it was built from (listed in `sources.snapshot.json`).

Import helpers by plain string specifiers (`import {nudge} from './shared/nudge.mjs'`). An `import()` of a computed name, or an escape such as `\x2f` in a specifier, hides which modules the icon depends on: its build then depends on every module the definitions load, other icons' batches included, so editing any of them makes it stale, and `submit` refuses it unless the user passes `--include-untraced`. `check` and `build` warn `UNTRACED_IMPORTS` before any export, naming the import as written, its file and line.

`check <id>` runs the same source preflight `build` runs first: every part is sampled at every frame, in both directions and at every outline weight. Then, on the scenes `build` would export, it runs the checks verification applies after the export: every visible part holds its final pose from rest to the end of each action, a gesture ends where it starts, Light and Strong start and end on the same pose, and a stateful icon's reverse starts where its action ends and ends where it starts. Its diagnostics name the icon, part, profile, weight and time, and a `path` (with a `line` when a callback threw) for the file at fault; fix each error before building. A load error (`SOURCE_ERROR`) starts with `file:line` for a syntax error, or names the file and the line at or before the throw. Once the icon is built, `check <id>` also verifies the build and warns `STALE_BUILD` when a file it depends on changed since: its batch, the helpers that batch imports, `definitions.mjs` (other than `create` adding a batch) or the set's config; the report's `status` is then `STALE`, and `stale` names the icons to rebuild. Editing one icon leaves the others current. A build made by an earlier version of the CLI tracked every source file, so any change makes it stale until it is rebuilt once.

Part names start with the icon id (`make` adds it). `constraints` entries (`{type: 'rigid', part}`) name whole part names, so rename them with the parts; `check` reports one that names no part (`UNKNOWN_CONSTRAINT_PART`) and lists the icon's real part names. A stub from `create --from` renames both, and starts a stateful icon's `initialState`, `finalState`, `forwardLabel` and `reverseLabel` at the kit's defaults: name them for the new icon, since apps use them (React `state`, `handle.setState`).

## The kit

Import from the managed kit copy; never edit it.

```js
import {make, P, F, stroke, fill, line, tube, framed, bothFill, rigid, posed, shown,
        rect, circle, arc, polygon, capsule, hole, holes, track, flow, clamp, mix}
  from '../../library/ai-authoring.mjs';

const bob = F([[0, 0], [.3, -2, 'out'], [.6, 0]],                   // Light keys: [seconds, value, ease?]
              [[0, 0], [.08, .6], [.28, -3.5, 'hit'], [.5, 0]]);    // Strong keys: [seconds, value, tag?]
export const icons = [
  make('bell-nudge', 'Bell nudge', 'The bell lifts once and settles.',
    [...framed('bell', rect(9, 8, 14, 13, 5)),
     ...line('clapper', [16, 23], [16, 25], {pose: rigid((t, s) => ({dy: bob(t, s)}))})],
    .6, .5)                                                          // active seconds: Light, Strong
];
```

- `make(id, label, intent, parts, lightSeconds, strongSeconds, rearm = [])` builds one icon. Part names are prefixed with the id. Passing `rearm` part names makes the icon stateful: those parts keep their result and a reset action rearms them.
- Parts: `stroke(name, d)` (outline centerline, width 2), `fill(name, d, style)`, `line(name, a, b)` and `tube(name, d)` (outline plus matching solid), `framed(name, d, solid)` (tinted interior, outline frame, solid fill), `bothFill(name, d)` (a filled mark in both Outline and Solid). Styles: `outline`, `solid`, `tint` (the Duotone surface).
- `d` is an SVG path string or a function `(t, s, r) => string`: `t` seconds, `s` true for Strong, `r` true for the reverse action. Keep the same command sequence at every time.
- Motion: `P(light, strong)` eases every interval to a stop; `F(light, strong)` does that for Light and carries velocity through Strong waypoints. Eased keys take `io` (default), `out`, `in`, `smooth` or `linear`; `F`'s Strong keys take `hit` (impact), `launch`, `pass` or `hold`.
- Rigid parts move with `pose: rigid((t, s, r) => ({dx, dy, angle, pivot}))`; `posed(parts, pose)` applies one pose to a group. `visibility: (t, s, r) => 0..1` fades a part; `shown(parts, fn)` applies it to a group.
- Shapes: `rect(x, y, w, h, r)`, `circle(x, y, r)`, `arc(cx, cy, r, a0, a1)`, `polygon(points, r)`, `capsule(a, b, width)`, `hole(outer, inner)` and `holes(outer, ...cuts)` for real counters.
- Declare `duotoneFill`: `true` when a `tint` part fills a real enclosed surface (`framed` draws one), `false` when no part is `tint`. The template declares `false`; `check` reports a mismatch as `DUOTONE_POLICY` and says which way to fix it.
- In reverse and reset actions, poses get `r = true`; a pose that ignores `r` replays the forward motion and fails `MOVES_AFTER_REST`. Preview a reset with `generative-icons preview <id> --motion reverse`; its columns span the reset's own timing.

## Family rules

- Draw on the 32-unit grid with round caps and joins. Round real corners unless the meaning needs a sharp one.
- Outline strokes stay at the family width; the player switches the 1.5 / 2 / 2.5 weights. Never scale a part to fake weight. Solid is its own filled drawing, not a heavy outline.
- Duotone tints only a real enclosed surface. Never invent a background shape to tint.
- Draw everything the motion exposes: rims, hinges, the hidden side of a flap. No loose ends, seams or background-colored patches.
- Write a motion brief first: cause, action, response, final state, for Light and for Strong.
- Light is one quiet, clear action. Strong changes the primary action visibly at normal speed and at 24 px: more travel, a wider turn or a more decisive stroke with preparation. A tiny follower, extra keyframes or a longer timeline is not Strong.
- Rigid parts stay rigid; progress never overshoots; secondary motion needs a cause.
- Pick a state policy per icon: a gesture returns to rest; a state or output icon keeps its result (use `rearm`); a loader or live voice indicator is a continuous cycle with a readable still pose.
- Reuse ideas from `generative-icons context`, never the reference art itself.
