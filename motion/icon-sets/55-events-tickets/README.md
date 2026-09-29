# Events & tickets

Source set `55-events-tickets` in the Events & tickets gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-events-tickets`.

## Icons

- RSVP (`rsvp`)
- Event invite (`event-invite`)
- Venue (`venue`)
- Seat map (`seat-map`)
- Agenda (`agenda`)
- Event speaker (`event-speaker`)
- Event badge (`event-badge`)
- Check-in (`check-in`)
- Add to calendar (`add-to-calendar`)
- Event reminder (`event-reminder`)
- Ticket scan (`ticket-scan`)
- Ticket transfer (`ticket-transfer`)
- Waitlist (`waitlist`)
- Sold out (`sold-out`)
- Early bird (`early-bird`)
- Event live (`event-live`)
- Event replay (`event-replay`)
- Sponsor (`sponsor`)
- Networking (`networking`)
- Event map (`event-map`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/55-events-tickets
node motion/icon-sets/55-events-tickets/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/55/assets/set.json)
