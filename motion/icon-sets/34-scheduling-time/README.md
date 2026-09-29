# Scheduling & time tools

Source set `34-scheduling-time` in the Scheduling & time tools gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-scheduling-time`.

## Icons

- Date range (`date-range`)
- Calendar view (`calendar-view`)
- Reschedule (`reschedule`)
- Alarm clock (`alarm-clock`)
- Stopwatch (`stopwatch`)
- Countdown timer (`countdown-timer`)
- World clock (`world-clock`)
- Availability (`availability`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/34-scheduling-time
node motion/icon-sets/34-scheduling-time/verify.mjs
node motion/icon-sets/34-scheduling-time/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/34/assets/set.json)
