# Known limitations

## Creating icons needs a LottieFiles sign-in

`generative-icons build` exports native Lottie on LottieFiles' hosted engine, so it needs a sign-in (`generative-icons login`). The hosted engine is internal until launch, so building may not work for your account yet. Sign-in uses the browser, or a pasted redirect address with `--no-browser`; there is no API key or device code for unattended machines yet. Searching, installing, checking and previewing need no sign-in, and existing icons, the packages and the local gallery work without one.

## Before launch

The npm packages, the hosted library behind `search` and `add`, and the submission service behind `submit` go live at launch. Until then, build the CLI from the repository, build the packages in the checkout (`generative-icons library packages`) and pass `--library <checkout>`; `submit --dry-run` writes the bundle without sending it.

## Platforms

- The CLI ships for macOS (arm64, x64) and Linux with glibc (arm64, x64). Windows is not supported yet.
- x64 builds need a CPU with FMA, so they don't run under Rosetta; use the arm64 build on Apple silicon.
- Apps: web (npm, pnpm, Yarn or Bun), iOS (SwiftUI or UIKit) and Android (Compose or Views). React Native, Expo and Flutter apps are not supported.
- Native players ignore the `style.outline.width` token, so iOS and Android show the 1.5, 2 and 2.5 weights unless `add --bake-weight` bakes another weight into the file.

## Legacy set coverage

Heart/Bell (set 01) and Search (set 02) use legacy inline builders and are not covered by the separate-definition preflight. Their assets are verified as they are and never rebuilt. Some legacy review records refer to historical comparison evidence; missing evidence is not a pass.

The set archives (`<set-id>.zip`, in `source/REBUILD.txt`) and the per-set READMEs under `motion/icon-sets/` still give the retired local build steps: a locally installed exporter, the set's own build script and a `doctor` npm script. Those steps no longer work. Rebuild a set with `generative-icons library build` and check it with `generative-icons library verify`.

## Verification scope

The checks cover source geometry, native output, paint, rendering and packaging, but CI runs only part of them. On every push and pull request, CI builds and tests the CLI on macOS and Linux, and runs the Node and Python source, asset, paint, docs and package checks. Native verification (`library verify`), the rendered checks (`library verify-visual`) and the parity tests against the committed outputs are not part of CI: most of them need the scenes and native files that Git ignores, so the parity tests skip there. They run locally before a release. None of these checks replaces rendered review; see [verification](verification.md).

## Formats

Packages ship native dotLottie files and per-icon descriptors. Animated SVG, SMIL, PNG and other export formats are not yet available.
