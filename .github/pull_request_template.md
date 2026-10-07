## What changed

<!-- The problem and the resulting behavior. For icons: meaning, product context, initial/completed/reverse states. -->

**Set(s) affected:** <!-- e.g. 39-food-drink, or "none" -->

**Type:** <!-- new icon · icon revision · CLI · package/runtime · docs · tooling -->

## Checks

- [ ] `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass (CLI or crate changes)
- [ ] Set preflight passes: `generative-icons library preflight SET-ID`
- [ ] Set built and verified: `generative-icons library build SET-ID`, `library verify SET-ID`, `library verify-visual SET-ID`
- [ ] `npm test` passes
- [ ] `npm run docs:build` and `npm run check:docs` pass (if docs or canonical guides changed)
- [ ] Existing icon IDs preserved; no unrelated assets changed

## Visual evidence

- [ ] Preview sheets or recordings of **Light** and **Strong** motion at normal speed
- [ ] Outline, Solid and Duotone, in light and dark themes
- [ ] Accessibility: reduced-motion pose is useful; stateful icons have a reverse/reset action

<!-- Attach media here. generative-icons preview and library verify-visual write the sheets. -->

## Checks I could not run

<!-- List anything skipped and why, for example a build that needs a LottieFiles sign-in. -->

---

### For maintainers

- [ ] Complete set built; `generative-icons library verify` and `verify-visual` pass on the committed assets
- [ ] Set registered (`generative-icons library register SET-ID`) if new
- [ ] Rendered review: Light/Strong, all styles and weights, both themes, 24/32/48 px, reduced motion
- [ ] `generative-icons library verify --paint` and `npm run check:assets` pass
