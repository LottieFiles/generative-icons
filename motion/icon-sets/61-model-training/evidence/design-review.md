# Model training & fine-tuning — design and review

Scope: exactly 20 generic icons, Light and Strong, Outline 1.5/2/2.5, Solid, Duotone. Ten icons have retained states and authored reverse actions. One coding agent authors and reviews this set. User acceptance is separate from these checks.

## Search and selection

Before selecting each ID, the exact manifest grep and `generation-context.mjs --query '<meaning>' --search` ran for all 24 proposed meanings. None was an exact ID match. Training run, checkpoint, training queue and human feedback were excluded because model-training, agent-checkpoint, batch-inference and human-input already cover nearby meanings. Reference packets were read for model-training, fine-tune, model-evaluation, embeddings, dataset, batch-inference, confidence-score, human-input, model-version, classification and clustering.

The existing training icon is a ball in a loss bowl; fine-tune is a tuning fork; evaluation is a balance; confidence-score is an interval around an estimate. This set avoids those drawings. Set 39 supplies enclosed-surface, Solid-counter and visible-reset lessons. Set 34 supplies rigid pivots, shared contacts and authored state boundaries. No previous artwork is copied.

## Proposals and integration review

All parts have stable names and declared pivots. Exact geometry and times are in the named batches. Every row has separate primary Light/Strong direction, not only extra follower keys. Stateful data progress is monotonic; mechanical insertion may have one bounded seating response.

| ID | Meaning and drawing | State / fixed structure | Primary Light / Strong action | Protected construction / references |
| --- | --- | --- | --- | --- |
| epoch-counter | Repeated dataset pass; orbit arrows around tally capsule | Gesture; fixed tally | One 360-degree turn / 16-degree preparation then two turns | Arrow envelope and tally counter; stopwatch, counter-arcs |
| loss-curve | Recorded loss declines along plot | High loss → lower loss; fixed axes | 10-unit decline in .78 s / most by .20 s, done .40 s | Hollow endpoint and shared trace terminal; chart, model-training distinction |
| learning-rate | Later learning steps use smaller rates | Constant → decay; first step fixed | Two later bars shrink sequentially / overlapping decisive contraction | Wide rounded step blocks; fine-tune distinction |
| gradient-step | Direction toward the next optimizer step | Gesture; fixed stair and parameter | 1.2-unit diagonal press / 2.7-unit drive with recoil | Arrow stays clear of staircase; next-step |
| overfitting | Fit bends excessively around observations | Gesture; fixed observations and window | 1.8-unit curve flex / 3.7-unit flex with release | Open Solid plot counter; bezier-handles |
| validation-split | Hold out one sample partition | Combined → separated | Lower packet moves +4, upper packets -2 / preparation and bounded wider split | Vertical packet lanes; dataset, split-view |
| data-augmentation | Changed view of the same image | Gesture; source fixed | Copy turns 12 degrees / 25 degrees plus 1-unit lift | Independent image counters, rigid copy; image-generate |
| data-labeling | Attach label to selected record | First record → selected record | Tag slides nine units / preparation, fast slide, seat | Tag connector follows actual tag boundary; dataset |
| gpu-cluster | Four linked compute modules under load | Gesture; linked module assembly | .8-unit inward draw / 1.8-unit drive | Bus terminals meet module edges at every frame; clustering distinction |
| hyperparameter | Search independent parameter values | Gesture; fixed rail ends | 2.5-unit sweeps / 6-unit staggered sweeps | Rails terminate at moving handle boundaries; fine-tune distinction |
| early-stopping | Stop when loss has plateaued | Monitoring → stopped | Stop block lowers 13 units / fast descent and brake | Loss curve, stop block and plot bounds; model-training distinction |
| distillation | Large teacher transfers to small student | Gesture; funnel and receiver fixed | One packet / two faster packets | Recycle only while invisible; enclosed vessel surfaces |
| adapter-layer | Insert smaller layer between fixed layers | Released → inserted | Four-unit slide / prepare, seat, settle | Painted clearances above/below adapter; layer-stack |
| reward-model | Reward signal above candidate outputs | Gesture; outputs fixed | Star turns 12 degrees / 32-degree prepared turn | Rounded star corners and candidate gap; model-evaluation distinction |
| benchmark-score | Measured score exceeds reference | Baseline → exceeded | Full monotonic rise in .8 s / main rise by .23 s | Separate benchmark rule and fixed baseline; confidence-score distinction |
| tokenizer | Split a word into token cells | Joined → separate | Gradual separation / decisive full separation | Open Outline cells and filled Solid blocks; dataset distinction |
| quantize | Map continuous values to discrete levels | Continuous → discrete | Rounded slope morph / fast main step formation | Constant path topology and three real cells; embeddings distinction |
| pretrained-weights | Load stored coefficient cartridge | Gesture; shelf and guide fixed | One-unit lift and seat / three-unit lift with landing response | Coefficients rigidly carried; embeddings distinction |
| data-shuffle | Mix sample packets in opposing lanes | Gesture; lane heights fixed | One two-unit mix / four-unit mix and shorter second pass | Packet lane clearance; shuffle, dataset distinction |
| convergence | Error bounds approach common target | Wide → converged range | Monotonic approach / main approach by .24 s | Trajectories stay separate and clear of target rim; model-training distinction |

## Gate record

Initial source preflight: PASS, 20 icons, 60 actions, no diagnostics. Native build, geometric checks and browser rendering follow. Current package hashes will be recorded in `build.json` and matched against `browser-verification.json`. Failed native render reports will remain in this evidence folder. Source checks do not imply visual approval.

## Native review corrections

Round 1 passed the full browser sweep: 60 actions, every style, weight, size, theme and native playback. Its report and images are preserved under `round-1/`. Agent inspection of the native style sheets found details that needed correction despite the passing gates:

- Epoch counter: widened the counter and shortened the tally to preserve a clear gap at Bold and Solid.
- Data augmentation: replaced chevrons with filled mountain and sun marks, so both tiles read as images.
- Gradient step: expanded the Solid parameter surface to bury the staircase cap.
- Overfitting: enlarged fixed sample marks so they remain distinct from the fit stroke.
- Tokenizer: removed crowded inner glyphs; Solid uses complete token blocks.
- Pretrained weights: changed tiny hollow coefficient marks to filled squares.
- Distillation: rounded the actual funnel contours, including Solid and its cutout.
- Data shuffle: increased vertical lane separation.

Round 2 preflight, complete native build, set verifier and token check passed. One verifier invocation during the rebuild read the previous manifest and correctly rejected a stale authoring-source hash; after the complete build finished, the unchanged verifier passed. No assertion was weakened. The drawing checks cover 80,596 observations across every action, with minimum painted margin 2.1, adapter clearance 1.5 and bus contact error zero. Browser proof and small native review are checked against the current package hashes before delivery.
