---
name: generative-icons
description: Add animated UI icons to a web, iOS or Android app with Generative Icons by LottieFiles and the generative-icons CLI. Search a library of over 1,000 icons, install and render one, or create a missing icon that matches the family. Use when the user wants an animated icon in an app (a like heart, a ringing bell, a menu that turns into a close button, a loader), asks for one that may not exist yet, or changes files this CLI manages. Not for illustrations, static icon packs or general animation work.
---

# Animated icons with generative-icons

Generative Icons are native Lottie icons on a 32-unit grid. Each one has two authored motions, Light and Strong, and comes in Outline (three weights), Solid and Duotone, recolored at runtime. Stateful icons, such as a toggle, also have a reverse action. The `generative-icons` CLI finds and installs them, and creates a missing one in the app's own project.

## Before you start

- Run the CLI as `npx generative-icons <command>`, or through the project's package manager (`pnpm exec`, `yarn`), unless `generative-icons` is on the `PATH`. It runs on macOS and Linux (arm64, x64), not on Windows yet. The examples below leave out `npx`.
- Run commands in the app folder, or pass `--cwd <app-folder>` (monorepos). `generative-icons <command> --help` lists a command's options.
- Start with `generative-icons info`. It reports the platform, config, library version, managed files, local icons and the sign-in. `local.icons` gives each icon made here its `spec` (or JS `batch`), its `build` (`current`, `stale` or `not-built`) and whether it is `installed`, so you can pick up work in progress.
- If there is no `generative-icons.json`, run `generative-icons init`. It writes the config and the authoring kit, and installs this skill into the project at the CLI's version. If this skill was installed another way (such as `npx skills add`) and `init` stops with `MODIFIED_FILE` or `SYMLINK_PATH` on a skill file, run `init --agents none` instead: it sets up the project and leaves the skill alone.
- Output is JSON when piped; pass `--json` to be sure. On failure, read `error.code`, `error.message`, `error.path` and `error.hint` on stderr and act on the hint. Exit codes: 1 a check failed (the report is on stdout) or the icon source failed to load (`SOURCE_ERROR`), 2 usage, 3 project or file conflict, 4 sign-in, 5 network or engine, 6 integrity.

## 1. Use a library icon when one fits

```sh
generative-icons search "<meaning>"      # ranked matches: id, label, category, stateful
generative-icons preview <id>            # PNG contact sheets of its motion, if you are unsure
generative-icons add <id> [<id>...]      # installs the .lottie asset and its descriptor
```

1. Search the meaning, not the look. Common synonyms are built in ("delete" finds `trash`, "envelope" finds `mail`). If nothing fits, try other words and the underlying object (`bell` for "alerts").
2. Results come best first. Pick by label and category, and copy ids exactly; never invent an id or guess a file name.
3. `add` the icon and render it (below). Tell the user which id you chose and why.

`generative-icons remove <id>` uninstalls an icon; it refuses files someone edited. Commit `generative-icons.json`, the installed assets and descriptors, and `.generative-icons/lock.json`.

## 2. Render it

**Web.** Install the runtime, plus the React adapter for React or Next.js, with the project's package manager:

```sh
npm install @lottiefiles/generative-icons-runtime @lottiefiles/generative-icons-react
```

Import the descriptor and its `src` from the module `add` wrote (`src/generative-icons/<id>.mjs` in an app with a `src` folder) and pass them together:

```jsx
'use client';
import { AnimatedIcon } from '@lottiefiles/generative-icons-react';
import icon, { src } from './generative-icons/heart.mjs';

export const Favorite = () => <AnimatedIcon icon={icon} src={src} size={24} color="#008c76" />;
```

```js
import { createIcon } from '@lottiefiles/generative-icons-runtime';
import icon, { src } from './generative-icons/heart.mjs';

const handle = await createIcon(canvas, { icon, src, color: '#008c76' });
button.addEventListener('click', () => handle.play()); // call handle.destroy() when the element is removed
```

**iOS and Android.** There is no Generative Icons package for native apps, and none is needed. `add` puts the `.lottie` file in the app's assets, and LottieFiles' dotLottie players play it: `dotlottie-ios` in SwiftUI, `dotlottie-android` in Compose or Views. Light and Strong are two animations inside the file (`<id>` and `<id>-strong`). Color, style and outline weight are slots, set when the player loads.

