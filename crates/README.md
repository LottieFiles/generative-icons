# generative-icons (Rust workspace)

`generative-icons` is one binary that finds, creates, builds, checks and packages animated icons. It replaces the Node, Python and browser pipeline in `motion/`, `scripts/` and `prototype/icons/*/verify.html`, and the earlier npm command `lottie-icons` (`packages/cli`). User docs: [docs/cli.md](../docs/cli.md); contributor setup: [docs/development.md](../docs/development.md).

## Status

Every command is wired, and the whole catalog rebuilds byte for byte through the binary. Measured on aarch64-apple-darwin:

- **Scenes:** 3,094 of 3,094 scenes from the definitions at the parity snapshot (`098688f`), in the embedded QuickJS, legacy sets included.
- **Builds:** all 3,088 animations of the 56 buildable sets at that snapshot rebuild from their natives, with assets, themes, manifests, `.lottie` files, `set.json` and `build.json` identical. Live hosted rebuilds of sets 12 and 23 match the committed natives once each path's `nm` is masked.
- **Checks:** 95 of 95 verification evidence files, the browser evidence of the 24 canonical sets (35–63, 1,428 actions) rendered natively, the 18 motion measurements of sets 22–39, and preflight reports identical to Node's.
- **Packaging:** set zips identical to Python's (the six legacy zips differ only in entry order), `icon-library.zip` equal to `delivery.json`, the npm `dist` folders and tarballs identical.
- **Spec icons:** 550 library icons are spec v2 (`motion/icon-sets/*/specs/`), reanimated or added since the snapshot. Every spec compiles, `library build` keeps their adopted assets, and `library verify` checks them against their specs.
- **Apps:** all 1,161 icons install and uninstall on web and Android, legacy `lottie-icons.json` projects get the old CLI's exact bytes, and 2,322 ranked queries match `generation-context.mjs`.

## Commands

| Group | Commands |
| --- | --- |
| Apps | `init`, `info`, `search`, `add`, `diff`, `remove`, `doctor` |
| Sign-in | `login`, `logout`, `whoami` (Creator MCP OAuth) |
| Creating icons | `context`, `create`, `check`, `build`, `preview`, `submit` |
| Agents | `mcp` (stdio MCP server: search, add, context, create, check, build, preview) |
| Maintainers | `library new-set`, `preflight`, `build`, `verify`, `verify-visual`, `measure`, `package`, `packages`, `register`, `release` |

The hidden `__verify-action` subcommand runs one renderer check in a child process for `library verify-visual`.

## How a build flows

```
definitions.mjs ──► gai-js (QuickJS-NG, V8-exact math) ──► gai-build::scene ──► scene.json
                                                                                   │
                     gai-engine (hosted Creator MCP, implements Exporter) ◄──────┘
                                        │ native Lottie (normalized)
                                        ▼
gai-build::post_export ─► assets, themes, manifests ─► gai-build::records/set_json ─► set.json
                                        │
                     gai-package ─► .lottie, set zip, catalog, npm packages
                                        │
       gai-check (preflight, verify, paint, contracts)   gai-render (dotlottie-rs: visual checks, sheets)
```

### Spec v2

```
icons/<id>.icon.json ─► gai-spec: parse + names + motion + lints (offline, ms) ─► plan (gai-plan/1)
      plan + build-v2.js ─► ONE creator_exec: every motion as rigs, eased keys, masks, mattes, markers
            draft: renderFilmstrip sheet (nothing stored)   build: one bundle file ─► split ─► normalize_native
      gai-build::post_export_v2 (slots by paint name, themes, markers) ─► gai-package .lottie ─► verify ─► install
```

v1 code paths are unchanged; `create` writes a spec unless given `--batch`; `check`, `draft`, `build` and `submit` take the v2 path only for icons with a spec (`check` without ids checks every spec as well), and the MCP server adds a `draft` tool that returns the sheets as images. A draft or build reports its engine round trip split at the script's own `startedAt` (`engine.phases`: before, in and after the script). Authoring reference: `generative-icons/assets/skill/references/authoring-v2.md`. A build's verification includes the library's renderer checks (`gai-render::verify_action` at every size, style, weight and theme).

