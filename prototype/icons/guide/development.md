# Development

This repository holds the icon library and the `gen-animated-icons` CLI, a Rust workspace in `crates/`. The CLI builds, checks and packages the whole catalog byte for byte the way the earlier JavaScript and Python pipeline did. That pipeline (`motion/`, `scripts/` and each set's `build.mjs`, `verify.mjs` and `package.py`) stays in the repository as legacy: the Rust parity tests run it as a reference, but new work goes through the CLI.

## Requirements

- **Rust** 1.89 or newer. CI lints and tests with Rust 1.93.1.
- **clang** for C and C++. The vendored QuickJS compiles V8's `ieee754.cc`, which must be built by clang; on Linux set `CC=clang CXX=clang++`. The renderer's bindings also need libclang (`libclang-dev` on Debian and Ubuntu).
- **`CXXFLAGS` with `-ffp-contract=off`.** The renderer must not fuse multiply-adds, or its pixels drift from the browser player's. `.cargo/config.toml` sets it, but a `CXXFLAGS` already in your environment replaces that value, so add the flag to it.
- **x86_64 needs FMA** (Intel Haswell, AMD Piledriver or newer). On Apple silicon, use the arm64 build: Rosetta has no FMA.
- **Node 22+ and Python 3** for the local gallery, the web packages and the repository checks, and for the legacy pipeline.

## Build the CLI

```sh
cargo build --release -p gen-animated-icons
target/release/gen-animated-icons --version
```

Put `target/release` on your `PATH`, or set `GEN_ANIMATED_ICONS_BINARY` to the binary so the npm shim runs it (`node npm/gen-animated-icons/bin/gen-animated-icons.js` in this checkout; `npx gen-animated-icons` once the package is published at launch). The [CLI reference](https://github.com/LottieFiles/lottie-generative-icons/blob/main/docs/cli.md) lists every command.

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

Tests never touch your sign-in. They use a temporary `GEN_ANIMATED_ICONS_HOME`, a fake keychain, or `GAI_ENGINE_TOKEN` with a local mock engine. Never run `login` or `logout` from a test or script. Only the two live tests use a real sign-in: `live_exports_match_the_committed_natives_except_path_names` (gai-engine) and `live_build_through_the_hosted_engine` (the CLI). They are ignored by default, and `--skip live_` keeps the whole-catalog run above from starting them. They export on the hosted engine with your session and refresh it, so run them only on purpose, after signing in yourself with `gen-animated-icons login`:

```sh
cargo test --release -p gai-engine --test live -- --ignored live_
cargo test --release -p gen-animated-icons --test engine -- --ignored live_
```

The repository's other checks still run on Node and Python:

```sh
npm ci --prefix motion   # dependencies of the legacy pipeline
npm test                 # gallery controller, generation harness and set scaffold tests
npm run check:docs       # links, publication hygiene, retired names and guide hashes
```

## Author and build a library set

New sets go through the CLI. Building exports on the hosted engine, so sign in first with `gen-animated-icons login`; the hosted engine is internal until launch.

```sh
gen-animated-icons library new-set landscaping --name "Landscaping"
# draw and animate in motion/icon-sets/NN-landscaping/definitions.mjs and batches/
gen-animated-icons library preflight NN-landscaping
gen-animated-icons library build NN-landscaping
gen-animated-icons library verify NN-landscaping
gen-animated-icons library verify-visual NN-landscaping
gen-animated-icons library register NN-landscaping
gen-animated-icons library package NN-landscaping   # repackage with the final evidence
gen-animated-icons library package --icon-library   # refresh icon-library.zip and delivery.json
```

- Registration needs a built, nonempty manifest, a passing preflight and passing verification. Registering a missing manifest would stop the catalog checks from running.
- Sets 01 (Heart/Bell) and 02 (Search) are frozen: their assets are verified as they are and never rebuilt. Sets 03–06 build through built-in legacy profiles.
- Builders replace a set's outputs. Build complete sets and confirm that unrelated asset hashes are unchanged. Do not rebuild only to preview existing files: `gen-animated-icons preview <id>` renders any icon.
- Intermediate `scenes/` and `native/` folders are rebuildable and ignored by Git; final assets under `prototype/icons/NN/assets/` are tracked.

The [creating icons](https://github.com/LottieFiles/lottie-generative-icons/blob/main/docs/creating-icons.md) guide covers the design loop; [verification](https://github.com/LottieFiles/lottie-generative-icons/blob/main/docs/verification.md) covers the checks.

## Reanimate a library icon with a spec

A library icon can be replaced by a spec v2 icon of the same id. Author and build it in any project, then adopt the build into its set:

```sh
gen-animated-icons create airplane --spec          # in a scratch project: port the drawing, write the motion
gen-animated-icons check airplane && gen-animated-icons draft airplane
gen-animated-icons build airplane --no-install
gen-animated-icons library adopt 14 path/to/project/.gen-animated-icons/build/airplane
gen-animated-icons library verify 14
gen-animated-icons library verify-visual 14
```

- `adopt` copies the spec to `motion/icon-sets/NN-…/specs/<id>.icon.json` and the actions into the set's assets, replaces the icon's `set.json` entry (keeping its number, category and state labels) and repackages the set. The build must have the icon's actions: the same reverse or reset actions for a stateful icon.
- The rest pose and, for a stateful icon, the end state must stay the library drawing, so apps that switch versions see no jump.
- From then on the spec is the icon's source. `library build` keeps adopted icons as they are (their old definitions stay for the legacy checks but no longer export), and `library verify` checks them against their spec: hashes, format, paint, themes and markers. Their motion checks run in `build`, and `verify-visual` renders them like every other action.
- Frozen sets 01 and 02 cannot adopt.

## Local gallery

`npm run dev` (or `python3 scripts/serve.py`) serves `prototype/` on port 4189; the gallery is at `/icons/01/`. Pass `-- --port 4190` to change the port. The gallery vendors its player and fonts and needs no build step.

## Packages

```sh
gen-animated-icons library packages    # packages/*/dist and the search sidecar
npm run test:packages                  # package unit tests
gen-animated-icons library release --output dist/packages-vX.Y.Z
npm run test:consumers -- dist/packages-vX.Y.Z   # install the tarballs into vanilla, React and Next.js fixtures
```

`npm run build:packages` and `npm run pack:release` are the legacy equivalents. See the [package contract](https://github.com/LottieFiles/lottie-generative-icons/blob/main/docs/package-contract.md) and the [release runbook](https://github.com/LottieFiles/lottie-generative-icons/blob/main/docs/maintainers/releasing.md).

## AI guide

`npm run docs:build` regenerates `prototype/icons/guide/` (the downloadable AI operating guide and its source hashes) from the canonical documents. Edit the canonical source, never the generated copy, then run `npm run check:docs`.
