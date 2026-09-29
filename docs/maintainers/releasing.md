# Releasing packages

Runbook for publishing `@lottiefiles/generative-icons`, `-runtime`, `-react` and `-cli`. All four share one version. Approvals are described in [governance](governance.md).

## 1. Open a release PR

1. Branch from an up-to-date `main`: `git switch -c release/vX.Y.Z`.
2. Bump the version in all four `packages/*/package.json` files, including the pinned internal `@lottiefiles/generative-icons*` dependencies:

   ```sh
   npm version X.Y.Z --workspaces --no-git-tag-version
   grep -n '"@lottiefiles/generative-icons' packages/*/package.json   # every pin must read X.Y.Z
   ```

3. In [CHANGELOG.md](https://github.com/LottieFiles/lottie-generative-icons/blob/main/CHANGELOG.md), move the `Unreleased` entries under `## [X.Y.Z] - YYYY-MM-DD`.
4. Open the PR. A maintainer approves and merges it once CI is green.

## 2. Preflight on the merged commit

Start from a clean checkout of the merged `main`:

```sh
git switch main && git pull
git status --porcelain            # must print nothing
npm ci && npm run setup

npm test
npm run check:docs
npm run check:assets
npm run check:paint
npm run check:source
npm run build:packages
npm run test:packages
npm run pack:release -- dist/packages-vX.Y.Z
npm run test:consumers
```

Every command must pass. Fix failures in a PR; never weaken a check to pass.

Then confirm:

- `packages/icons/dist/release.json` has `"sourceDirty": false` and `sourceCommit` equals `git rev-parse HEAD`. A dirty tree means the build does not match the tag; clean up and rebuild.
- `dist/packages-vX.Y.Z/release.json` lists all four tarballs at version X.Y.Z.
- Tarball contents (`tar -tzf <file>.tgz`) contain nothing from `.local/` or other unreleased work (see [distribution](distribution.md)).
- License and [third-party notices](https://github.com/LottieFiles/lottie-generative-icons/blob/main/THIRD_PARTY_NOTICES.md) cover the shipped files.

## 3. Tag

```sh
git tag -a vX.Y.Z -m "vX.Y.Z"
git push origin vX.Y.Z
```

Tags use the format `vX.Y.Z` (for example `v0.1.0`, or `v0.2.0-next.1` for a prerelease).

## 4. Publish to npm

Publish the exact tarballs that passed `test:consumers`, in dependency order. `--provenance` requires publishing from GitHub Actions with `id-token: write`.

```sh
cd dist/packages-vX.Y.Z
npm publish lottiefiles-generative-icons-runtime-X.Y.Z.tgz --access public --provenance
npm publish lottiefiles-generative-icons-X.Y.Z.tgz         --access public --provenance
npm publish lottiefiles-generative-icons-react-X.Y.Z.tgz   --access public --provenance
npm publish lottiefiles-generative-icons-cli-X.Y.Z.tgz     --access public --provenance
```

For a prerelease (`X.Y.Z-next.N`), add `--tag next` to every command so `latest` is untouched.

Verify: `npm view @lottiefiles/generative-icons@X.Y.Z version` for each package, then install into a scratch app and run `npx lottie-icons add heart`.

## 5. GitHub Release

Create a release from tag `vX.Y.Z` with the CHANGELOG section as its notes. Mark prereleases as such.

## 6. Update the website

In the website repository, open a PR that bumps the pinned package versions from the vendored tarballs (or the previous registry version) to the new registry versions. Review the preview deployment, get maintainer approval, merge and redeploy.

## Rollback

See [governance](governance.md#rollback).
