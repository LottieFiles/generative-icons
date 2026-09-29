# Onboarding & tips

20 native Lottie icons with editable geometry and Light/Strong motion in `definitions.mjs`.
Stateful icons include explicit reverse actions (54 actions in total).

Icons, in set order:

- `sparkle`
- `lightbulb`
- `product-tour`
- `tooltip`
- `hotspot`
- `new-badge`
- `confetti`
- `welcome-wave`
- `progress-checklist`
- `skip-step`
- `next-step`
- `spark-wand`
- `hotkey-hint`
- `spotlight`
- `coachmark`
- `getting-started`
- `tip-of-the-day`
- `welcome-gift`
- `first-run`
- `dismiss-tip`

From the repository root, with `CREATOR_MCP_BIN` set to the Engine exporter:

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/49-onboarding-tips
node motion/icon-sets/49-onboarding-tips/verify.mjs
```

Build the complete set. The native render check is at `/icons/49/verify.html?save=1`
on the verification server described in `docs/verification.md`.
Current source and native evidence is in `evidence/`; timestamped failed reports
remain available for diagnosis.
