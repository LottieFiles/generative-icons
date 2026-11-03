---
name: generative-icons
description: Find, install and create animated icons (Generative Icons by LottieFiles) in a web, iOS or Android app with the generative-icons CLI. Use when the user wants an animated icon, asks for one that may not exist yet, or changes icon files this tool manages. Not for general illustration or unrelated animation.
---

# Animated icons with generative-icons

Run commands in the app folder, or add `--cwd <app-folder>` (monorepos). `generative-icons <command> --help` lists a command's options. Output is JSON when piped; pass `--json` to be sure. On failure, read `error.code`, `error.message`, `error.path` and `error.hint` from stderr and act on the hint. Exit codes: 1 a check failed (the report is on stdout) or the icon source failed to load (`SOURCE_ERROR` on stderr), 2 usage, 3 project or file conflict, 4 sign-in, 5 network or engine, 6 integrity.

Start with `generative-icons info`: it reports the platform, config, library version, managed files, every local icon and the sign-in. `localIcons` includes icons you created but have not built, and `local.icons` gives each one's `spec` (or JS `batch`), its `build` (`current`, `stale` or `not-built`) and whether it is `installed`, so you can pick up work in progress. `auth.session` is `verified`, `not-verified` (offline, or the engine could not be reached; `auth.reason` says why), `ended` or `signed-out`; on `ended` or `signed-out`, ask the user to sign in before you build. `--offline` never uses the network: `info` and `whoami` report a stored session as `not-verified` without checking it, and `login`, `logout` and `build` fail with `NETWORK`. If there is no `generative-icons.json`, run `generative-icons init`. `generative-icons doctor` checks the whole setup, including whether the library's catalog can be read. Commands use the hosted library unless a checkout or an installed `@lottiefiles/generative-icons` package sits above the app; when the hosted one is unreachable (`NETWORK`, whose message gives the cause), pass `--library <dir>` with a package `dist` folder or a library checkout, or `--offline` to use what is cached. Once you pass `--library`, pass the same one to every later command (or set `GEN_ANIMATED_ICONS_LIBRARY` for the session): `check`, `build` and `submit` compare new ids with that library, and without it they warn that ids were not checked.

## 1. Use an existing icon when one fits

```sh
generative-icons search "<meaning>"      # ranked matches: id, label, category, stateful
generative-icons add <id> [<id>...]      # installs the .lottie asset and its descriptor
generative-icons preview <id>            # look at it first if unsure (PNG sheets)
```

1. Search the meaning. `search` adds common synonyms itself ("delete" finds `trash`, "envelope" finds `mail`) and leaves out icons that only mention a word in their description when others are named for it. If nothing fits, search other words and the underlying object (`bell` for "alerts").
2. Pick by label and category. Results come best first: an icon the query names leads, and `score` follows that order. Copy ids exactly; never invent one or guess a file name.
3. `add` it, then wire it up as [runtime.md](references/runtime.md) shows for the platform.
4. Tell the user which id you chose and why.

`generative-icons remove <id>` uninstalls an icon; it refuses files someone edited.

Where the player can't run or isn't wanted (a static page, Markdown, email or a design tool), `generative-icons export <id> --out <file>.svg` writes an animated SVG that browsers play with SMIL, without JavaScript. Options are `--motion`, `--style`, `--weight` and `--color`, which defaults to `currentColor` so CSS `color` themes it. `--trigger` defaults to `toggle` for stateful icons, `loop` for loaders and `hover` otherwise. `hover`, `click`, `toggle` and `js` need the SVG inline; an `<img>` plays only `loop` and `load`. Give each inline copy on a page its own `--id-prefix`. In apps, prefer the player: it follows reduced motion and app state, and an SVG can't.

## 2. Create an icon when nothing fits

An icon you make must match the library's drawing and motion quality. Before you brief, draw or animate, read the family's direction, the rules every library icon was made and reviewed by:

- [motion-principles.md](references/motion-principles.md): events, not nudges; Light and Strong; material and mechanics; state and looping.
- [design-principles.md](references/design-principles.md): construction, styles, geometry that survives motion, optical review.
- [craft.md](references/craft.md): drawing the complete object and directing the motion before keyframing.
- [polish-review.md](references/polish-review.md): what to decide before drawing, construction that fails in motion, and Strong acceptance.
- [motion-system.md](references/motion-system.md): the motion system's principles, roles and decision procedure.

