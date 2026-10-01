# Distribution

## What ships

| Package | Contents |
| --- | --- |
| `gen-animated-icons` | The npm wrapper: a Node shim that runs the binary from the matching platform package |
| `@lottiefiles/gen-animated-icons-<platform>` | The `gen-animated-icons` binary for one target: `darwin-arm64`, `darwin-x64`, `linux-arm64-gnu`, `linux-x64-gnu` |
| `@lottiefiles/generative-icons` | One descriptor module per icon, native `.lottie` assets, `catalog.json`, `release.json`, schemas and bundled user docs |
| `@lottiefiles/generative-icons-runtime` | Runtime API and the shared controller |
| `@lottiefiles/generative-icons-react` | React adapter |

The binary embeds the authoring kit, the agent skill (`crates/gen-animated-icons/assets/skill/`), the per-set rules and the export script. It contains no engine code: native export runs on LottieFiles' hosted engine. The GitHub release carries the same binaries as tarballs with SHA-256 checksums.

Each release tarball and npm platform package carries `LICENSE`, `THIRD_PARTY_NOTICES.md` and `THIRD_PARTY_LICENSES.txt`. The Release workflow generates the last one per target from `Cargo.lock`: the license texts of every crate the binary links statically and of the C and C++ code those crates compile.

Each npm package's `files` list in its `package.json` is an explicit allowlist. `gen-animated-icons library packages` builds the web packages only from sets registered in `prototype/icons/catalog.json`. `packages/cli`, the package of the retired `lottie-icons` command, is still built and packed with the web packages, but `gen-animated-icons` supersedes it and it is not published.

## What stays out

- Authoring tooling outside the binary (`motion/`, `scripts/`, set sources and review evidence).
- Verification pages, capture helpers and unregistered assets.
- The root and `motion/` npm packages, which are private development workspaces.

## Unreleased work

Drafts, plans and unreleased experiments live in `.local/`, which is ignored by Git and npm and sits outside the gallery server root. `gen-animated-icons library release` refuses any tarball containing `.local/` or other unreleased paths, and `npm run test:distribution` checks the site build for the same.

A hidden link, robots rule or unlisted URL is not a release boundary. Moving work out of `.local/` into a public surface requires a maintainer decision and the normal review in [governance](governance.md).

## Legacy gallery build

`npm run build:site` stages the local gallery as a static site in `dist/site/` from an explicit file list. It refuses an existing output directory; pass `-- --output <new-dir>`. Never deploy `prototype/` directly. The public website is built in its own repository from the published packages.
