# Shared icon library

Shared authoring code for every source set: the family contract and the authoring kit, which the `generative-icons` CLI embeds. The build, verification and packaging scripts here are the legacy pipeline; the CLI reproduces their outputs and its parity tests still run them. Current counts come from `prototype/icons/catalog.json` and its set manifests.

- Gallery: `prototype/icons/01/index.html`. Run `npm run dev` from the repository root and open `/icons/01/`.
- New icon work: follow the [animated icon workflow](../../../.agents/skills/animated-icon-workflow/SKILL.md).
- Generation context and preflight: see the [generation harness](../../../.agents/skills/animated-icon-workflow/references/generation-harness.md). Run `npm --prefix motion run test:harness` for its regression tests.

## Key files

| File | Purpose |
| --- | --- |
| `family-contract.json` | Shared numeric contract: grid, stroke weights, Duotone tint, timeline |
| `drawing.mjs`, `direction.mjs` | Contour helpers; timing and result/reset helpers |
| `generation-context.mjs` | Search and reference packets for a design brief |
| `preflight.mjs`, `build-checked.mjs` | Source screening; screened build entry |
| `verify-paint.mjs` | Catalog-wide stroke and fill policy (`npm --prefix motion run verify:paint`) |
| `package.py` | Library archive (`npm --prefix motion run package:library`) |
| `build-guide.py` | Regenerates the AI operating guide (`npm run docs:build`) |
| `director-revision/serve-verification.py` | Native renderer verification server |

## Registration

Register a built set with `generative-icons library register SET-ID`; it adds the set's manifest to `prototype/icons/catalog.json` under a named category. The gallery resolves asset URLs from each manifest and renders every icon in order. Keep names unique.

Run `npm --prefix motion run test:library` after catalog, state or controller changes.

## Gallery behavior

Cards show name, artwork, motion choice and download. The selected motion loops while the pointer or keyboard focus stays inside a card; switching Light/Strong keeps it looping, and leaving finishes the current cycle. Stateful previews use their reverse or reset action. Touch plays one preview. Loop all runs every catalog icon; Stop, reduced motion and cleanup cancel pending cycles. Downloads follow the selected profile. Customize offers weight, color, preview size (24/32/48/128 px), speed, theme and sequence playback.
