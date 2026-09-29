# Tables & analytics

Source set `28-tables-analytics` in the Tables & analytics gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-tables-analytics`.

## Icons

- Sort order (`sort-order`)
- Show columns (`show-columns`)
- Resize column (`resize-column`)
- Group rows (`group-rows`)
- Freeze column (`freeze-column`)
- Pivot table (`pivot-table`)
- Formula (`formula`)
- Sum (`sum`)
- Percentage (`percentage`)
- Line chart (`line-chart`)
- Pie chart (`pie-chart`)
- Scatter plot (`scatter-plot`)
- Heatmap (`heatmap`)
- Funnel chart (`funnel-chart`)
- Radar chart (`radar-chart`)
- Dashboard (`dashboard`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/28-tables-analytics
node motion/icon-sets/28-tables-analytics/verify.mjs
node motion/icon-sets/28-tables-analytics/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/28/assets/set.json)