**No player** (a static page, Markdown, email or a design tool): `generative-icons export <id> --out <file>.svg` writes an animated SVG that browsers play with SMIL, without JavaScript. Options are `--motion`, `--style`, `--weight`, `--color` (default `currentColor`, so CSS `color` themes it) and `--trigger` (default `toggle` for stateful icons, `loop` for loaders, `hover` otherwise). `hover`, `click`, `toggle` and `js` need the SVG inline; an `<img>` plays only `loop` and `load`. Give each inline copy on a page its own `--id-prefix`. In apps, prefer the player: it follows reduced motion and app state, and an SVG can't.

Setup, state, paint and code for every platform: [runtime.md](references/runtime.md).

- Colors are six-digit hex; resolve theme tokens first. Styles are `outline`, `solid` or `duotone` (the React prop is `variant`). Weight is 1 to 3, outline only; iOS and Android players show 1.5, 2 or 2.5 unless `add --bake-weight <w>` bakes another.
- Stateful icons follow app state through the React `state` prop or `handle.setState(name)`; `play('reverse')` runs the authored reverse.
- Icons are decorative by default: the control around them carries the accessible name. The web runtime follows reduced motion; on iOS and Android, check the setting before you play.

## 3. Create an icon only when nothing fits

Ask the user before you create one. Tell them what you searched for, which icons came closest, and that a new icon:

- takes several rounds of drafting, building and reviewing;
- needs their LottieFiles sign-in, because `draft` and `build` export on Creator MCP, LottieFiles' hosted engine. They send it the icon's spec (its shapes and motion), never the app's code;
- stays in this project unless they choose to share it.

If they would rather use the closest library icon, use it.

### Sign-in

`info` reports `auth.session`: `verified`, `not-verified` (offline, or the engine could not be reached; `auth.reason` says why), `ended` or `signed-out`. On `ended` or `signed-out`, or when a command fails with `NOT_LOGGED_IN` or `SESSION_EXPIRED`, ask the user to run `generative-icons login` in their terminal (`--no-browser` over SSH) and wait. Never sign in yourself. On `SIGN_IN_BUSY`, another generative-icons process holds the sign-in, often while a macOS Keychain prompt waits for the user: ask them to approve it or end that process, then retry. Everything before `draft` works without a sign-in.

### Read the direction, then brief

An icon you make must match the library's drawing and motion. Before you brief, read the rules every library icon was made and reviewed by:

- [motion-principles.md](references/motion-principles.md): events, not nudges; Light and Strong; material and mechanics; state and looping.
- [design-principles.md](references/design-principles.md): construction, styles, geometry that survives motion, optical review.
- [craft.md](references/craft.md): drawing the complete object and directing the motion before keyframing.
- [polish-review.md](references/polish-review.md): what to decide before drawing, construction that fails in motion, and Strong acceptance.
- [motion-system.md](references/motion-system.md): the motion system's roles and decision procedure.

Then brief each icon before you write its spec:

- its meaning, and the event as **cause → action → response → final state**;
- the state policy: a gesture that ends at rest, a state with a reverse that keeps its result, or a continuous indicator;
- the driver, its followers and their lag, and any cue hidden at rest (sound, sparks, a trail) or mark that draws on;
- Light (the event once, one beat) and Strong (what changes in the primary action), with travel in units, turns in degrees and times in seconds.

A sway, wobble, bob, pulse, or a tilt or turn that comes back is a nudge, not an event; `check` warns `NUDGE` on one. Draw the subject once and transform it. Never swap it for a copy that fades in where it was (`check` warns `DUPLICATE_DRAWING`); parts hidden at rest are cues, never the subject again.

### Author, check and build

```sh
generative-icons context "<meaning and motion>"    # family contract, nearest references, their geometry and timing
generative-icons create <id> --label "<Label>" --intent "<what the motion shows>" [--from <library-id>] [--neighbor <id>]
# edit the spec it writes: generative-icons/icons/<id>.icon.json
generative-icons check <id>              # offline, in milliseconds: structure, names, motion and lints
generative-icons draft <id>              # one engine call: a review sheet of every motion, nothing stored
generative-icons build <id>              # one engine call: export, package, verify, preview sheets, install
generative-icons preview <id>            # re-render the sheets; open every PNG it lists
```

An icon is a spec: plain JSON shapes, roles, rigs and eased keys, with no SVG paths and no per-frame pose math. Write it as [authoring-v2.md](references/authoring-v2.md) says, and read its "Motion that reads" and "What build's visual check fails" sections before you design the motion.

