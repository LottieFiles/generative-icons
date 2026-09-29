# Infrastructure & devices

Source set `32-infrastructure-devices` in the Infrastructure & devices gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-infrastructure-devices`.

## Icons

- Server (`server`)
- Database (`database`)
- Processor (`processor`)
- Memory chip (`memory-chip`)
- Hard drive (`hard-drive`)
- Ethernet (`ethernet`)
- Router (`router`)
- Load balancer (`load-balancer`)
- Firewall (`firewall`)
- USB drive (`usb-drive`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/32-infrastructure-devices
node motion/icon-sets/32-infrastructure-devices/verify.mjs
node motion/icon-sets/32-infrastructure-devices/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/32/assets/set.json)
