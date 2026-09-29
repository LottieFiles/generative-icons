# Travel

Source set `14-travel` in the Travel gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-travel`.

## Icons

- Airplane (`airplane`)
- Passport (`passport`)
- Boarding pass (`boarding-pass`)
- Suitcase (`suitcase`)
- Backpack (`backpack`)
- Train (`train`)
- Bus (`bus`)
- Car rental (`car-rental`)
- Ferry (`ferry`)
- Bicycle (`bicycle`)
- Hotel (`hotel`)
- Bed (`bed`)
- Tent (`tent`)
- Beach umbrella (`beach-umbrella`)
- Map (`map`)
- Route (`route`)
- Globe (`globe`)
- Ticket (`ticket`)
- Luggage tag (`luggage-tag`)
- Signpost (`signpost`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/14-travel
node motion/icon-sets/14-travel/verify.mjs
node motion/icon-sets/14-travel/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/14/assets/set.json)
