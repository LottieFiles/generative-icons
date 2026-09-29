# Forms & input

Source set `27-forms-input` in the Forms & input gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-forms-input`.

## Icons

- Checkbox (`checkbox`)
- Radio button (`radio-button`)
- Toggle switch (`toggle-switch`)
- Text field (`text-field`)
- Number field (`number-field`)
- Select field (`select-field`)
- Multiple choice (`multiple-choice`)
- Signature (`signature`)
- Form builder (`form-builder`)
- QR code (`qr-code`)
- Barcode (`barcode`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/27-forms-input
node motion/icon-sets/27-forms-input/verify.mjs
node motion/icon-sets/27-forms-input/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/27/assets/set.json)
