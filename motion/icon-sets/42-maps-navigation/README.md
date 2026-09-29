# Maps & navigation

Source set `42-maps-navigation` in the Maps & navigation gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-maps-navigation`.

## Icons

- Pin cluster (`pin-cluster`)
- Current location (`current-location`)
- Directions (`directions`)
- Turn by turn (`turn-by-turn`)
- Route recalculate (`route-recalculate`)
- Traffic light (`traffic-light`)
- Map layers (`map-layers`)
- Satellite (`satellite`)
- Street view (`street-view`)
- Geofence (`geofence`)
- Distance (`distance`)
- Elevation (`elevation`)
- Parking (`parking`)
- Fuel station (`fuel-station`)
- EV charger (`ev-charger`)
- Transit stop (`transit-stop`)
- Walking route (`walking-route`)
- Roundabout (`roundabout`)
- Offline map (`offline-map`)
- Toll gate (`toll-gate`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/42-maps-navigation
node motion/icon-sets/42-maps-navigation/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/42/assets/set.json)
