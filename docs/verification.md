# Verification

## Icons in your app

`generative-icons check` runs offline: the source preflight for local icons, the installed descriptors, the config and lock, and the verification of each current build. `build` runs the same source checks before it exports, verifies the native result before it installs anything, and renders preview sheets. See [creating icons](creating-icons.md) for the review loop.

## The library

Run from the repository root:

| Command | Checks | Needs sign-in |
| --- | --- | --- |
| `generative-icons library preflight --all` | Source definitions across frames, profiles, directions and weights (legacy sets reported separately) | No |
| `generative-icons library verify SET-ID` | Set gates on the native build: provenance, format, paint and slots, holds, Light/Strong and reverse pairing, set-specific geometry and clearance | No |
| `generative-icons library verify --paint` | The shared native paint contract across the catalog | No |
| `generative-icons library verify-visual SET-ID` | Every action rendered with the players' dotLottie core at each size, style, weight and theme (the checks the old browser page ran), plus review sheets | No |
| `generative-icons library build SET-ID` | Preflight, hosted export and packaging | Yes |
| `cargo test --workspace` | The CLI and its parity with the committed outputs | No |
| `npm test` | Gallery controller, generation harness and set scaffold | No |
| `npm run check:assets` | Unique catalog IDs, package hashes, ZIP integrity, loose JSON presence | No |
| `npm run check:docs` | Relative links, publication hygiene, retired names, generated guide hashes | No |

CI runs the Rust build, lints and tests on macOS (arm64) and Linux (x64), and the Node and Python checks plus the package build and consumer tests, on every push and pull request. The legacy `npm run check:source` and `npm run check:paint` stay in CI as references for the CLI's parity.

**Automated checks don't replace visual review.** A passing test, a successful export or a nonempty preview says nothing about artistic quality. Every icon change needs the rendered review below.

## Rendered review

`generative-icons library verify-visual SET-ID` renders every action with the same dotLottie core the players use and writes `evidence/browser-verification.json` with rest, style and pose sheets. `generative-icons preview <id>` renders any icon's motions to contact sheets for a closer look (`--times` puts columns on chosen moments).

Review:

- 24, 32, 48 and 192 px
- Outline (all three weights), Solid and Duotone, in light and dark themes
- Initial, contact, settle and rest states
- Light and Strong side by side at equal elapsed time, next to family neighbors
- Hover, focus, touch, loop, profile switches and downloads in the local gallery (`npm run dev`)
- Reduced motion shows a useful static pose

Existing reports apply only to files with matching hashes. Do not hide findings with baseline waivers. See [known limitations](limitations.md).
