# Weather

Source set `35-weather` in the Weather gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-weather`.

## Icons

- Partly cloudy (`partly-cloudy`)
- Rain (`rain`)
- Snow (`snow`)
- Hail (`hail`)
- Thunderstorm (`thunderstorm`)
- Snowflake (`snowflake`)
- Wind (`wind`)
- Fog (`fog`)
- Tornado (`tornado`)
- Hurricane (`hurricane`)
- Rainbow (`rainbow`)
- Sunrise (`sunrise`)
- Sunset (`sunset`)
- Moon phase (`moon-phase`)
- Temperature (`temperature`)
- Humidity (`humidity`)
- UV index (`uv-index`)
- Umbrella (`umbrella`)
- Windsock (`windsock`)
- Weather radar (`weather-radar`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/35-weather
node motion/icon-sets/35-weather/verify.mjs
node motion/icon-sets/35-weather/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/35/assets/set.json)
