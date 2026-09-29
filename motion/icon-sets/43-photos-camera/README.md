# Photos & camera

Source set `43-photos-camera` in the Photos & camera gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-photos-camera`.

## Icons

- Flash toggle (`flash-toggle`)
- Self-timer (`self-timer`)
- Switch camera (`switch-camera`)
- Tripod (`tripod`)
- Aperture (`aperture`)
- Portrait mode (`portrait-mode`)
- Live photo (`live-photo`)
- Burst mode (`burst-mode`)
- Panorama (`panorama`)
- Exposure (`exposure`)
- Photo gallery (`photo-gallery`)
- Photo album (`photo-album`)
- Photo edit (`photo-edit`)
- Rotate photo (`rotate-photo`)
- Flip photo (`flip-photo`)
- Brightness (`brightness`)
- Saturation (`saturation`)
- Red-eye fix (`red-eye-fix`)
- Photo delete (`photo-delete`)
- Photo favorite (`photo-favorite`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/43-photos-camera
node motion/icon-sets/43-photos-camera/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/43/assets/set.json)
