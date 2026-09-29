# Twenty interface actions

Source set `05-interface` in the Interface gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-interface`.

## Icons

- Home (`home`)
- Calendar (`calendar`)
- Clock (`clock`)
- Camera (`camera`)
- Image (`image`)
- Mail (`mail`)
- Message (`message`)
- Phone (`phone`)
- Microphone (`microphone`)
- Volume (`volume`)
- Wi-Fi (`wifi`)
- Cloud (`cloud`)
- Sun (`sun`)
- Moon (`moon`)
- Location (`location`)
- Compass (`compass`)
- Link (`link`)
- Shield (`shield`)
- Cart (`cart`)
- Settings (`settings`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/05-interface
node motion/icon-sets/05-interface/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/05/assets/set.json)
