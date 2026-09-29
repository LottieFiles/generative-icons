# Models & providers

Source set `60-models-providers` in the Models & providers category. This set contains generic app integration concepts and no company logos or brand marks. The catalog is intentionally unchanged for this task. Native verification is available at `/icons/60/verify.html` on the verification server.

## Icons

- API key (`api-key`)
- Endpoint (`endpoint`) — stateful; explicit reverse
- Rate limit (`rate-limit`)
- Token counter (`token-counter`)
- Request timeout (`request-timeout`)
- Model card (`model-card`)
- Open weights (`open-weights`) — stateful; explicit reverse
- Model picker (`model-picker`) — stateful; explicit reverse
- Provider switch (`provider-switch`) — stateful; explicit reverse
- Latency meter (`latency-meter`)
- Cost estimate (`cost-estimate`)
- Temperature dial (`temperature-dial`) — stateful; explicit reverse
- Top P (`top-p`)
- Max tokens (`max-tokens`) — stateful; explicit reverse
- Seed lock (`seed-lock`) — stateful; explicit reverse
- API quota (`api-quota`)
- Model status (`model-status`) — stateful; explicit reverse
- Fallback model (`fallback-model`) — stateful; explicit reverse
- Prompt cache hit (`prompt-cache-hit`) — stateful; explicit reverse
- Multimodal input (`multimodal-input`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/preflight.mjs motion/icon-sets/60-models-providers
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/60-models-providers
node motion/icon-sets/60-models-providers/verify.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest. The verifier includes independent contact, clearance, pivot and shackle paint-margin checks on every exported action.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/60/assets/set.json)
- [Design and review record](evidence/design-review.md)

## Verified output

The final preflight passed with 20 icons, 60 actions and no diagnostics. Native source verification and the stroke-width token check passed. The full browser check passed all 60 actions at 24, 32, 48 and 192 px, across styles, weights and themes, including forward/reverse continuity and native playback completion.

- [Native browser evidence](evidence/browser-verification.json)
- [Native visual review](evidence/visual-review.md)
- [Source bundle rebuild proof](evidence/bundle-verification.json)

The extracted source bundle was rebuilt with the Engine exporter. Its set verifier passed, and all 60 rebuilt native JSON actions were byte-identical to the checkout. User acceptance is separate from these checks.
