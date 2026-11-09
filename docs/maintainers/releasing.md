# Releasing

Runbook for publishing the `generative-icons` CLI and the web packages `@lottiefiles/generative-icons`, `-runtime` and `-react`. They share one version. Approvals are described in [governance](governance.md). Nothing is published without an explicit go from a maintainer: only the Release workflow publishes, when it is run with **Publish** checked, and only after a maintainer approves its `npm` environment.

## 1. Open a release PR

1. Branch from an up-to-date `main`: `git switch -c release/vX.Y.Z`.
2. Bump every version:

   ```sh
   npm version X.Y.Z --workspaces --no-git-tag-version          # packages/*
   # Cargo.toml [workspace.package] version = "X.Y.Z"
   # npm/generative-icons/package.json: version and every optionalDependencies pin
   # npm/platforms/*/package.json: version
   grep -rn '"@lottiefiles/gen' packages/*/package.json npm/*/package.json npm/platforms/*/package.json
   ```

3. In [CHANGELOG.md](https://github.com/LottieFiles/generative-icons/blob/main/CHANGELOG.md), move the `Unreleased` entries under `## [X.Y.Z] - YYYY-MM-DD`.
4. Open the PR. A maintainer approves and merges it once CI is green.

## 2. Preflight on the merged commit

Start from a clean checkout of the merged `main`, with the parity oracles unpacked (see [crates/README.md](https://github.com/LottieFiles/generative-icons/blob/main/crates/README.md#tests)):

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
generative-icons library packages
npm run test:packages
generative-icons library release --output dist/packages-vX.Y.Z
npm run test:consumers -- dist/packages-vX.Y.Z
```

Every command must pass. Fix failures in a PR; never weaken a check to pass.

Then confirm:

- `packages/icons/dist/release.json` has `"sourceDirty": false` and `sourceCommit` equals `git rev-parse HEAD`. A dirty tree means the build does not match the tag; clean up and rebuild.
- `dist/packages-vX.Y.Z/release.json` lists every tarball at version X.Y.Z.
- Tarball contents (`tar -tzf <file>.tgz`) contain nothing from `.local/` or other unreleased work (see [distribution](distribution.md)).
- License and [third-party notices](https://github.com/LottieFiles/generative-icons/blob/main/THIRD_PARTY_NOTICES.md) cover the shipped files. For the CLI binaries, the Release workflow adds the generated `THIRD_PARTY_LICENSES.txt` (step 3).

## 3. Build the CLI release

Run the **Release** workflow (Actions → Release → Run workflow) on the merged commit. Check **Publish** to publish from this run once you approve it (step 4); leave it unchecked to only build. It:

- generates `THIRD_PARTY_LICENSES.txt` for each target with [cargo-about](https://github.com/EmbarkStudios/cargo-about): every crate the binary links, and the license texts of the crates and of the C and C++ code they compile (QuickJS-NG, V8's `ieee754.cc` with fdlibm, CORE-MATH, ThorVG with RapidJSON and JerryScript), plus the embedded svgpath and DejaVu Sans Mono;
- builds the binary for `aarch64-apple-darwin`, `x86_64-apple-darwin`, `aarch64-unknown-linux-gnu` and `x86_64-unknown-linux-gnu`, and checks each native build with `doctor`;
- creates a **draft** GitHub release for tag `vX.Y.Z` with one tarball per target (the binary, `LICENSE`, `THIRD_PARTY_NOTICES.md` and `THIRD_PARTY_LICENSES.txt`), a `.sha256` file for each and `SHA256SUMS`;
- copies each binary and its three license files into its `npm/platforms/*` package, packs the wrapper and the four platform packages with `npm pack`, and uploads the tarballs as the `npm-packages` workflow artifact;
- builds the web packages from the same commit (`npm run build:packages`, `npm run test:packages`), packs them with `npm run pack:release`, checks that `release.json` says a clean release of that commit at X.Y.Z, installs them in fresh Vite and Next apps (`npm run test:consumers`), and uploads the tarballs and `release.json` as the `web-packages` workflow artifact.

The license step pins the license files of the workspace crates, the vendored `rquickjs-sys` and the pinned `dotlottie-rs` by SHA-256 in the workflow's inline `about.toml`. If it stops because a checksum no longer matches, a license file changed (for example after a renderer bump): review the new text, then update the checksum in `.github/workflows/release.yml`.

Download both artifacts (`gh run download <run-id> -n npm-packages -n web-packages`) and check a platform tarball (`tar -tvzf`): `package/bin/generative-icons` must be executable, and `package/LICENSE`, `package/THIRD_PARTY_NOTICES.md` and `package/THIRD_PARTY_LICENSES.txt` must be present. The workflow checks both, but look before you publish.

## 4. Publish

**Publish by hand for now.** Trusted publishing from the workflow doesn't work for this repository yet. GitHub gives repositories created after 15 July 2026 an immutable OIDC subject (ours is `repo:LottieFiles@30947444/generative-icons@1375198936`), and npm's token exchange rejects it with "OIDC token exchange error - package not found" ([npm/cli#9969](https://github.com/npm/cli/issues/9969)). npm then falls back to setup-node's placeholder token, so the publish job fails with `E404` and publishes nothing. 0.1.1 was published by hand from the reviewed artifacts. Until npm fixes the bug, run the Release workflow without **Publish**, publish its artifacts by hand ([below](#publishing-by-hand)), then publish its draft release (`gh release edit vX.Y.Z --draft=false --latest`).

1. If the run has **Publish** checked, its publish job waits once the build, draft and pack jobs pass. Review the draft release and the artifacts (step 3). Then approve the job: open the run, choose **Review deployments**, select `npm` and approve. The job:
   - publishes the eight packages to npm in order: the platform packages first (the wrapper pins them exactly), then the wrapper, then the web packages in dependency order. It skips a version already on npm, so a failed run can run again. A prerelease (`X.Y.Z-next.N`) goes to the `next` tag, so `latest` is untouched;
   - publishes the draft release this run made, which creates the tag `vX.Y.Z`.

   It needs no token: each package trusts this workflow in the `npm` environment (see [Trusted publishing](#trusted-publishing)), and npm adds a provenance attestation to every version. If the run went without **Publish**, run it again with it checked: it rebuilds the same commit and drafts a new release. Delete the earlier draft, which stays unpublished.
2. Verify: `npm view generative-icons@X.Y.Z version` and `npm view @lottiefiles/generative-icons-react@X.Y.Z version $R` (with `R` as below), then in a scratch app run `npx generative-icons@X.Y.Z init` and `add heart` (without `--library`, so it reads the hosted library). Each package page on npmjs.com shows its provenance.

### Publishing by hand

When the workflow can't publish (a new package npm doesn't know yet, or a trusted publisher that changed), a maintainer in the `lottiefiles` npm org publishes the reviewed tarballs of both artifacts in the same order.

Many LottieFiles machines send the `@lottiefiles` scope to GitHub Packages (`@lottiefiles:registry=https://npm.pkg.github.com` in `~/.npmrc`). That setting wins over `publishConfig.registry` and over `--registry`, so every command names the scope's registry itself. Check the first one with `--dry-run`: it must say `Publishing to https://registry.npmjs.org/`.

```sh
R=--@lottiefiles:registry=https://registry.npmjs.org/
npm publish lottiefiles-generative-icons-darwin-arm64-X.Y.Z.tgz --access public $R
npm publish lottiefiles-generative-icons-darwin-x64-X.Y.Z.tgz --access public $R
npm publish lottiefiles-generative-icons-linux-arm64-gnu-X.Y.Z.tgz --access public $R
npm publish lottiefiles-generative-icons-linux-x64-gnu-X.Y.Z.tgz --access public $R
npm publish generative-icons-X.Y.Z.tgz --access public $R

npm publish lottiefiles-generative-icons-runtime-X.Y.Z.tgz --access public $R
npm publish lottiefiles-generative-icons-X.Y.Z.tgz         --access public $R
npm publish lottiefiles-generative-icons-react-X.Y.Z.tgz   --access public $R
```

Then publish the run's draft release on GitHub: its address is in the log of the run's draft job. Delete any older draft with the same tag, so `vX.Y.Z` names one release.

For a prerelease (`X.Y.Z-next.N`), add `--tag next` to every `npm publish` so `latest` is untouched.

### Trusted publishing

Each of the eight packages trusts one publisher on npm: GitHub Actions, repository `LottieFiles/generative-icons`, workflow `release.yml`, environment `npm`. The repository's `npm` environment asks a maintainer to approve each deployment and deploys only from `main`. A package new to the release is published by hand once (above), then trusted. Setting up trust needs write access to the package and 2FA on the npm account, and the first command asks for 2FA. `npm trust` refuses the `--@lottiefiles:registry` flag, so the scope's registry comes from the environment instead:

```sh
for package in generative-icons @lottiefiles/generative-icons @lottiefiles/generative-icons-runtime @lottiefiles/generative-icons-react \
    @lottiefiles/generative-icons-darwin-arm64 @lottiefiles/generative-icons-darwin-x64 \
    @lottiefiles/generative-icons-linux-arm64-gnu @lottiefiles/generative-icons-linux-x64-gnu; do
  env 'npm_config_@lottiefiles:registry=https://registry.npmjs.org/' \
    npm trust github "$package" --repo LottieFiles/generative-icons --file release.yml --env npm --allow-publish --yes
  sleep 2
done
```

`env 'npm_config_@lottiefiles:registry=https://registry.npmjs.org/' npm trust list <package>` shows a package's trusted publisher. If a run fails with "unable to authenticate", compare the trusted publisher with the workflow's file name and environment: they match case-sensitively. If the job's "Show what npm said about trusted publishing" step reports "package not found" while the trusted publisher matches, it's npm/cli#9969 (above).

## 5. Update the website

In the website repository, open a PR that bumps the pinned package versions to the new registry versions. Review the preview deployment, get maintainer approval, merge and redeploy.

## Rollback

See [governance](governance.md#rollback).
