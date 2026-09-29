# Preferences

Source set `50-preferences` in the Preferences gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-preferences`.

## Icons

- Language (`language`)
- Region (`region`)
- Currency (`currency`)
- Appearance (`appearance`)
- Dark mode (`dark-mode`)
- Quiet hours (`quiet-hours`)
- Privacy shield (`privacy-shield`)
- Shortcut key (`shortcut-key`)
- Mouse (`mouse`)
- Auto-rotate (`auto-rotate`)
- Sound toggle (`sound-toggle`)
- Battery saver (`battery-saver`)
- Mobile data (`mobile-data`)
- Haptic feedback (`haptic-feedback`)
- Reset settings (`reset-settings`)
- Auto-update (`auto-update`)
- Startup apps (`startup-apps`)
- Pointer speed (`pointer-speed`)
- Advanced settings (`advanced-settings`)
- Trackpad (`trackpad`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/50-preferences
node motion/icon-sets/50-preferences/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/50/assets/set.json)
