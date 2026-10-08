# The generative-icons CLI

`generative-icons` is one binary that finds, installs, creates, checks and previews Generative Icons in web, iOS and Android apps. It also serves the same flow to coding agents over MCP, and carries the maintainer commands for this repository.

Run `generative-icons --help` or `generative-icons <command> --help` for the exact options of the version you have.

## Install

```sh
npx generative-icons --version
```

The `generative-icons` npm package is a small Node shim. npm installs the matching binary package for your machine as an optional dependency, and nothing runs at install time. Prebuilt binaries cover macOS (arm64, x64) and Linux with glibc (arm64, x64); Windows is not supported yet. Homebrew and a curl installer are planned. To install it in a project, add it as a dev dependency and run it with your package manager (`npx`, `pnpm exec`, `yarn`).

Before the first release, build the binary from this repository ([development](development.md#build-the-cli)) and run it directly, or point the shim at it with `GEN_ANIMATED_ICONS_BINARY`.

## Conventions

- Run commands in the app folder, or pass `--cwd <app-folder>` (monorepos). `--platform web|ios|android` picks one when a folder holds several apps.
- Output is JSON when stdout is not a terminal, or with `--json`; `--format text` forces text.
- Errors go to stderr. In JSON mode an error is one line: `{"error": {"code", "message", "path"?, "id"?, "hint"?}}`. Warnings and `--verbose` notes also go to stderr; `--quiet` hides warnings.
- Exit codes: `0` success, `1` a check failed, `2` usage, `3` project or file conflict, `4` sign-in, `5` network or engine, `6` integrity, `70` internal.
- `--offline` uses only local and cached data and never the network. `build`, `login` and `logout` refuse to run offline (`NETWORK`). `whoami` and `info` report the stored session as not verified.

## Commands for apps

| Command | What it does |
| --- | --- |
| `init` | Detects the app, writes `generative-icons.json`, installs the agent skill (`--agents claude,agents` or `none`), copies the authoring kit and starts the lock. `--migrate` moves a project set up as `gen-animated-icons` (this CLI's earlier name: `gen-animated-icons.json`, `.gen-animated-icons/`) or by the earlier `lottie-icons` npm command; `--update-kit` replaces an edited kit copy. |
| `info` | Platform, config, library version, managed files, local icons and their build state, and the sign-in |
| `search <meaning…>` | Ranked library matches (`--limit`, `--local-only`, `--library-only`, `--match ranked\|all-words`). Common UI synonyms match too ("delete" finds `trash`), and weak description-only matches are dropped. |
| `add <id…>` | Installs each icon's `.lottie` asset and descriptor. `--installed` re-adds every installed library icon (upgrade), `--dry-run` shows the changes, `--bake-weight <w>` bakes an outline weight from 1 to 3 into native files. |
| `diff` | Lists managed files and whether they were edited |
| `remove <id…>` | Uninstalls icons whose files are unedited |
| `doctor` | Checks the binary and CPU, project, skills, kit, lock, library, sign-in, engine and renderer |
| `context [<query…>]` | Prints the reference packet an AI reads before drawing: the family contract and the nearest library icons' geometry and timing. `--neighbor <id>` (up to four) chooses references, `--search` lists candidates, `--prompt` prints a design prompt, `--output <dir>` saves the packet. `--format text` prints a summary; use JSON for the geometry. |
| `create <id>` | Scaffolds a local icon as a spec, `generative-icons/icons/<id>.icon.json`: plain JSON shapes, roles, rigs and eased keys (`--label`, `--intent`), and saves a context packet (`--neighbor`). `--from <library-id>` starts from a library icon reanimated as a spec (`search` marks them `"spec": true`); the icons package ships those specs under `specs/`, so any library works. Ids are kebab-case and can't be names Windows reserves (`con`, `aux`, `nul`, `com1`…). `--batch` writes a v1 JS batch instead, `generative-icons/icons/batches/<id>.mjs`, for what specs can't express yet; with it, `--from` starts from a library icon's set source and refuses icons that have a spec. Maintainers start a reanimation with `create <library-id> --replace`: the icon's own spec, or its v1 drawing ported to a spec, under its library id for `library adopt`. |
| `check [<id>…]` | Offline checks: source preflight, installed descriptors, config and lock, and (once built) the build's verification. `STALE` means the source changed since the last build, so rebuild before previewing or submitting. `UNTRACED_IMPORTS` warns about imports `submit` can't follow (use plain string specifiers). A spec v2 icon is validated, compiled and linted instead, with JSON Pointers into the spec; with no ids, every spec is. |
| `draft <id>` | Spec icons only: one hosted-engine call builds every motion and renders a labelled review sheet (`.generative-icons/drafts/<id>.png` and `<id>-24px.png`). Nothing is exported or stored. A spec with errors stops before the engine. |
| `build [<id>…]` | Exports on the hosted engine, then packages, verifies, renders preview sheets and installs (`--no-install`, `--jobs 1-4`, `--all`). A spec v2 icon takes one engine call for all its motions. v1 and v2 icons build in separate runs. |
| `preview <id>` | Renders a built, installed or library icon to PNG contact sheets (`--motion`, `--animation`, `--times`, `--frames`, `--styles`, `--weights`, `--themes`, `--cell`, `--out`) |
| `export <id>` | Writes a built, installed or library icon as an animated SVG that browsers play without a player (SMIL): `--motion`, `--style`, `--weight`, `--color` (`currentColor` unless you pass a hex value), `--trigger auto\|hover\|click\|toggle\|loop\|load\|js` (`auto`: `toggle` for stateful icons, `loop` for loaders, `hover` otherwise), `--state` (a stateful icon's first state), `--size`, `--id-prefix`, and `--out <file>` (without it the SVG is printed, or returned as `svg` in JSON). `hover`, `click`, `toggle` and `js` need an inline `<svg>`; an `<img>` plays only `loop` and `load`. See [Animated SVG](using-packages.md#animated-svg). |
| `submit <id…>` | Sends verified local icons for review (`--license MIT\|CC-BY-4.0`, `--message`). The bundle holds each icon's verified build snapshot, only the modules it imports, and its two review sheets (regenerated if they are missing or edited). Icons with untraceable imports are refused unless you pass `--include-untraced`. A spec v2 icon's bundle holds its spec and its verified build. `--dry-run --output bundle.zip` only writes the bundle. |
| `login`, `logout`, `whoami` | Signs in to the hosted engine, signs out, and shows the account and whether the session is verified |
| `mcp` | Serves `search`, `add`, `context`, `create`, `check`, `draft`, `build`, `preview` and `export` as MCP tools over stdio |

## Sign-in and the hosted engine

`build` exports native Lottie on LottieFiles' hosted engine, so it needs a LottieFiles sign-in. The hosted engine is internal until launch, so building may not work for your account yet. Every other app command works without signing in.

- `generative-icons login` opens the browser. `--no-browser` prints the sign-in address and reads the redirect address from stdin; that mode is automatic in CI and over SSH.
- The refresh token is kept in the OS keychain, or in a `0600` credentials file where no keychain is available. The credentials folder is the OS config folder's `generative-icons` (override with `GEN_ANIMATED_ICONS_HOME`).
- `build` sends the icon's sampled shapes and motion to the engine. Your app's code stays on your machine.
- Commands share a sign-in lock in that folder while they refresh the session. When another `generative-icons` process holds it (it is signing in, or waiting on a macOS Keychain prompt), a command says so after 3 s and stops after 60 s with `SIGN_IN_BUSY` (exit 4): approve the prompt, or end that process. `--verbose` says when a command reads the keychain.
- Coding agents never sign in themselves: on `NOT_LOGGED_IN` or `SESSION_EXPIRED` they ask you to run `login`.

## Files in your app

| Path | What it is | Commit it |
| --- | --- | --- |
| `generative-icons.json` | Platform, folders, default appearance, skill folders, local icon settings | Yes |
| Assets folder (web: `public/generative-icons/`, iOS: `<Target>/GenerativeIcons/`, Android: `app/src/main/assets/`) | Installed `.lottie` files | Yes |
| Descriptors folder (web: `src/generative-icons/` in an app with `src/`, native: `generative-icons/descriptors/`) | One descriptor per icon, plus `settings.*` on the web | Yes |
| `.generative-icons/lock.json` | Installed icons and the hash of every managed file | Yes |
| `.generative-icons/` (other files) | Build intermediates, previews and context packets; ignored by its own `.gitignore` | No |
| `generative-icons/library/` | Read-only copy of the authoring kit and family contract | Yes |
| `generative-icons/icons/` | Your local icons' source: `definitions.mjs`, `batches/`, `set-config.json`, `evidence/` | Yes |
| `.claude/skills/generative-icons/`, `.agents/skills/generative-icons/` | The agent skill | Yes |

Managed files are never overwritten once edited: commands stop with `MODIFIED_FILE`, and `diff` shows what changed. `init` never edits `AGENTS.md`, `CLAUDE.md` or MCP settings; it prints the MCP entry for you to add if you want it, plus `mcpLocal`, an entry that runs this binary by its absolute path (useful before the npm package is published).

## Where icons come from

The library is chosen in this order:

1. `--library <dir|url>` or `GEN_ANIMATED_ICONS_LIBRARY`: a package `dist` folder, a checkout of this repository (it reads `packages/icons/dist`), or an https URL.
2. A checkout of this repository, or `node_modules/@lottiefiles/generative-icons/dist`, in the app folder or above it.
3. The hosted library at `generativeicons.lottiefiles.com`, cached for `--offline` use.

The hosted library and the submission service go live at launch. Until then, pass `--library` with a checkout whose packages are built.

## Agents with MCP

`generative-icons mcp` runs an MCP server on stdio. Each tool call runs as its own process, arguments must match each tool's input schema, and a tool's `cwd` must stay inside the server's folder. `submit` and `login` are not exposed. To add it to an agent:

```json
{"mcpServers": {"generative-icons": {"command": "npx", "args": ["generative-icons", "mcp"]}}}
```

## Environment variables

| Variable | Meaning |
| --- | --- |
| `GEN_ANIMATED_ICONS_LIBRARY` | Same as `--library` |
| `GAI_LIBRARY_ROOT` | Same as `--library-root`: the checkout for maintainer commands |
| `GEN_ANIMATED_ICONS_ENGINE_URL` | Same as `--engine-url`. Otherwise `GAI_ENGINE_URL`, then the config's `engine.url` (LottieFiles engines only), then the default engine. |
| `GAI_ENGINE_TOKEN` | A short-lived access token used instead of the stored sign-in, for automation. It is sent only to the `GAI_ENGINE_URL` engine or a LottieFiles engine. |
| `GEN_ANIMATED_ICONS_HOME` | Folder for the credentials file and refresh lock |
| `GEN_ANIMATED_ICONS_CACHE_DIR` | Download cache (default: the OS cache folder's `generative-icons`) |
| `GEN_ANIMATED_ICONS_SUBMIT_URL` | Same as `submit --endpoint` |
| `GEN_ANIMATED_ICONS_BINARY` | Makes the npm shim run this binary instead |
| `GEN_ANIMATED_ICONS_V2_DOCUMENTS` | Prefix of the engine documents spec v2 drafts and builds use (default `gen-animated-icons-v2`) |

## Maintainer commands

`generative-icons library <command>` works on a checkout of this repository (found from the working folder, or `--library-root`):

| Command | What it does |
| --- | --- |
| `new-set <slug>` | Scaffolds an empty source set with the next free number |
| `preflight [<set>] [--all]` | Screens set sources before export |
| `build [<set>…] [--all]` | Preflights, exports on the hosted engine and packages sets (`--reuse-native` rebuilds from `native/`, `--offline-scenes` writes scenes only) |
| `adopt <set> <build>…` | Replaces library icons with spec v2 builds (`.generative-icons/build/<id>`): spec, assets and `set.json` entry, then repackages the set. `--add` adds new icons to the set instead, numbered after the library's last icon, in the set's category |
| `verify [<set>…] [--all] [--paint]` | Runs the set gates and the paint audit, and writes their evidence |
| `verify-visual <set>` | Runs the renderer checks and writes `browser-verification.json` and review sheets |
| `measure [<set>…] [--all]` | Writes `evidence/motion-measurements.json` (sets 22–39) |
| `package [<set>…] [--icon-library]` | Packages sets, or builds `icon-library.zip` and its `delivery.json` |
| `packages [--output <dir>]` | Builds the npm packages' `dist` folders and the search sidecar |
| `register <set>` | Adds a built, verified set to the catalog |
| `release [--output <dir>]` | Builds the packages and packs the release tarballs; it never publishes |

The maintainer workflow is in [development](development.md) and [verification](verification.md).
