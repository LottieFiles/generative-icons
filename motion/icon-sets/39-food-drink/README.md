# Food & Drink

Source set `39-food-drink` in the Food & Drink gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-food-drink`.

## Icons

- Coffee cup (`coffee-cup`)
- Tea cup (`tea-cup`)
- Wine glass (`wine-glass`)
- Beer mug (`beer-mug`)
- Cocktail (`cocktail`)
- Cooking pot (`cooking-pot`)
- Frying pan (`frying-pan`)
- Kettle (`kettle`)
- Toaster (`toaster`)
- Blender (`blender`)
- Chef hat (`chef-hat`)
- Cutlery (`cutlery`)
- Pizza (`pizza`)
- Burger (`burger`)
- Cake (`cake`)
- Ice cream (`ice-cream`)
- Noodle bowl (`noodle-bowl`)
- Bread (`bread`)
- Recipe book (`recipe-book`)
- Restaurant menu (`restaurant-menu`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/39-food-drink
node motion/icon-sets/39-food-drink/verify.mjs
node motion/icon-sets/39-food-drink/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/39/assets/set.json)
