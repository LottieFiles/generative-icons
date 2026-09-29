## What changed

<!-- The problem and the resulting behavior. For icons: meaning, product context, initial/completed/reverse states. -->

**Set(s) affected:** <!-- e.g. 39-food-drink, or "none" -->

**Type:** <!-- new icon · icon revision · package/runtime · docs · tooling -->

## Checks

- [ ] `npm test` passes
- [ ] Set preflight passes: `node motion/icon-sets/library/preflight.mjs motion/icon-sets/SET-ID`
- [ ] `npm run check:source` passes (or failures are listed below)
- [ ] `npm run docs:build` and `npm run check:docs` pass (if docs or canonical guides changed)
- [ ] Existing icon IDs preserved; no unrelated assets changed

## Visual evidence

- [ ] Screenshots or recordings of **Light** and **Strong** motion at normal speed
- [ ] Outline, Solid and Duotone (if you could render them)
- [ ] Accessibility: reduced-motion pose is useful; stateful icons have a reverse/reset action

<!-- Attach media here. If you do not have the exporter, say so; a maintainer will render it. -->

## Checks I could not run

<!-- List anything skipped and why. -->

---

### For maintainers

- [ ] Native export built for the complete set; `verify.mjs` passes
- [ ] Set registered (`npm run icons:register -- SET-ID`) if new
- [ ] Rendered review: Light/Strong, all styles and weights, both themes, 24/32/48 px, reduced motion
- [ ] `npm run check:assets` and `npm run check:paint` pass on the native build
