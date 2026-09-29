# Notifications

Source set `40-notifications` in the Notifications gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-notifications`.

## Icons

- Bell ring (`bell-ring`)
- Bell off (`bell-off`)
- Bell snooze (`bell-snooze`)
- Notification badge (`notification-badge`)
- Toast message (`toast-message`)
- Banner alert (`banner-alert`)
- Push notification (`push-notification`)
- Inbox zero (`inbox-zero`)
- Mark all read (`mark-all-read`)
- Notification stack (`notification-stack`)
- Mention alert (`mention-alert`)
- Priority alert (`priority-alert`)
- Do not disturb (`do-not-disturb`)
- Reminder ping (`reminder-ping`)
- Daily digest (`daily-digest`)
- Notification settings (`notification-settings`)
- Alert dismiss (`alert-dismiss`)
- Alert expand (`alert-expand`)
- Sound alert (`sound-alert`)
- Vibrate alert (`vibrate-alert`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/40-notifications
node motion/icon-sets/40-notifications/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/40/assets/set.json)