Library icons can be replaced by specs: `library adopt <set> <build>…` copies a build's spec to the set's `specs/<id>.icon.json` and its actions into the assets, replaces the `set.json` entry (records with `source: "gai-icon/2"`) and repackages. `library build` keeps adopted icons as they are (`gai-build` splices their committed entries back), and `library verify` runs the legacy stages without them and checks them against their spec (`gai-check::verify::adopted`).

## Crates

| Crate | Owns |
|---|---|
| `gai-core` | JS-compatible JSON (`to_js_json`), Python `json.dumps` (`to_py_json`), `js_round`/`round5`, sha256, JS truthiness and `String()`, ordered parallel maps, the `Exporter` trait |
| `gai-js` | Embedded QuickJS-NG with V8-exact `Math`, svgpath, the kit, the `gai:host` bootstrap (`install`, `describe`, `sampleVariant`, `probe`), time limits and the watchdog |
| `gai-build` | Scenes, the legacy `.engine.js` template, profiles (set variants), post-export, records, `set.json`, build orchestration |
| `gai-package` | dotLottie archives (byte-identical ZIP writer), set zips, catalog, release, npm package builds, asset audit |
| `gai-check` | Preflight, set verification (generic checks plus rule data in `rules/`), paint contract, geometry contracts, motion measurement |
| `gai-render` | dotlottie-rs rendering, the visual checks that replace `verify.html`, review sheets |
| `gai-engine` | Hosted Creator MCP client: OAuth sign-in, token storage, export, downloads, native normalization; `PlanRunner` for spec v2 (one call per icon, draft images, bundle split) |
| `gai-svg` | Animated SVG (SMIL) export for `export`: a byte-identical port of `packages/runtime/src/svg.mjs`, held to it by golden files in `tests/golden/` |
| `gai-spec` | Icon spec v2 (`gai-icon/2`): JSON validation with pointer diagnostics, the compiler to a plan (`gai-plan/1`), lints, and the engine script `assets/build-v2.js` |
| `generative-icons` | The CLI: app commands, `library` maintainer commands, the `mcp` server, and the agent skill it installs (`assets/skill/`, the one source of that skill) |

## Build

- **Rust** 1.89 or newer (`rust-version`). CI lints and tests with 1.93.1: newer clippy releases add lints (1.98 flags `chunks_exact` in gai-js), so a toolchain bump is its own change.
- **clang.** `vendor/rquickjs-sys` compiles V8's `ieee754.cc`, which must be built by clang with FMA contraction on (on Linux, `CC=clang CXX=clang++`). dotlottie-rs runs bindgen, so libclang is needed too.
- **`CXXFLAGS=-ffp-contract=off`.** ThorVG must not fuse multiply-adds, or its pixels drift from the browser evidence (only 4 of 24 sets match without it). `.cargo/config.toml` sets it; an environment `CXXFLAGS` replaces that value and must carry the flag. `vendor/rquickjs-sys/build.rs` re-enables contraction for V8's math objects through `CXXFLAGS_<target>`, and adds `-mfma` on x86_64.
- **x86_64 needs FMA at run time.** gai-js checks for it before running JS; Rosetta has none.

```sh
cargo build --release -p generative-icons
cargo clippy --workspace --all-targets -- -D warnings
```

## Tests

```sh
cargo test --workspace                                       # unit tests and oracle tests whose inputs exist
cargo test --workspace --release -- --ignored --skip live_   # whole-catalog parity (several minutes)
```

- **Oracle tests skip when their inputs are missing.** Inputs:
  - the gitignored `motion/icon-sets/*/scenes/` and `native/` folders. Restore the parity snapshot from the repository root with `tar -xzf .local/parity-oracles/scenes-native-098688f.tgz` (it was taken at `098688f`, before any Rust rebuild overwrote them);
  - the V8 math capture for `corpus_math_matches_v8`: `.local/parity-oracles/mathcap/`, or `GAI_MATHCAP_DIR`;
  - `packages/*/dist`, built by `npm run build:packages`. Use the Python build for this reference: `library packages` also writes `search-index.json`, which gai-package's `packages_match_checkout_dist` does not expect;
  - `node` with `motion/node_modules` (`npm ci --prefix motion`), and `python3`, for the side-by-side runs against the legacy tools.
