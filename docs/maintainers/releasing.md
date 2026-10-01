# Releasing

Runbook for publishing the `gen-animated-icons` CLI and the web packages `@lottiefiles/generative-icons`, `-runtime` and `-react`. They share one version. Approvals are described in [governance](governance.md). Nothing is published without an explicit go from the maintainers; no workflow publishes to npm.

## 1. Open a release PR

1. Branch from an up-to-date `main`: `git switch -c release/vX.Y.Z`.
2. Bump every version:

   ```sh
   npm version X.Y.Z --workspaces --no-git-tag-version          # packages/*
   # Cargo.toml [workspace.package] version = "X.Y.Z"
   # npm/gen-animated-icons/package.json: version and every optionalDependencies pin
   # npm/platforms/*/package.json: version
   grep -rn '"@lottiefiles/gen' packages/*/package.json npm/*/package.json npm/platforms/*/package.json
   ```

3. In [CHANGELOG.md](https://github.com/LottieFiles/lottie-generative-icons/blob/main/CHANGELOG.md), move the `Unreleased` entries under `## [X.Y.Z] - YYYY-MM-DD`.
4. Open the PR. A maintainer approves and merges it once CI is green.

## 2. Preflight on the merged commit

Start from a clean checkout of the merged `main`, with the parity oracles unpacked (see [crates/README.md](https://github.com/LottieFiles/lottie-generative-icons/blob/main/crates/README.md#tests)):

```sh
git switch main && git pull
git status --porcelain            # must print nothing
npm ci && npm ci --prefix motion
npm run build:packages            # the reference dist the parity tests compare with

cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --release -- --include-ignored --skip live_
npm test
npm run check:docs
npm run check:assets
npm run check:source
npm run check:paint
gen-animated-icons library packages
npm run test:packages
gen-animated-icons library release --output dist/packages-vX.Y.Z
npm run test:consumers -- dist/packages-vX.Y.Z
```

Every command must pass. Fix failures in a PR; never weaken a check to pass.

Then confirm:

- `packages/icons/dist/release.json` has `"sourceDirty": false` and `sourceCommit` equals `git rev-parse HEAD`. A dirty tree means the build does not match the tag; clean up and rebuild.
- `dist/packages-vX.Y.Z/release.json` lists every tarball at version X.Y.Z. The pack also contains `packages/cli`, the retired `lottie-icons` command that `gen-animated-icons` supersedes; leave it unpublished unless the maintainers decide otherwise.
- Tarball contents (`tar -tzf <file>.tgz`) contain nothing from `.local/` or other unreleased work (see [distribution](distribution.md)).
- License and [third-party notices](https://github.com/LottieFiles/lottie-generative-icons/blob/main/THIRD_PARTY_NOTICES.md) cover the shipped files. For the CLI binaries, the Release workflow adds the generated `THIRD_PARTY_LICENSES.txt` (step 3).

## 3. Build the CLI release

Run the **Release** workflow (Actions → Release → Run workflow) on the merged commit. It:

- generates `THIRD_PARTY_LICENSES.txt` for each target with [cargo-about](https://github.com/EmbarkStudios/cargo-about): every crate the binary links, and the license texts of the crates and of the C and C++ code they compile (QuickJS-NG, V8's `ieee754.cc` with fdlibm, CORE-MATH, ThorVG with RapidJSON and JerryScript), plus the embedded svgpath and DejaVu Sans Mono;
- builds the binary for `aarch64-apple-darwin`, `x86_64-apple-darwin`, `aarch64-unknown-linux-gnu` and `x86_64-unknown-linux-gnu`, and checks each native build with `doctor`;
- creates a **draft** GitHub release for tag `vX.Y.Z` with one tarball per target (the binary, `LICENSE`, `THIRD_PARTY_NOTICES.md` and `THIRD_PARTY_LICENSES.txt`), a `.sha256` file for each and `SHA256SUMS`;
- copies each binary and its three license files into its `npm/platforms/*` package, packs the wrapper and the four platform packages with `npm pack`, and uploads the tarballs as the `npm-packages` workflow artifact.

The license step pins the license files of the workspace crates, the vendored `rquickjs-sys` and the pinned `dotlottie-rs` by SHA-256 in the workflow's inline `about.toml`. If it stops because a checksum no longer matches, a license file changed (for example after a renderer bump): review the new text, then update the checksum in `.github/workflows/release.yml`.

Download the artifact and check a platform tarball (`tar -tvzf`): `package/bin/gen-animated-icons` must be executable, and `package/LICENSE`, `package/THIRD_PARTY_NOTICES.md` and `package/THIRD_PARTY_LICENSES.txt` must be present. The workflow checks both, but look before you publish.

## 4. Publish

1. Review the draft release, then publish it. Publishing creates the tag `vX.Y.Z`.
2. Publish the npm packages from the reviewed tarballs: the platform packages first, then the wrapper, then the web packages in dependency order. `--provenance` requires publishing from GitHub Actions with `id-token: write`.

   ```sh
   npm publish lottiefiles-gen-animated-icons-darwin-arm64-X.Y.Z.tgz --access public
   npm publish lottiefiles-gen-animated-icons-darwin-x64-X.Y.Z.tgz --access public
   npm publish lottiefiles-gen-animated-icons-linux-arm64-gnu-X.Y.Z.tgz --access public
   npm publish lottiefiles-gen-animated-icons-linux-x64-gnu-X.Y.Z.tgz --access public
   npm publish gen-animated-icons-X.Y.Z.tgz --access public

   cd dist/packages-vX.Y.Z
   npm publish lottiefiles-generative-icons-runtime-X.Y.Z.tgz --access public
   npm publish lottiefiles-generative-icons-X.Y.Z.tgz         --access public
   npm publish lottiefiles-generative-icons-react-X.Y.Z.tgz   --access public
   ```

   For a prerelease (`X.Y.Z-next.N`), add `--tag next` to every command so `latest` is untouched.

3. Verify: `npm view gen-animated-icons@X.Y.Z version`, then in a scratch app run `npx gen-animated-icons@X.Y.Z init` and `add heart`.

## 5. Update the website

In the website repository, open a PR that bumps the pinned package versions to the new registry versions. Review the preview deployment, get maintainer approval, merge and redeploy.

## Rollback

See [governance](governance.md#rollback).
