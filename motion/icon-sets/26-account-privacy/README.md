# Account & privacy

Source set `26-account-privacy` in the Account & privacy gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-account-privacy`.

## Icons

- Sign in / out (`sign-in-out`)
- Switch account (`switch-account`)
- ID card (`id-card`)
- Password (`password`)
- Passkey (`passkey`)
- Fingerprint (`fingerprint`)
- Face authentication (`face-authentication`)
- Two-factor code (`two-factor-code`)
- Recovery codes (`recovery-codes`)
- Private browsing (`private-browsing`)
- Cookie preferences (`cookie-preferences`)
- Block user (`block-user`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/26-account-privacy
node motion/icon-sets/26-account-privacy/verify.mjs
node motion/icon-sets/26-account-privacy/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/26/assets/set.json)
