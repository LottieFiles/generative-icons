# Development

This repository holds the icon library and the `generative-icons` CLI, a Rust workspace in `crates/`. The CLI builds, checks and packages the whole catalog byte for byte the way the earlier JavaScript and Python pipeline did. That pipeline (`motion/`, `scripts/` and each set's `build.mjs`, `verify.mjs` and `package.py`) stays in the repository as legacy: the Rust parity tests run it as a reference, but new work goes through the CLI.

## Requirements

- **Rust** 1.89 or newer. CI lints and tests with Rust 1.93.1.
- **clang** for C and C++. The vendored QuickJS compiles V8's `ieee754.cc`, which must be built by clang; on Linux set `CC=clang CXX=clang++`. The renderer's bindings also need libclang (`libclang-dev` on Debian and Ubuntu).
- **`CXXFLAGS` with `-ffp-contract=off`.** The renderer must not fuse multiply-adds, or its pixels drift from the browser player's. `.cargo/config.toml` sets it, but a `CXXFLAGS` already in your environment replaces that value, so add the flag to it.
- **x86_64 needs FMA** (Intel Haswell, AMD Piledriver or newer). On Apple silicon, use the arm64 build: Rosetta has no FMA.
- **Node 22+ and Python 3** for the local gallery, the web packages and the repository checks, and for the legacy pipeline.

## Build the CLI

```sh
cargo build --release -p generative-icons
target/release/generative-icons --version
```

Put `target/release` on your `PATH`, or set `GEN_ANIMATED_ICONS_BINARY` to the binary so the npm shim runs it (`node npm/generative-icons/bin/generative-icons.js` in this checkout; `npx generative-icons` once the package is published at launch). The [CLI reference](https://github.com/LottieFiles/generative-icons/blob/main/docs/cli.md) lists every command.

## Test

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                                       # unit tests, plus the oracle tests whose inputs exist
cargo test --workspace --release -- --ignored --skip live_   # whole-catalog parity; takes several minutes
```

Oracle tests compare the CLI with committed files, or with the legacy tools run side by side, and skip when an input is missing:

- intermediate `motion/icon-sets/*/scenes/` and `native/` folders, which Git ignores;
- the built `packages/*/dist` folders (`npm run build:packages`);
- `node` with `motion/node_modules` (`npm ci --prefix motion`), and `python3`, for the side-by-side runs.

Tests never touch your sign-in. They use a temporary `GEN_ANIMATED_ICONS_HOME`, a fake keychain, or `GAI_ENGINE_TOKEN` with a local mock engine. Never run `login` or `logout` from a test or script. Only the two live tests use a real sign-in: `live_exports_match_the_committed_natives_except_path_names` (gai-engine) and `live_build_through_the_hosted_engine` (the CLI). They are ignored by default, and `--skip live_` keeps the whole-catalog run above from starting them. They export on the hosted engine with your session and refresh it, so run them only on purpose, after signing in yourself with `generative-icons login`:

```sh
cargo test --release -p gai-engine --test live -- --ignored live_
cargo test --release -p generative-icons --test engine -- --ignored live_
```

The repository's other checks still run on Node and Python:

```sh
npm ci --prefix motion   # dependencies of the legacy pipeline
npm test                 # gallery controller, generation harness and set scaffold tests
npm run check:docs       # links, publication hygiene, retired names and guide hashes
```

## Author and build a library set

New sets go through the CLI. Building exports on the hosted engine, so sign in first with `generative-icons login`; the hosted engine is internal until launch.

```sh
generative-icons library new-set landscaping --name "Landscaping"
# draw and animate in motion/icon-sets/NN-landscaping/definitions.mjs and batches/
generative-icons library preflight NN-landscaping
generative-icons library build NN-landscaping
generative-icons library verify NN-landscaping
generative-icons library verify-visual NN-landscaping
generative-icons library register NN-landscaping
generative-icons library package NN-landscaping   # repackage with the final evidence
generative-icons library package --icon-library   # refresh icon-library.zip and delivery.json
```

- Registration needs a built, nonempty manifest, a passing preflight and passing verification. Registering a missing manifest would stop the catalog checks from running.
- A set can be made only of spec icons. Scaffold it with `new-set`, leave its `definitions.mjs` exporting no icons, build each icon in a project and add it with `library adopt NN --add <build>` ([below](#reanimate-a-library-icon-with-a-spec)). The first addition writes the set's `set.json` from its `set-config.json`. Preflight passes the set once every icon in its manifest is adopted from `specs/`, `library build` keeps them, and verification and registration run as for any set.
- Sets 01 (Heart/Bell) and 02 (Search) are frozen: their assets are verified as they are and never rebuilt. Sets 03–06 build through built-in legacy profiles.
- Builders replace a set's outputs. Build complete sets and confirm that unrelated asset hashes are unchanged. Do not rebuild only to preview existing files: `generative-icons preview <id>` renders any icon.
- Intermediate `scenes/` and `native/` folders are rebuildable and ignored by Git; final assets under `prototype/icons/NN/assets/` are tracked.

The [creating icons](creating-icons.md) guide covers the design loop; [verification](verification.md) covers the checks.

## Reanimate a library icon with a spec

A library icon can be replaced by a spec v2 icon of the same id. Start it in any project with `create <id> --replace`, author and build it there, then adopt the build into its set:

```sh
generative-icons create airplane --replace       # in a scratch project: the icon's spec, or its v1 drawing ported
# write the motion in generative-icons/icons/airplane.icon.json
generative-icons check airplane && generative-icons draft airplane
generative-icons build airplane --no-install
generative-icons library adopt 14 path/to/project/.generative-icons/build/airplane
generative-icons library verify 14
generative-icons library verify-visual 14
```

- `create --replace` keeps the library id. An icon already reanimated starts from its own spec. Any other starts from its v1 drawing, ported from the Light animation at frame 0: one part per outline layer, and the tint and Solid layers as the part's own where the geometry matches, else as parts that draw only in that style, so every style keeps v1's shapes. The port has no motion (`check` reports an empty `motion`). It carries rest opacity, Solid-only strokes, per-weight geometry and the reverse name, and its warnings name what it could not: geometry only another weight draws, a Solid stroke other than 2.6, an even-odd Solid fill, a paint with no style slot, a trim at frame 0, and keys or shape items the format lacks.
- `adopt` copies the spec to `motion/icon-sets/NN-…/specs/<id>.icon.json` and the actions into the set's assets, replaces the icon's `set.json` entry (keeping its number, category and state labels) and repackages the set. The build must have the icon's actions: the same reverse or reset actions for a stateful icon.
- The rest pose and, for a stateful icon, the end state must stay the library drawing, so apps that switch versions see no jump.
- From then on the spec is the icon's source. `library build` keeps adopted icons as they are (their old definitions stay for the legacy checks but no longer export), and `library verify` checks them against their spec: hashes, format, paint, themes and markers. Their motion checks run in `build`, and `verify-visual` renders them like every other action.
- Frozen sets 01 and 02 can adopt: they are still never rebuilt, so adopting only replaces the icon's assets and entry, and their legacy checks skip it.
- A new icon joins a set the same way with `library adopt NN --add <build>`: `create <new-id>` (not a library id), build it, then add it, and give it search terms (below). It takes the set's category and the number after the library's last icon, and `library build` keeps it. New icons also change the catalog size that some tests pin (see [crates/README.md](../crates/README.md)), so update those counts in the same commit.

What the library-wide reanimation (Oct 2026, 412 icons) learned about porting, beyond [authoring-v2.md](../crates/generative-icons/assets/skill/references/authoring-v2.md):

- Decide first: keep an icon whose motion already is its meaning, with rhythm in Strong, readable at 24 px (a dial indexing to its next stop, a refresh arrow's full turn, a toggle knob landing). Reanimate generic nudges, lifts and tilts, and moves under a pixel at 24 px.
- Port the rest drawing exactly, vertex for vertex, and match the icon's Solid (`"solid": "fill"` or `"ring"`): the default Solid adds a rim the library drawing does not have.
- A stateful icon ends on the library's end state (compare the build's last frame with the old one) and keeps its animation ids: a reverse apps play under another name (`lock-unlock`, `reconnect-disconnect`, `suitcase-retract`, `menu-open`, `check-reset`) keeps it with `state.reverseName`, which `create --replace` fills in.
- A `previewReset` icon uses `"reverse": "fade"` with states `prepared` → `result`.
- Some sets protect openings (counters, ring holes, Solid openings) in every frame: read the set's entry in `crates/gai-render/src/rules.rs` before designing, so the motion never shrinks, fills or covers one.
- `build` runs the visual rules every set shares; a set's own rules run only in `library verify-visual NN`, so run it after adopting and before you commit.
- The port carries what v1 drew: a part resting translucent takes `"opacity"` (gradient-stops' ramp bands), a stroke drawn only in Solid `"outline": "none"`, and geometry that changes with the weight `"weights"` (more-options' dots). Continuous loaders stay v1.

## Search terms

Each set's `search-terms.json` maps every icon id to the words and short phrases people type to find it: other names for what is drawn (bin, garbage), the action in UI words (delete, remove), button text (log out, add to cart), what an object stands for (doctor on stethoscope) and real abbreviations (2fa, otp). The packages ship them as the descriptor's `aliases`, so the website, `search` and apps all find the icon by them.

- Every library icon needs at least 3 terms, none of them its own label or id; a test checks the whole catalog.
- Terms are lowercase letters, digits, single spaces and hyphens, 1 to 40 characters, at most 24 per icon.
- Leave a term to the icon that is the better answer for it, and skip words that would match hundreds of icons (icon, button, outline) or describe the motion.
- Maintainers judge the website's search on the queries in the internal website repository (`search/queries.json`, `npm run search:eval`) and check each batch of new terms there before release.

## Local gallery

`npm run dev` (or `python3 scripts/serve.py`) serves `prototype/` on port 4189; the gallery is at `/icons/01/`. Pass `-- --port 4190` to change the port. The gallery vendors its player and fonts and needs no build step.

## Packages

```sh
generative-icons library packages    # packages/*/dist and the search sidecar
npm run test:packages                  # package unit tests
generative-icons library release --output dist/packages-vX.Y.Z
npm run test:consumers -- dist/packages-vX.Y.Z   # install the tarballs into vanilla, React and Next.js fixtures
```

`npm run build:packages` and `npm run pack:release` are the legacy equivalents. See the [package contract](package-contract.md) and the [release runbook](https://github.com/LottieFiles/generative-icons/blob/main/docs/maintainers/releasing.md).

## AI guide

`npm run docs:build` regenerates `prototype/icons/guide/` (the downloadable AI operating guide and its source hashes) from the canonical documents. Edit the canonical source, never the generated copy, then run `npm run check:docs`.
