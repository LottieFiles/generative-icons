# Search & discovery

Source set `41-search-discovery` in the Search & discovery gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-search-discovery`.

## Icons

- Search suggest (`search-suggest`)
- Recent searches (`recent-searches`)
- Search clear (`search-clear`)
- Search voice (`search-voice`)
- Search image (`search-image`)
- Search filter (`search-filter`)
- Sort ascending (`sort-ascending`)
- Sort descending (`sort-descending`)
- Hashtag (`hashtag`)
- Tag add (`tag-add`)
- Tag remove (`tag-remove`)
- Trending up (`trending-up`)
- Trending down (`trending-down`)
- Telescope (`telescope`)
- Related items (`related-items`)
- Saved search (`saved-search`)
- Web search (`web-search`)
- Search match (`search-match`)
- Search scope (`search-scope`)
- Searching (`searching`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/41-search-discovery
node motion/icon-sets/41-search-discovery/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/41/assets/set.json)
