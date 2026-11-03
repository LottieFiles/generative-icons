# Changelog

All notable changes to the Generative Icons packages and the `generative-icons` CLI are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the packages follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The packages and the CLI share one version.

## [Unreleased]

## [0.1.0] - 2026-11-03

The first release. Changed, Deprecated and Removed compare it with the pre-release builds of this repository.

### Added

- 1,161 animated icons in 59 sets, each with Light and Strong motion, Outline (three weights), Solid and Duotone styles, light and dark themes, and a recolorable primary slot.
- `@lottiefiles/generative-icons`: per-icon descriptors, native dotLottie assets, catalog, schemas and docs.
- `@lottiefiles/generative-icons-runtime`: playback, paint, state, reduced motion and lifecycle API.
- `@lottiefiles/generative-icons-react`: `AnimatedIcon` component for React and Next.js.
- Editable icon source, family contract, AI operating guide and validation tooling for creating new icons.
- `generative-icons`, one binary for macOS and Linux (arm64 and x64) that finds, installs, creates, checks, previews and submits icons in web, iOS and Android apps, run with `npx generative-icons`.
  - `init`, `info`, `search`, `add`, `diff`, `remove` and `doctor` set up an app and manage its icons; iOS (SwiftUI, UIKit) and Android (Compose, Views) apps get `.lottie` assets and JSON descriptors, and `add --bake-weight` bakes an outline weight into native files.
  - `context`, `create`, `check`, `build` and `preview` let a coding agent create a missing icon in the app's own project with the family's authoring kit. `build` exports on Creator MCP, LottieFiles' hosted engine, and needs a LottieFiles sign-in (`login`, `logout`, `whoami`).
  - `submit` sends verified local icons for review; `--dry-run` writes the bundle.
  - `mcp` serves the same flow to coding agents as MCP tools.
  - `init` installs the `generative-icons` agent skill for Claude Code (`.claude/skills/`) and other agents (`.agents/skills/`), and `init --migrate` moves a `lottie-icons` project over, or one set up while the CLI was called `gen-animated-icons` (`gen-animated-icons.json`); that one keeps its icon paths.
  - `library` commands build, verify, measure, package, register and release the library's own sets. They reproduce the earlier JavaScript, Python and browser pipeline's outputs byte for byte, including the renderer checks that `verify.html` ran in a browser.
- The `generative-icons` npm wrapper and one npm package per platform binary.
- Spec v2 (`gai-icon/2`), the default format for new icons: plain JSON parts, rigs and eased keys, compiled to one hosted-engine call. `create` writes a spec (`--batch` writes a v1 JS batch instead), `check` validates, compiles and lints it offline, and `draft` renders a review sheet. Specs can rest a part translucent (`"opacity"`), draw a stroke only in Solid (`"outline": "none"`), change geometry with the weight (`"weights"`) and keep a custom reverse animation name (`state.reverseName`).
- Emoji, a set of the 20 most-used emoji from Unicode's frequency ranking (😂 ❤️ 🤣 👍 😭 🙏 😘 🥰 😍 😊 🎉 😁 💕 🥺 😅 🔥 🤦 🤷 🙄 😆), each drawn on one shared face with its own event: tears spurt, the kiss flies off, the eyes roll.
- Every spec icon's editable source ships in `@lottiefiles/generative-icons` as `specs/<id>.icon.json`, hashed in `release.json`. `create <id> --from <library-id>` starts from it, from a checkout, an installed package or the hosted library.
- `library adopt` replaces a library icon with a spec v2 build and `library adopt --add` adds a new one to a set; `create <library-id> --replace` starts a reanimation from the icon's spec, or from its v1 drawing ported to a spec.
- A library set can be made only of spec icons: the first `library adopt --add` into a new set writes its `set.json` from `set-config.json`, and `library preflight` passes a set whose `definitions.mjs` exports no icons once every icon in its manifest is adopted from `specs/`.
- `generative-icons export <id>` writes an icon as an animated SVG that browsers play without a player (SMIL). Stateful icons toggle on click, loaders loop and the rest play on hover, or pick a trigger with `--trigger`. The MCP server serves it as `export`, and `@lottiefiles/generative-icons-runtime/svg` (`iconToSvg`, `dotLottieToSvg`) makes the same SVG in JavaScript.
- The agent skill ships the family's whole direction, generated from the library's own docs: motion principles, the motion system, drawing principles, drawing and motion craft, and the polish review. Creating an icon starts by reading it and writing a brief (the event as cause → action → response → final state, the state policy, Light and Strong), the MCP instructions point to it, and `context --prompt` includes it in an app instead of the v1 authoring guide.
- `check` warns `NUDGE` when a gesture's whole drawing only tilts under 20° and comes back with nothing drawing on, appearing, landing or holding. None of the library's 550 specs warns. The starter spec `create` writes now rings: ring arcs hidden at rest flick out and fade as the bell swings.
- `check` warns `DUPLICATE_DRAWING` when a part hidden at rest draws exactly the shape of a shown part that fades out: the subject swapped for a copy of itself. The family draws its subject once and transforms it, and a test keeps every library spec free of it.
- The CLI says when it is waiting for macOS Keychain approval (after 3 s) and stops with an error after 120 s.
- CI for the Rust workspace on macOS and Linux, and a manual release workflow that builds the binaries, drafts a GitHub release and packs the npm packages without publishing them.

