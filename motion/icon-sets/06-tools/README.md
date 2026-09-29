# Tools and controls

Source set `06-tools` in the Tools gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-tools`.

## Icons

- User (`user`)
- Users (`users`)
- Key (`key`)
- Scan (`scan`)
- Battery (`battery`)
- Plug (`plug`)
- Bluetooth (`bluetooth`)
- Monitor (`monitor`)
- Smartphone (`smartphone`)
- Laptop (`laptop`)
- Printer (`printer`)
- Document (`document`)
- Clipboard (`clipboard`)
- Book (`book`)
- Credit card (`credit-card`)
- Wallet (`wallet`)
- Gift (`gift`)
- Flag (`flag`)
- Chart (`chart`)
- Filter (`filter`)
- Keyframes (`keyframes`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/06-tools
node motion/icon-sets/06-tools/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/06/assets/set.json)
