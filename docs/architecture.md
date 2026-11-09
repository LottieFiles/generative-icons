# Architecture

This repository holds the editable icon source, the `generative-icons` CLI, the local gallery and the npm packages. The website consumes the published packages.

| Location | Responsibility |
| --- | --- |
| `crates/` | The `generative-icons` Rust workspace: authoring runtime, builds, checks, rendering, packaging, the hosted-engine client and the CLI. See [crates/README.md](../crates/README.md). |
| `skills/generative-icons/` | The agent skill the CLI embeds and installs into apps (the one source for it). `npx skills add` and plugin directories find it there; `plugin.json` is the Agent Plugins manifest around it. |
| `npm/` | The `generative-icons` npm wrapper and one binary package per platform |
| `packages/icons/` | Generated per-icon descriptors, native assets, catalog and bundled docs |
| `packages/runtime/` | Shared playback, paint, state, reduced motion and lifecycle API |
| `packages/react/` | Client-side React adapter with controlled state |
| `prototype/icons/catalog.json` | Ordered set manifests and browse categories |
| `prototype/icons/01/` | Cumulative local gallery |
| `prototype/icons/NN/assets/` | Distributed JSON, dotLottie and set metadata |
| `prototype/icons/03/controller.mjs` | Shared playback state, loop and interaction behavior |
| `prototype/vendor/` | Pinned native browser player, WASM and license |
| `motion/icon-sets/NN-name/` | Editable definitions, set rules, verification records, and the legacy builder, packaging and set checks |
| `motion/icon-sets/library/` | Family contract, authoring kit, context search and the legacy source screening |
| `motion/src/` | The legacy Engine scene template |
| `.agents/skills/animated-icon-workflow/` | Maintainer workflow and review rules for this repository (marked internal, so `npx skills add` doesn't offer it) |
| `scripts/` | Local gallery, package builds and repository checks |

## Data flow

Named vector geometry and motion (`definitions.mjs`, run in the CLI's embedded QuickJS) → sampled scenes → native Lottie from the hosted engine → themed, slot-enabled dotLottie files → package descriptors and assets.

The gallery loads catalog manifests and plays the native files. The browser controller owns interaction timing; drawing choreography stays in authoring source. Intermediate `scenes/` and `native/` folders are ignored by Git; final assets are tracked. `generative-icons library package --icon-library` regenerates the aggregate download.

The earlier JavaScript and Python pipeline (`motion/`, `scripts/`, each set's `build.mjs`, `verify.mjs` and `package.py`, and the browser `verify.html` pages) is legacy. The CLI reproduces its outputs byte for byte, and its parity tests still run it as a reference.

## Agent workflow

One coding agent owns creative decisions, authoring, inspection and revision. The CLI supplies reference geometry, shared rules, source screening, builds and verification; it never calls a model itself. See [creating icons](creating-icons.md).

## Family contract

`motion/icon-sets/library/family-contract.json` is the numeric source shared by generation context and preflight: 32-unit grid, 1.5/2/2.5 outline weights, round joins and caps, 18% Duotone tint on eligible surfaces, Light/Strong profiles, 60 fps and a 2.4-second family timeline. Continuous indicators declare an explicit cycle policy.