### Changed

- MOTION.md allows the subtract masks and inverted alpha mattes that spec v2 Solid cut-outs export; it had listed masks and mattes as denied.
- Documentation, the maintainer skill and the contribution flow use `generative-icons`. Creating icons no longer needs a clone of this repository.
- Native export runs only on Creator MCP, LottieFiles' hosted engine. The CLI uses its production server, `https://creator.mcp.lottiefiles.com/mcp`, by default. A sign-in made against the earlier development server, or against `creator-mcp.lottiefiles.com` (Creator MCP's production host until 2 Nov 2026), doesn't carry over: run `generative-icons login` again. `--engine-url` or `GAI_ENGINE_URL` still choose another engine.
- 414 library icons are reanimated with spec v2: each action now plays an event (something arrives, leaves, draws on or lands) instead of one move there and back. Each icon keeps its earlier drawing at rest, and its animation names are unchanged.
- 13 icons that swapped their subject for a cross-faded copy now move the one drawing they have, with cues hidden at rest for the cause: active-speaker, api-quota, export-video, face-blowing-a-kiss, freeze-column, gradient-step, grinning-face-with-sweat, insert-row, next-step, schedule-send, snowflake, thumbnail and trackpad. Their rest poses and animation names are unchanged.
- The AI operating guide now bundles the family's direction (motion principles, the motion system, drawing principles, the polish review), the spec format and the preview review, and its workflow authors specs and adds them with `library adopt`.
- **Breaking for apps that use state names:** 33 stateful icons' `startState` and `endState` are now kebab-case ids (`Voice A` → `voice-a`, `Sharing` → `sharing`), and `background-run`'s states are `prepared` → `result` (were `Foreground` → `Running behind`).

### Deprecated

- The `lottie-icons` command (`@lottiefiles/generative-icons-cli`) and its skill in `skills/lottie-icons/`, superseded by `generative-icons`. They are kept frozen for migration.
- The JavaScript and Python build, verify and package scripts in `motion/` and `scripts/`. They remain as references for the CLI's parity tests.

### Removed

- The local Engine exporter setup: `CREATOR_MCP_BIN`, `ENGINE_MCP_ROOT`, `.env.example`, `motion/src/exporter-config.mjs`, `motion/src/doctor.mjs`, and the `setup`, `doctor` and `test:tooling` npm scripts. `motion/src/export.mjs` now stops with a pointer to `generative-icons library build`.
- The optional Claude Code design adapter (`claude_design.py`); `generative-icons context --prompt` prints the design prompt it sent.

[Unreleased]: https://github.com/LottieFiles/generative-icons/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/LottieFiles/generative-icons/releases/tag/v0.1.0
