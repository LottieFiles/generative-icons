# Automotive

Source set `58-automotive` in the Automotive gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-automotive`.

## Icons

- Car (`car`)
- Electric car (`electric-car`)
- Fuel pump (`fuel-pump`)
- Speedometer (`speedometer`)
- Parking brake (`parking-brake`) — stateful: released / engaged
- Tire pressure (`tire-pressure`)
- Engine (`engine`)
- Seatbelt (`seatbelt`) — stateful: unfastened / fastened
- Headlights (`headlights`) — stateful: off / on
- Wiper (`wiper`)
- Steering wheel (`steering-wheel`)
- Car key (`car-key`)
- Car lock (`car-lock`) — stateful: unlocked / locked
- Car battery (`car-battery`)
- Gear shift (`gear-shift`) — stateful: first / second
- Car navigation (`car-navigation`) — stateful: idle / routed
- Car wash (`car-wash`)
- Engine oil (`engine-oil`)
- Car window (`car-window`) — stateful: closed / open
- Trunk (`trunk`) — stateful: closed / open

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/58-automotive
node motion/icon-sets/58-automotive/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest. `verify.mjs` also runs the set's drawing contracts (painted canvas margin, joints, contacts and clearances on the exported Bold and Solid geometry) and writes `evidence/clearance-verification.json`.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/58/assets/set.json)
