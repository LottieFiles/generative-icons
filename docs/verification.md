# Verification

## Automated checks

Run from the repository root:

| Command | Checks | Needs exporter |
| --- | --- | --- |
| `npm test` | Controller and catalog behavior, generation harness, exporter setup, set tooling | No |
| `npm run check:source` | Source definitions across frames and profiles (legacy sets reported separately) | No |
| `node motion/icon-sets/library/preflight.mjs motion/icon-sets/SET-ID` | Source screening for one set | No |
| `npm run check:assets` | Unique catalog IDs, package hashes, ZIP integrity, loose JSON presence | No |
| `npm run check:paint` | Shared native paint contract | No |
| `npm run check:docs` | Relative links, publication hygiene, generated guide hashes | No |
| `npm run doctor` | Authoring dependencies and a real temporary native export | Yes |
| Set `verify.mjs` / `verify-clearance.mjs` | Set-specific geometry, state and clearance invariants on the native build | Yes |

CI runs the exporter-free checks plus the package build and consumer tests on every push and pull request.

**Automated checks don't replace visual review.** A passing test, a successful export or a nonempty screenshot says nothing about artistic quality. Every icon change needs the rendered review below.

## Rendered review

Start the verification server and open the affected set's renderer page:

```sh
python3 motion/icon-sets/library/director-revision/serve-verification.py
# open http://127.0.0.1:4191/icons/NN/verify.html
```

Use `--port 4194` if 4191 is taken. Newly scaffolded pages accept `?save=1` to collect evidence on any port; legacy pages use 4191. The server writes bounded evidence into the set and keeps each failed aggregate under a timestamped filename before a retry.

Review:

- 24, 32, 48 and 192 px, at measured canvas size
- Outline (all three weights), Solid and Duotone, in light and dark themes
- Initial, contact, settle and rest states
- Light and Strong side by side at normal speed and equal elapsed time, next to family neighbors
- Hover, focus, touch, loop, profile switches and downloads
- Reduced motion shows a useful static pose

Existing reports apply only to files with matching hashes. Do not hide findings with baseline waivers. See [known limitations](limitations.md).
