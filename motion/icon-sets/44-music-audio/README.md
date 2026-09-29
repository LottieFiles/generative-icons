# Music & audio

Source set `44-music-audio` in the Music & audio gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-music-audio`.

## Icons

- Music note (`music-note`)
- Vinyl record (`vinyl-record`)
- Cassette tape (`cassette-tape`)
- Audio waveform (`audio-waveform`)
- Loudspeaker (`loudspeaker`)
- Radio (`radio`)
- Metronome (`metronome`)
- Tuner (`tuner`)
- Piano keys (`piano-keys`)
- Guitar (`guitar`)
- Drum (`drum`)
- Lyrics (`lyrics`)
- Music album (`music-album`)
- Earbuds (`earbuds`)
- Mute audio (`mute-audio`)
- Crossfade (`crossfade`)
- Equalizer preset (`equalizer-preset`)
- Audio jack (`audio-jack`)
- Spatial audio (`spatial-audio`)
- Tuning fork (`tuning-fork`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/44-music-audio
node motion/icon-sets/44-music-audio/verify.mjs
```

Artwork and motion live in `definitions.mjs`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/44/assets/set.json)
