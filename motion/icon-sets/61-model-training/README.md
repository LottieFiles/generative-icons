# Model training & fine-tuning

Source set `61-model-training` in the Model training & fine-tuning category. The native review page is `/icons/61/verify.html` on the verification server. Catalog registration is outside this task.

## Icons

- Epoch counter (`epoch-counter`)
- Loss curve (`loss-curve`) — stateful, with an authored reverse
- Learning rate (`learning-rate`) — stateful, with an authored reverse
- Gradient step (`gradient-step`)
- Overfitting (`overfitting`)
- Validation split (`validation-split`) — stateful, with an authored reverse
- Data augmentation (`data-augmentation`)
- Data labeling (`data-labeling`) — stateful, with an authored reverse
- GPU cluster (`gpu-cluster`)
- Hyperparameter (`hyperparameter`)
- Early stopping (`early-stopping`) — stateful, with an authored reverse
- Distillation (`distillation`)
- Adapter layer (`adapter-layer`) — stateful, with an authored reverse
- Reward model (`reward-model`)
- Benchmark score (`benchmark-score`) — stateful, with an authored reverse
- Tokenizer (`tokenizer`) — stateful, with an authored reverse
- Quantize (`quantize`) — stateful, with an authored reverse
- Pretrained weights (`pretrained-weights`)
- Data shuffle (`data-shuffle`)
- Convergence (`convergence`) — stateful, with an authored reverse

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/preflight.mjs motion/icon-sets/61-model-training
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/61-model-training
node motion/icon-sets/61-model-training/verify.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest. The set has 20 icons and 60 native actions. Ten stateful icons retain their result and include a true reverse in each motion profile.

The set verifier checks provenance, theme and style bindings, state boundaries, holds, rigid transforms, painted bounds, GPU bus contacts, slider contacts and adapter clearances. Start the native renderer server with `python3 motion/icon-sets/library/director-revision/serve-verification.py`, open `/icons/61/verify.html?save=1`, and run verification. After a passing render, run `python3 motion/icon-sets/61-model-training/package.py` to include current evidence.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Design and review record](evidence/design-review.md)
- [Native asset manifest](../../../prototype/icons/61/assets/set.json)