- **Whole-catalog tests are `#[ignore]`.** Run them in release.
- **Sign-in.** Tests never use the real sign-in: they use a temporary `GEN_ANIMATED_ICONS_HOME`, the fake keychain, or `GAI_ENGINE_TOKEN` with a wiremock engine. Never run `login` or `logout` against the default credentials home; an early test once deleted a real refresh token. The two live tests (`live_exports_match_the_committed_natives_except_path_names`, `live_build_through_the_hosted_engine`) need a prior `login` and are ignored by default.
- **Catalog size is pinned.** Some oracle tests assert today's catalog (59 sets, 1,161 icons, 3,472 actions); registering a set or adding icons (`library adopt --add`) means updating them with the new committed outputs.
- **Tests that read the built packages skip without them.** CI has no `packages/icons/dist`, so a test that reads it starts with `require!(packages_dist(), "packages/icons/dist")`; run `npm run build:packages` locally to run it in full.

## CI

- **`.github/workflows/rust.yml`** (every push and pull request): build, clippy with `-D warnings` and tests on macos-15 (arm64) and ubuntu-latest (x64), with Rust 1.93.1, `CXXFLAGS=-ffp-contract=off`, clang and libclang on Linux, Node 22 for the one side-by-side test that needs only node, and a Cargo cache. No oracle inputs are provided, so oracle tests skip. A second job checks the workspace with its `rust-version`.
- **`.github/workflows/cli-linux.yml`** (every push to main, or manual): builds the release CLI for x86_64-unknown-linux-gnu on ubuntu-22.04, runs `doctor`'s binary, cpu, quickjs and renderer checks, and keeps the binary with a `manifest.json` (commit, sha256, bytes, rustc) as the 90-day artifact `generative-icons-linux-x64`. The website's container image vendors it (`npm run vendor:cli` in the web repository). It publishes nothing.
- **`.github/workflows/release.yml`** (manual): checks that the npm package versions equal the workspace version; builds release binaries for aarch64/x86_64 apple-darwin (x86_64 cross-compiled on macos-15) and x86_64/aarch64 unknown-linux-gnu (ubuntu-22.04 and ubuntu-22.04-arm, for an older glibc); runs `doctor`'s binary, cpu, quickjs and renderer checks on each native build; drafts a GitHub release with tarballs, `.sha256` files and `SHA256SUMS`; fills `npm/platforms/*/bin` and packs every npm package as the `npm-packages` artifact. Run with `publish` checked, it then waits for a maintainer to approve its `npm` environment, publishes the packages through npm trusted publishing and publishes the run's draft release; otherwise it never publishes.
- **`.github/workflows/check.yml`** still runs the Node and Python checks, the package build and the consumer tests.

## Decisions

- **Parity first.** Every ported component must reproduce the committed outputs byte for byte where the old output was deterministic. The oracles:
  - `motion/icon-sets/*/scenes/*.scene.json` and `native/*.json` (gitignored; the snapshot above)
  - `prototype/icons/*/assets/*`
  - `motion/icon-sets/*/evidence/*`
