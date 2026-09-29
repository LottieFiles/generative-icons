# Creation and multimodal

Source set `09-ai-creation` in the AI · Creation gallery category. To browse it, run `npm run dev` from the repository root and open `/icons/01/#category-ai-creation`.

## Icons

- Image generate (`image-generate`)
- Generative fill (`generative-fill`)
- Background remove (`background-remove`)
- Object select (`object-select`)
- Image upscale (`image-upscale`)
- Style transfer (`style-transfer`)
- Smart crop (`smart-crop`)
- Image caption (`image-caption`)
- Text to speech (`text-to-speech`)
- Speech to text (`speech-to-text`)
- Voice chat (`voice-chat`)
- Audio generate (`audio-generate`)
- Audio clean (`audio-clean`)
- Music generate (`music-generate`)
- Video generate (`video-generate`)
- Storyboard (`storyboard`)
- Motion generate (`motion-generate`)
- Code generate (`code-generate`)
- Code explain (`code-explain`)
- Code review (`code-review`)

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/09-ai-creation
node motion/icon-sets/09-ai-creation/verify.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/09/assets/set.json)
