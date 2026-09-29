# Commerce & delivery

Source set `15-commerce-delivery` in the Commerce & delivery gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-commerce-delivery`.

## Icons

- Storefront (`storefront`)
- Shopping bag (`shopping-bag`)
- Basket (`basket`)
- Product tag (`product-tag`)
- Product variants (`product-variants`)
- Discount (`discount`)
- Coupon (`coupon`)
- Loyalty points (`loyalty-points`)
- Gift card (`gift-card`)
- Subscription box (`subscription-box`)
- Checkout (`checkout`)
- Order confirmed (`order-confirmed`)
- Open package (`open-package`)
- Packing tape (`packing-tape`)
- Return package (`return-package`)
- Delivery truck (`delivery-truck`)
- Order tracking (`order-tracking`)
- Pickup locker (`pickup-locker`)
- Warehouse (`warehouse`)
- Inventory (`inventory`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/15-commerce-delivery
node motion/icon-sets/15-commerce-delivery/verify.mjs
node motion/icon-sets/15-commerce-delivery/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/15/assets/set.json)
