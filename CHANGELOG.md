# Changelog

All notable changes to the Generative Animated Icons packages and the `gen-animated-icons` CLI are recorded here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the packages follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html). The packages and the CLI share one version.

## [Unreleased]

### Added

- `gen-animated-icons`, one binary for macOS and Linux (arm64 and x64) that finds, installs, creates, checks, previews and submits icons in web, iOS and Android apps, run with `npx gen-animated-icons`.
  - `init`, `info`, `search`, `add`, `diff`, `remove` and `doctor` set up an app and manage its icons; iOS (SwiftUI, UIKit) and Android (Compose, Views) apps get `.lottie` assets and JSON descriptors, and `add --bake-weight` bakes an outline weight into native files.
  - `context`, `create`, `check`, `build` and `preview` let a coding agent create a missing icon in the app's own project with the family's authoring kit. `build` exports on LottieFiles' hosted engine and needs a LottieFiles sign-in (`login`, `logout`, `whoami`).
  - `submit` sends verified local icons for review; `--dry-run` writes the bundle.
  - `mcp` serves the same flow to coding agents as MCP tools.
  - `init` installs the `gen-animated-icons` agent skill for Claude Code (`.claude/skills/`) and other agents (`.agents/skills/`), and `init --migrate` moves a `lottie-icons` project over.
  - `library` commands build, verify, measure, package, register and release the library's own sets. They reproduce the earlier JavaScript, Python and browser pipeline's outputs byte for byte, including the renderer checks that `verify.html` ran in a browser.
- The `gen-animated-icons` npm wrapper and one npm package per platform binary.
- Spec v2 (`gai-icon/2`), the default format for new icons: plain JSON parts, rigs and eased keys, compiled to one hosted-engine call. `create` writes a spec (`--batch` writes a v1 JS batch instead), `check` validates, compiles and lints it offline, and `draft` renders a review sheet. Specs can rest a part translucent (`"opacity"`), draw a stroke only in Solid (`"outline": "none"`), change geometry with the weight (`"weights"`) and keep a custom reverse animation name (`state.reverseName`).
- 136 new icons in 14 categories, so every category has at least 20: 1,141 icons in 58 sets.
- Every spec icon's editable source ships in `@lottiefiles/generative-icons` as `specs/<id>.icon.json`, hashed in `release.json`. `create <id> --from <library-id>` starts from it, from a checkout, an installed package or the hosted library.
- `library adopt` replaces a library icon with a spec v2 build and `library adopt --add` adds a new one to a set; `create <library-id> --replace` starts a reanimation from the icon's spec, or from its v1 drawing ported to a spec.
- The CLI says when it is waiting for macOS Keychain approval (after 3 s) and stops with an error after 120 s.
- CI for the Rust workspace on macOS and Linux, and a manual release workflow that builds the binaries, drafts a GitHub release and packs the npm packages without publishing them.

### Changed

- Documentation, the maintainer skill and the contribution flow use `gen-animated-icons`. Creating icons no longer needs a clone of this repository.
- Native export runs only on LottieFiles' hosted engine.
- 414 library icons are reanimated with spec v2: each action now plays an event (something arrives, leaves, draws on or lands) instead of one move there and back. Each icon keeps its earlier drawing at rest, and its animation names are unchanged.
- **Breaking for apps that use state names:** 33 stateful icons' `startState` and `endState` are now kebab-case ids (`Voice A` → `voice-a`, `Sharing` → `sharing`), and `background-run`'s states are `prepared` → `result` (were `Foreground` → `Running behind`).

### Deprecated

- The `lottie-icons` command (`@lottiefiles/generative-icons-cli`) and its skill in `skills/lottie-icons/`, superseded by `gen-animated-icons`. They are kept frozen for migration.
- The JavaScript and Python build, verify and package scripts in `motion/` and `scripts/`. They remain as references for the CLI's parity tests.

### Removed

- The local Engine exporter setup: `CREATOR_MCP_BIN`, `ENGINE_MCP_ROOT`, `.env.example`, `motion/src/exporter-config.mjs`, `motion/src/doctor.mjs`, and the `setup`, `doctor` and `test:tooling` npm scripts. `motion/src/export.mjs` now stops with a pointer to `gen-animated-icons library build`.
- The optional Claude Code design adapter (`claude_design.py`); `gen-animated-icons context --prompt` prints the design prompt it sent.

## [0.1.0] - TBD

### Added

- 1,005 animated icons in 58 sets, each with Light and Strong motion, Outline (three weights), Solid and Duotone styles, light and dark themes, and a recolorable primary slot.
- `@lottiefiles/generative-icons`: per-icon descriptors, native dotLottie assets, catalog, schemas and docs.
- `@lottiefiles/generative-icons-runtime`: playback, paint, state, reduced motion and lifecycle API.
- `@lottiefiles/generative-icons-react`: `AnimatedIcon` component for React and Next.js.
- `@lottiefiles/generative-icons-cli`: `lottie-icons` command for project setup, search and safe asset installation, plus a consumer agent skill.
- Editable icon source, family contract, AI operating guide and validation tooling for creating new icons.

[Unreleased]: https://github.com/LottieFiles/lottie-generative-icons/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/LottieFiles/lottie-generative-icons/releases/tag/v0.1.0