Then brief each icon before you write its spec:

- its meaning, and the event as **cause → action → response → final state**;
- the state policy: a gesture that ends at rest, a state with a reverse that keeps its result, or a continuous indicator;
- the driver, its followers and their lag, and any cue hidden at rest (sound, sparks, a trail) or mark that draws on;
- Light (the event once, one beat) and Strong (what changes in the primary action), with travel in units, turns in degrees and times in seconds.

A sway, wobble, bob, pulse, or a tilt or turn that comes back is a nudge, not an event; `check` warns `NUDGE` on one. Draw the subject once and transform it: never swap it for a copy that fades in where it was (`check` warns `DUPLICATE_DRAWING`); parts hidden at rest are cues, never the subject again.

```sh
generative-icons context "<meaning and motion>"    # contract, nearest references, their geometry and timing
generative-icons create <id> --label "<Label>" --intent "<what the motion shows>" [--from <library-id>] [--neighbor <id>]
# edit the spec it writes: generative-icons/icons/<id>.icon.json
generative-icons check <id>              # offline, in milliseconds: structure, names, motion and lints
generative-icons draft <id>              # one engine call: a review sheet of every motion, nothing stored
generative-icons build <id>              # one engine call: export, package, verify, preview sheets, install
generative-icons preview <id>            # re-render the sheets; open every PNG it lists
```

An icon is a spec: plain JSON shapes, roles, rigs and eased keys, with no SVG paths and no per-frame pose math. Write it as [authoring-v2.md](references/authoring-v2.md) says. Its "Motion that reads" and "What build's visual check fails" sections are what reanimating over 400 library icons taught: read them before you design the motion.

