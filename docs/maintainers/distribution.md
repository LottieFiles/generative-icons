# Distribution

## What ships in the packages

| Package | Contents |
| --- | --- |
| `@lottiefiles/generative-icons` | One descriptor module per icon, native `.lottie` assets, `catalog.json`, `release.json`, schemas and bundled user docs |
| `@lottiefiles/generative-icons-runtime` | Runtime API and the shared controller |
| `@lottiefiles/generative-icons-react` | React adapter |
| `@lottiefiles/generative-icons-cli` | `lottie-icons` command and the consumer agent skill |

Each package's `files` list in its `package.json` is an explicit allowlist. `npm run build:packages` builds only from sets registered in `prototype/icons/catalog.json`.

## What stays out

- The Engine exporter and all authoring tooling (`motion/`, `scripts/`, set sources and review evidence).
- Verification pages, capture helpers and unregistered assets.
- The root and `motion/` npm packages, which are private development workspaces.

## Unreleased work

Drafts, plans and unreleased experiments live in `.local/`, which is ignored by Git and npm and sits outside the gallery server root. `npm run pack:release` refuses any tarball containing `.local/` or other unreleased paths, and `npm run test:distribution` checks the site build for the same.

A hidden link, robots rule or unlisted URL is not a release boundary. Moving work out of `.local/` into a public surface requires a maintainer decision and the normal review in [governance](governance.md).

## Legacy gallery build

`npm run build:site` stages the local gallery as a static site in `dist/site/` from an explicit file list. It refuses an existing output directory; pass `-- --output <new-dir>`. Never deploy `prototype/` directly. The public website is built in its own repository from the published packages.
