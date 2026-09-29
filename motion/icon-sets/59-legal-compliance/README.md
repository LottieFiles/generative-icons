# Legal & compliance

Source set `59-legal-compliance` in the Legal & compliance gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-legal-compliance`.

## Icons

- Gavel (`gavel`)
- Justice scale (`justice-scale`)
- Contract (`contract`)
- Rubber stamp (`rubber-stamp`)
- Official seal (`official-seal`)
- Courthouse (`courthouse`)
- GDPR shield (`gdpr-shield`)
- Audit (`audit`)
- Policy (`policy`)
- Terms of service (`terms-of-service`)
- Compliance check (`compliance-check`)
- Redact (`redact`)
- Consent (`consent`)
- Cookie banner (`cookie-banner`)
- Data retention (`data-retention`)
- Disclaimer (`disclaimer`)
- Copyright (`copyright`)
- Whistleblower (`whistleblower`)
- Evidence bag (`evidence-bag`)
- Legal hold (`legal-hold`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/59-legal-compliance
node motion/icon-sets/59-legal-compliance/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/59/assets/set.json)
