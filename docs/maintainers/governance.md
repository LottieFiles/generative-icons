# Governance

## Roles

| Role | Who | Can |
| --- | --- | --- |
| Contributor | Anyone | Open issues and pull requests, author icon source, run exporter-free checks |
| Maintainer | LottieFiles employees, or anyone with maintainer access to this repository | Run native export, review rendered output, approve and merge, tag, publish to npm, deploy the website |

Every change lands through a pull request with at least one maintainer approval. Maintainers do not approve their own release PRs when another maintainer is available.

## Icon contribution

1. Contributor opens a PR from the template with source changes and exporter-free checks.
2. CI is green.
3. A maintainer checks out the branch, builds the complete set natively and runs its `verify.mjs`:

   ```sh
   node motion/icon-sets/library/build-checked.mjs motion/icon-sets/SET-ID
   node motion/icon-sets/SET-ID/verify.mjs
   npm run icons:register -- SET-ID   # new sets only
   ```

4. The maintainer reviews rendered output following [verification](../verification.md):
   - [ ] Light and Strong at normal speed, beside family neighbors
   - [ ] Outline (all weights), Solid and Duotone, light and dark themes
   - [ ] 24, 32 and 48 px legibility
   - [ ] Reduced-motion pose and any reverse/reset action
   - [ ] Existing IDs and unrelated asset hashes unchanged
5. The maintainer pushes the native assets to the PR (or asks the contributor for changes), then approves and merges.

## Package release

1. Release PR bumps all four package versions and updates `CHANGELOG.md`.
2. A maintainer approves and merges.
3. A maintainer runs the preflight on the merged commit, tags `vX.Y.Z` and publishes.

Full steps: [releasing](releasing.md).

## Website release

1. PR in the website repository bumps the pinned package versions.
2. Review the preview deployment: gallery, search, icon detail pages, downloads and docs.
3. A maintainer approves and merges; the site deploys.

## Rollback

| Problem | Action |
| --- | --- |
| Bad merged PR | `git revert` the merge commit in a new PR |
| Bad npm release | Publish a fixed patch release. Meanwhile, point `latest` back to the last good version on each package (`npm dist-tag add @lottiefiles/<pkg>@<good> latest`) and `npm deprecate @lottiefiles/<pkg>@<bad> "<reason>"`. Avoid `npm unpublish`. |
| Bad website deploy | Redeploy the previous deployment, then revert the version bump PR |

Record every rollback in `CHANGELOG.md` under the next release.