- `context` prints JSON when piped; `--format text` prints a summary (the candidates, and each reference's intent and timing) without the geometry. It picks the two nearest library icons as references; pass `--neighbor <id>` (up to four) to choose them. Read their profiles (timing) and `initialOutlineGeometry` for construction, never to copy art. `create` saves a packet for the new icon in `.generative-icons/context/<id>.json`; pass it the same `--neighbor` ids you chose with `context`. Its output lists the packet's `references`, and `nearest` (the closest library icons) only when you chose none.
- Start from a library icon's spec when one is close (the same object, or the same kind of event): `create <id> --from <library-id>` copies its parts, rigs, motion, state and reverse under your id, with the craft that passed the library's checks. Over 400 library icons are specs, and `search` marks them `"spec": true`; every library carries them (a checkout, the installed package or the hosted library), verified against the package's `release.json`. Without `--from`, replace the starter's parts and keys (a bell that checks clean). Rewrite the intent, redraw what the new meaning needs, and rename a stateful reference's state names and labels.
- A new id is kebab-case, not a library id, and not a Windows device name (`con`, `prn`, `aux`, `nul`, `com1` to `com9`, `lpt1` to `lpt9`): `create` refuses those with `USAGE`, since `submit` could never bundle their files.
- Fix every error `check` lists before you draft; each finding has a JSON Pointer into the spec and a hint. Draft after each change you cannot judge from the numbers, and build once the draft looks right: `draft` and `build` stop before the engine while the spec has errors. The draft is the engine's renderer; `build`'s sheets are the player's, so judge Solid cut-outs on those.
- `draft` and `build` run on Creator MCP, LottieFiles' hosted engine, so they need the user's sign-in and the network: with `--offline` it exports nothing and fails with `NETWORK`. On `NOT_LOGGED_IN` or `SESSION_EXPIRED`, ask the user to run `generative-icons login` (`--no-browser` over SSH) and wait; never try to sign in yourself. On `SIGN_IN_BUSY`, another generative-icons process holds the sign-in, often while a macOS Keychain prompt waits for the user: ask the user to approve it or end that process, then retry. `build` prints the preview paths and installs only icons whose checks pass; when an icon cannot be built, the ones built before it are still installed and the error names them.
- `preview` options: `--motion light|strong|both|reverse|all` (`reverse` is a stateful icon's reset actions), or `--animation <name>` for the action a verify failure names; `--times 0,0.1,0.2` or `--frames N` (evenly spaced from the start to the rest frame) for up to 64 columns; `--styles`, `--weights`, `--themes` for rows. Light and Strong share columns, and reverse actions get their own over their own timing. Previewing an icon you created but have not built fails with `UNKNOWN_ICON` and a hint to build it.

Iterate until the icon reads well:

1. Open every sheet `draft` and `preview` write, and look at it. Check each style and weight, both themes, and Light against Strong in the same columns (the same elapsed times). Judge the motion against your brief and the direction: is it the event, with the driver first and followers in order, and is Strong stronger in the primary action ([polish-review.md](references/polish-review.md))?
2. Name each flaw you see, fix it in the spec, then rerun `check` and `draft`, and `build` and `preview` once the draft looks right.
3. Stop when a round finds nothing new, or when the same flaw repeats twice; then report what remains.

What to look for in previews: [review.md](references/review.md).

### JS batches (`--batch`)

Specs cannot express everything yet: continuous loaders, above all (authoring-v2.md, "Not in the format yet"). For those, `create <id> --batch` writes a v1 JS batch, `generative-icons/icons/batches/<id>.mjs`, drawn and animated in code as [authoring.md](references/authoring.md) says. `check`, `build` and `preview` work as above; `draft` takes specs only.

- `create <id> --batch --from <library-id>` starts the batch from that icon's working source (drawing and motion) to adapt; its parts and constraints already carry the new id. A stateful reference's state names and action labels (`initialState`, `finalState`, `forwardLabel`, `reverseLabel`) start as the kit's defaults (`prepared`, `result`) in the stub: name them for the new icon, since they ship in its descriptor. It needs a library checkout (`--library <checkout>` or `--library-root <checkout>`); the hosted library has no sources, so without a checkout leave `--from` out. Only icons of sets 07 onward can be references: `search` lists each icon's `set`, and `UNSUPPORTED_REFERENCE` suggests eligible ones in its hint. An icon reanimated as a spec is refused too, because its set source is the motion the spec replaced: start from its spec with `create <id> --from <library-id>`. Without `--from` the batch is a commented template, and `check` reports `ICON_NOT_EXPORTED` until you draw the icon with `make('<id>', …)`.
- Fix every error `check` lists before building; `build` runs the same source checks and exports nothing while one fails. Besides the drawing rules, they include what verification would reject after the export: a part still moving after rest (`MOVES_AFTER_REST`), a gesture ending away from its first pose (`RESET_MOVED`), Light and Strong starting or ending apart (`LIGHT_STRONG_DIFFER`) and a reverse that does not undo its action (`REVERSE_MISMATCH`). A finding with a `path` names the file to edit, and a `line` where the code threw (QuickJS gives the line its statement starts on, at or before the fault). A `SOURCE_ERROR` message starts with `file:line` for a syntax error, or names the file and the line at or before the throw. Icon code that runs too long stops with `SOURCE_ERROR` too, naming the file and the line it was running where it can.
- `check` never waits on the network: library ids are compared with `--library` or the cached catalog, and `build` checks them online. Before the first build the native check is skipped (`<id> is not built yet`); after a build it verifies it, and `STALE_BUILD` means a file that icon depends on (its batch, the helpers it imports, `definitions.mjs` or the set's config) changed since, so rebuild it. Then the report's `status` is `STALE` (exit 0), the native check's too, and `stale` lists the icons to build again before you preview or rely on them; a file it depends on that became a symlink, or is no longer a regular file, counts as changed too. Other icons stay current, and `submit` accepts only current builds. A build made by an earlier version of the CLI tracked every source file, so any change makes it stale until it is rebuilt once; its message says so.
- Helpers may sit in subfolders of `generative-icons/icons/batches/`; a build snapshots every module the source loads. Import each one by a plain string such as `'./shared/h.mjs'`: an `import()` of a computed name or an escape in a specifier hides which modules the icon depends on, so its build depends on every module and `submit` refuses it. `check` and `build` warn `UNTRACED_IMPORTS` before the export, naming the import, its file and line; fix it then. Keep source files regular files, not symlinks. Never add files to the kit folder `generative-icons/library/`.

To share a finished icon, the user runs `generative-icons submit <id> --license MIT --wait` (`--dry-run --output bundle.zip` writes the bundle without sending). Only verified builds of the current source are accepted. The community at generativeicons.com builds a spec icon again from its spec, checks it as it checks the icons it generates, and publishes what passes marked unreviewed until a maintainer reviews it; `--wait` prints each icon's page, or why it was not published. A batch icon can't be sent (`USAGE`), only bundled with `--dry-run`. A batch icon's bundle holds what each build verified, for the submitted icons only: from the build's snapshot, the icon's batch, the modules it imports and the set's config, plus a `definitions.mjs` that imports just those icons. Other icons, unrelated files and later edits stay out. Its preview sheets are the build's review sheets (Light and Strong at `preview`'s defaults), rendered again from the verified build: sheets you rendered with other options, or of an older build, never ship, and a missing or changed review sheet is rewritten in `.generative-icons/previews/<id>/` (`regenerated` lists them). A source file that is or runs through a symlink is refused (`SYMLINK_PATH`), and so is a build of a `definitions.mjs` edited by hand (keep one import per batch, as `create` writes it). When an icon's imports cannot be traced, its bundle would hold every module its build loaded, other icons' batches included, so `submit` refuses it (`CHECK_FAILED`) before writing or sending anything; fix the imports and rebuild, or the user passes `--include-untraced` to send them all. A bundled file whose name holds a backslash, a colon or a control character, or is a Windows device name such as `con` or `nul`, is refused (`INVALID_PATH`); rename it.

## Agents with MCP

`generative-icons mcp` serves the same flow as tools: `search`, `add`, `context`, `create`, `check`, `draft` and `preview` (which return the sheets as images), `build`, `export`. Arguments must match each tool's input schema exactly: an unknown or misspelled name, a missing one or a wrong type is refused (`invalid_params`) with the names the tool takes. `build`, `check` and `add` take `ids` (a list); `preview` and `create` take `id`. A tool's `cwd` must be a folder inside the server's. Each call runs as a process of its own: icon code that runs too long ends that call with `SOURCE_ERROR` (any call stops after 30 minutes), and the server keeps answering meanwhile. Build one icon per `build` call; with a progress token, each exported action arrives as a progress notification. A server started with `--offline` refuses `build` (`NETWORK`). A line that is not a JSON-RPC message is answered with an error (-32700, or -32600 for JSON that is not a request) and the server keeps serving. Add the server to the agent's MCP settings only when the user asks: `{"mcpServers": {"generative-icons": {"command": "npx", "args": ["generative-icons", "mcp"]}}}`. To run a binary that was not installed from npm, use its absolute path as `command` with `"args": ["mcp"]`; `init` prints that entry as `mcpLocal`.

## Rules

- Never hand-edit generated files: installed `.lottie` assets, descriptors, `settings.*`, `.generative-icons/`, this skill, or the kit in `generative-icons/library/`. Change `generative-icons.json` or your batch source, then rerun the command.
- `MODIFIED_FILE` means someone edited a managed file. Run `generative-icons diff`, show the user, and let them decide. Never delete or overwrite a file to get past it. An edited kit copy is fixed with `generative-icons init --update-kit`.
- Colors are six-digit hex; resolve theme tokens first. Styles are `outline`, `solid` or `duotone`. Weight is 1 to 3 (outline only); iOS and Android players show 1.5, 2 or 2.5 unless `add --bake-weight <w>` bakes another.
- Light and Strong are separate authored motions. Never fake Strong by changing speed.
- Every action needs an event, not a nudge: animate what the object does for the user (a microphone takes sound, send leaves, a bell rings), show the cause briefly with parts hidden at rest (sound, sparks, wind, a trail), draw marks that mean something, and move in beats that get smaller with the parts in order. The subject is drawn once: the motion moves, turns, scales or draws it and brings it back, never a copy swapped in. The full rules are in [motion-principles.md](references/motion-principles.md) and [design-principles.md](references/design-principles.md).
- Decorative icons need no label; the control around them carries the accessible name. Respect reduced motion.
- Report every check you did not run. Never claim you reviewed a preview you did not open.
- Do not edit `AGENTS.md`, `CLAUDE.md`, MCP settings or global configuration unless the user asks.
