# Health & Fitness

Source set `36-health-fitness` in the Health & Fitness gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-health-fitness`.

## Icons

- Heart rate (`heart-rate`)
- Step counter (`step-counter`)
- Dumbbell (`dumbbell`)
- Running (`running`)
- Water intake (`water-intake`)
- Sleep tracking (`sleep-tracking`)
- Calories (`calories`)
- Stethoscope (`stethoscope`)
- Pill (`pill`)
- Syringe (`syringe`)
- First aid kit (`first-aid-kit`)
- Bandage (`bandage`)
- Weight scale (`weight-scale`)
- Blood pressure (`blood-pressure`)
- Yoga (`yoga`)
- Meditation (`meditation`)
- Nutrition (`nutrition`)
- Jump rope (`jump-rope`)
- Hospital (`hospital`)
- DNA (`dna`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/36-health-fitness
node motion/icon-sets/36-health-fitness/verify.mjs
node motion/icon-sets/36-health-fitness/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/36/assets/set.json)
