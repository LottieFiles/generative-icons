# Fintech & finance

Source set `13-fintech-finance` in the Fintech & finance gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-fintech-finance`.

## Icons

- Bank (`bank`)
- Bank account (`bank-account`)
- Bank transfer (`bank-transfer`)
- Currency exchange (`currency-exchange`)
- Savings (`savings`)
- Deposit (`deposit`)
- Withdraw (`withdraw`)
- Payment terminal (`payment-terminal`)
- Contactless payment (`contactless-payment`)
- Recurring payment (`recurring-payment`)
- Receipt (`receipt`)
- Invoice (`invoice`)
- Refund (`refund`)
- Bill payment (`bill-payment`)
- Budget (`budget`)
- Transaction history (`transaction-history`)
- Investment (`investment`)
- Portfolio (`portfolio`)
- Calculator (`calculator`)
- Vault (`vault`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/13-fintech-finance
node motion/icon-sets/13-fintech-finance/verify.mjs
node motion/icon-sets/13-fintech-finance/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/13/assets/set.json)
