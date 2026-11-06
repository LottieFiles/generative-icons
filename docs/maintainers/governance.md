# Governance

## Roles

| Role | Who | Can |
| --- | --- | --- |
| Contributor | Anyone | Open issues and pull requests, author icon source, run the checks, and build with `generative-icons` once signed in |
| Maintainer | LottieFiles employees, or anyone with maintainer access to this repository | Review rendered output, review icon submissions, approve and merge, run the release workflow, tag, publish to npm, deploy the website |

Every change lands through a pull request with at least one maintainer approval. Maintainers do not approve their own release PRs when another maintainer is available.

## Icon contribution

Icons reach the library in two ways: a pull request that adds or revises a set in this repository, or a `generative-icons submit` from an app. Every one is reviewed. A submitted icon is built again from its spec, checked like the icons generated on the website and published in the community marked unreviewed; maintainers review it at generativeicons.com/review/, where they approve or hide it.

1. The contributor opens a PR from the template with the set's source, its build and its evidence, or a submission arrives.
2. CI is green.
3. A maintainer checks out the branch and confirms the build and its evidence:

   ```sh
   generative-icons library preflight SET-ID
   generative-icons library verify SET-ID
   generative-icons library verify-visual SET-ID
   generative-icons library register SET-ID   # new sets only
   ```

   If the PR carries no build yet, build it first with `generative-icons library build SET-ID`.

4. The maintainer reviews rendered output following [verification](../verification.md):
   - [ ] Light and Strong at normal speed, beside family neighbors
   - [ ] Outline (all weights), Solid and Duotone, light and dark themes
   - [ ] 24, 32 and 48 px legibility
   - [ ] Reduced-motion pose and any reverse/reset action
   - [ ] Existing IDs and unrelated asset hashes unchanged
5. The maintainer pushes any rebuilt assets to the PR (or asks the contributor for changes), then approves and merges.

## Package release

1. Release PR bumps the package, CLI and npm wrapper versions and updates `CHANGELOG.md`.
2. A maintainer approves and merges.
3. A maintainer runs the preflight, then the Release workflow on the merged commit with **Publish** checked, reviews the draft release and the packed packages, and approves the workflow's `npm` environment. The workflow publishes to npm and publishes the draft release, which tags `vX.Y.Z`.

Full steps: [releasing](releasing.md).

## Website release

The website repository is internal: only maintainers can see it. The site is built from the published packages, so changes to icons, guides and packages are made in this repository, and website problems are reported here as issues.

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
