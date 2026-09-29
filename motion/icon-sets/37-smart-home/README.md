# Smart Home

Source set `37-smart-home` in the Smart Home gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-smart-home`.

## Icons

- Smart bulb (`smart-bulb`)
- Thermostat (`thermostat`)
- Smart lock (`smart-lock`)
- Door sensor (`door-sensor`)
- Garage door (`garage-door`)
- Ceiling fan (`ceiling-fan`)
- Air conditioner (`air-conditioner`)
- Smart speaker (`smart-speaker`)
- Video doorbell (`video-doorbell`)
- Security camera (`security-camera`)
- Window blinds (`window-blinds`)
- Robot vacuum (`robot-vacuum`)
- Washing machine (`washing-machine`)
- Refrigerator (`refrigerator`)
- Oven (`oven`)
- Smoke alarm (`smoke-alarm`)
- Water leak (`water-leak`)
- Power outlet (`power-outlet`)
- Energy usage (`energy-usage`)
- Motion sensor (`motion-sensor`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/37-smart-home
node motion/icon-sets/37-smart-home/verify.mjs
node motion/icon-sets/37-smart-home/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/37/assets/set.json)
