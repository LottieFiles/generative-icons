# Architecture

This repository holds the editable icon source, the authoring toolchain, the local gallery and the npm packages. The website consumes the published packages.

| Location | Responsibility |
| --- | --- |
| `packages/icons/` | Generated per-icon descriptors, native assets, catalog and bundled docs |
| `packages/runtime/` | Shared playback, paint, state, reduced motion and lifecycle API |
| `packages/react/` | Client-side React adapter with controlled state |
| `packages/cli/` and `skills/lottie-icons/` | Project detection, setup, search, safe asset installation and the consumer agent skill |
| `prototype/icons/catalog.json` | Ordered set manifests and browse categories |
| `prototype/icons/01/` | Cumulative local gallery |
| `prototype/icons/NN/assets/` | Distributed JSON, dotLottie and set metadata |
| `prototype/icons/03/controller.mjs` | Shared playback state, loop and interaction behavior |
| `prototype/vendor/` | Pinned native browser player, WASM and license |
| `motion/icon-sets/NN-name/` | Editable definitions, builder, packaging, set checks and verification records |
| `motion/icon-sets/library/` | Family contract, context search, source screening and shared authoring helpers |
| `motion/src/` | Engine scene bridge and exporter client |
| `.agents/skills/animated-icon-workflow/` | AI operating guide and review rules |
| `scripts/` | Local preview, package builds and repository checks |

## Data flow

Named vector geometry and motion → sampled Engine scenes → native Lottie JSON → themed, slot-enabled dotLottie files → package descriptors and assets.

The gallery loads catalog manifests and plays the native files. The browser controller owns interaction timing; drawing choreography stays in authoring source. Intermediate `scenes/` and `native/` directories are ignored by Git; final assets are tracked. The aggregate download is regenerated with `npm run package:icons`.

## Agent workflow

One coding agent owns creative decisions, authoring, inspection and revision. The harness supplies reference geometry, shared rules, source screening, native build commands and verification; it never calls a model itself. See [creating icons](creating-icons.md).

## Family contract

`motion/icon-sets/library/family-contract.json` is the numeric source shared by generation context and preflight: 32-unit grid, 1.5/2/2.5 outline weights, round joins and caps, 18% Duotone tint on eligible surfaces, Light/Strong profiles, 60 fps and a 2.4-second family timeline. Continuous indicators declare an explicit cycle policy.
