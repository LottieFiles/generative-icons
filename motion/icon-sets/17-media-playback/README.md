# Media & playback

Source set `17-media-playback` in the Media & playback gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-media-playback`.

## Icons

- Play / Pause (`play-pause`)
- Stop (`stop`)
- Next track (`next-track`)
- Previous track (`previous-track`)
- Replay 10 seconds (`replay-10`)
- Forward 10 seconds (`forward-10`)
- Shuffle (`shuffle`)
- Repeat (`repeat`)
- Playlist (`playlist`)
- Add to queue (`add-to-queue`)
- Headphones (`headphones`)
- Podcast (`podcast`)
- Live broadcast (`live-broadcast`)
- Cast (`cast`)
- Screen share (`screen-share`)
- Video call (`video-call`)
- Closed captions (`closed-captions`)
- Fullscreen (`fullscreen`)
- Picture-in-picture (`picture-in-picture`)
- Camera off (`camera-off`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/17-media-playback
node motion/icon-sets/17-media-playback/verify.mjs
node motion/icon-sets/17-media-playback/verify-clearance.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/17/assets/set.json)