- **The legacy pipeline stays until the oracles are committed.** The parity tests run the JS and Python tools side by side, so `motion/`, `scripts/` and the per-set scripts stay, marked legacy in the docs. The local-exporter plumbing is gone: `motion/src/export.mjs` now stops with a pointer to `library build`.
- **Authoring stays JavaScript.** Definitions run in QuickJS-NG 0.8.0 (`rquickjs =0.9.0`), with a vendored `rquickjs-sys` patched so `Math.*`, `Math.hypot` and `**` use V8's fdlibm (`ieee754.cc`, FMA contraction on) and a correctly rounded `pow`, plus time-limit, linear-time and module-lifetime patches (`vendor/rquickjs-sys/PATCHED.md`). Rounding stays in JS. Every output number is formatted by Rust (`ryu-js`), never by QuickJS. Keep `Cargo.lock` pinned: a floating `rquickjs-sys` would bypass the patch (it fails at link time).
- **Engine is hosted only.** The engine is Creator MCP, LottieFiles' hosted engine, at `https://creator.mcp.lottiefiles.com/mcp` (`DEFAULT_ENGINE_URL`); `--engine-url` or `GAI_ENGINE_URL` choose another, and a project's `engine.url` may name only a LottieFiles engine (`LOTTIEFILES_ENGINE_HOSTS`: Creator MCP's production host). Exports use a group-per-part script that clears documents with `creator.removeComp` (deleting composition nodes fails on reused documents). The client unwraps the identity group and sorts keys, which matches the old local exporter except each path's `nm`. Export file names carry the process id and a random suffix, so concurrent exports don't collide.
- **Credentials.** The refresh token lives in the OS keychain under an account keyed by the engine origin and the credentials home, or in a `0600` file; refreshes are serialized across processes with a file lock, which is never held across a keychain read (macOS may ask the user first); a wait for it is said after 3 s and ends with `SIGN_IN_BUSY` after 60 s. Agents never sign in: hints tell them to ask the user.
- **Renderer is the one users run.** dotlottie-rs at `e112d3f`, the commit dotlottie-web 0.80.0 was built from. The player reloads for every paint change, because a second slot flush after a render crashes natively.
- **Frozen sets.** 01 and 02 are never rebuilt (their assets are verified as they are). 03–06 build through built-in legacy profiles.
- **Formats kept for now.** `set.json` and dotLottie manifests stay in Python `json.dumps` format, and the `.lottie` ZIP bytes stay exact (classic zlib level 9, fixed dates). Changing a format is a deliberate, separate step.
- **One skill source.** The app skill is `generative-icons/assets/skill/`, embedded in the binary; the root `skills.sh.json` points the skills CLI (`npx skills add`) at it too. `packages/cli/skill/` is the retired `lottie-icons` skill, frozen byte for byte because `init --migrate` recognizes it by hash and the legacy package build copies it. It sits outside `skills/` so the skills CLI doesn't offer it.

## Known limits

- **Targets.** Parity was measured on aarch64-apple-darwin only. x86_64 and Linux are checked first by CI (the renderer probe and the math self-test run in the tests and in `doctor`). Windows is unsupported: CORE-MATH needs `__int128`, and the build passes GNU-style flags. Linux arm64 release builds need hosted arm64 runners, which a private repository must have enabled.
- **Sign-in.** The engine has no device-code flow or API keys: `login --no-browser` pastes the redirect, and `GAI_ENGINE_TOKEN` only carries a short-lived access token.
- **Legacy evidence.** Sets 01–20 wrote older browser-evidence schemas. The tests validate them on shared keys only, and running `verify-visual` on them would rewrite their evidence in the current schema. Motion measurement for sets 18–21 is not ported (their legacy scripts used other algorithms); their committed evidence carries forward.
- **Not ported.** `ai-icons.zip` (`package-ai.py`), `merge-render-evidence.py`, filmstrip PNGs next to natives, the gallery server and site build, the docs build and check, and an exact `lottie-icons` compat mode (legacy projects already get the old file bytes). The asset audit is ported (`gai_package::audit_assets`) but has no command yet, so `npm run check:assets` stays.
- **Outputs.** Stale outputs are never deleted (for example set 50's `do-not-disturb*` files), matching the old builders. Set zips still list the JS pipeline in `REBUILD.txt` and their source bundles, for parity with Python.
- **Native weights.** Players ignore the `style.outline.width` slot; `add --bake-weight` rewrites the file instead. When dotlottie-rs ships ThorVG 1.1.2 or newer with stroke-width slots, flip `STROKE_WIDTH_SLOT_SUPPORTED` and re-baseline the evidence, probe hashes and renderer pin together.
- **Contracts.** `schemas/project.schema.json` still describes only the original web config (weights 1.5, 2 and 2.5); the CLI accepts 1 to 3 and writes native configs.
- **Performance.** Preflight interprets in QuickJS: the largest set (22-teamwork) takes about 24 s against Node's 2 s. Whole-catalog verification peaks around 690 MB.
- **Engine limits.** A single slow BigInt operation (libbf) never checks the time limit; the watchdog ends the process at the deadline plus `Limits::grace`.
- **Internals.** gai-check has two rule interpreters (contracts and verify) whose small differences evidence depends on; merging them is deferred. serde_json runs without `arbitrary_precision`, so an integer beyond u64 in `set.json` would be rewritten as a float (none exist).

## Conventions

- Library crates use `thiserror`; the binary uses `anyhow`. No `unwrap` on user input.
- JSON output goes through `gai_core`. Don't hand-format numbers.
- Repo discovery: `GAI_LIBRARY_ROOT`, otherwise walk up from the working directory to the folder holding `motion/icon-sets`.
- Keep comments short and factual, as in the surrounding code.