- **Start from a library spec when one is close** (the same object, or the same kind of event): `create <id> --from <library-id>` copies its parts, rigs, motion, state and reverse under your id. Over 400 library icons are specs, and `search` marks them `"spec": true`. Rewrite the intent, redraw what the new meaning needs, and rename a stateful reference's state names and labels. Without `--from`, replace the starter bell's parts and keys.
- `context` picks the two nearest library icons as references; `--neighbor <id>` (up to four) chooses them, and `--format text` prints a summary without the geometry. Read the references for construction and timing, never to copy art, and pass `create` the same `--neighbor` ids.
- A new id is kebab-case and not a library id; `create` refuses anything else with a hint.
- Fix every error `check` lists before you draft. Each finding has a JSON Pointer into the spec and a hint, and `draft` and `build` stop before the engine while the spec has errors. Draft after each change you cannot judge from the numbers, and build once the draft looks right.
- The draft uses the engine's renderer; `build`'s sheets use the player's, so judge Solid cut-outs on those. `build` installs only icons whose checks pass; if one fails, the ones built before it are still installed and the error names them.
- `preview` options: `--motion light|strong|both|reverse|all` (`reverse` is a stateful icon's reset actions), `--animation <name>` for the action a verify failure names, `--times 0,0.1,0.2` or `--frames N` for up to 64 columns, and `--styles`, `--weights` and `--themes` for rows.

Continuous loaders can't be specs yet: `create <id> --batch` writes a JS batch, drawn and animated in code. Read [authoring.md](references/authoring.md) before you write one; `draft` takes specs only.

### Review until it reads

1. Open every sheet `draft` and `preview` write, and look at it: each style and weight, both themes, and Light against Strong at the same elapsed times. Is it the event, with the driver first and followers in order, and is Strong stronger in the primary action? What to look for: [review.md](references/review.md).
2. Name each flaw you see, fix it in the spec, then rerun `check` and `draft`, and `build` and `preview` once the draft looks right.
3. Stop when a round finds nothing new, or when the same flaw repeats twice. Then report what remains.

### Sharing is the user's call

To share a finished icon, the user runs `generative-icons submit <id> --license MIT --wait` (`--dry-run --output bundle.zip` writes the bundle without sending it). Only verified builds of the current spec are accepted. The community at generativeicons.com builds each icon again from its spec, checks it, and lists what passes as unreviewed until a maintainer reviews it; `--wait` prints each icon's page, or why it was not published. Up to 5 icons go at a time, within a daily limit shared with generating icons on the website. Batch icons can only be bundled with `--dry-run`.

## When the library or network is unavailable

Commands use the hosted library unless a library checkout or an installed `@lottiefiles/generative-icons` package sits above the app. When the hosted one is unreachable (`NETWORK`; its message gives the cause), pass `--library <dir>` with a package `dist` folder or a library checkout, or `--offline` to use what is cached. Once you pass `--library`, pass the same one to every later command, or set `GEN_ANIMATED_ICONS_LIBRARY` for the session: `check`, `build` and `submit` compare new ids with that library. `--offline` never uses the network, so `login`, `logout`, `draft` and `build` fail with `NETWORK`. `generative-icons doctor` checks the whole setup, including whether the library's catalog can be read.

## Agents with MCP

`generative-icons mcp` serves the same flow as tools: `search`, `add`, `context`, `create`, `check`, `draft` and `preview` (both return the sheets as images), `build` and `export`. `submit` and `login` stay in the terminal. Arguments must match each tool's input schema exactly: `build`, `check` and `add` take `ids` (a list), `preview` and `create` take `id`, and a tool's `cwd` must be a folder inside the server's. Build one icon per `build` call. Add the server to the agent's MCP settings only when the user asks: `{"mcpServers": {"generative-icons": {"command": "npx", "args": ["generative-icons", "mcp"]}}}`. For a binary not installed from npm, `init` prints `mcpLocal`, an entry that runs it by its absolute path.

## Rules

- Never hand-edit generated files: installed `.lottie` assets, descriptors, `settings.*`, `.generative-icons/`, this skill, or the kit in `generative-icons/library/`. Change `generative-icons.json` or the icon's source, then rerun the command.
- `MODIFIED_FILE` means someone edited a managed file. Run `generative-icons diff`, show the user, and let them decide. Never delete or overwrite a file to get past it. An edited kit copy is fixed with `generative-icons init --update-kit`.
- Light and Strong are separate authored motions. Never fake Strong by changing speed.
- Animate what the object does for the user (a microphone takes sound, send leaves, a bell rings), in beats that get smaller, with the parts in order. The subject is drawn once: the motion moves, turns, scales or draws it and brings it back.
- Report every check you did not run. Never claim you reviewed a preview you did not open.
- Do not edit `AGENTS.md`, `CLAUDE.md`, MCP settings or global configuration unless the user asks.
