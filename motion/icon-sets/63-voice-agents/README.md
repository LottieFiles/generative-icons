# Voice agents

Source set `63-voice-agents` in the Voice agents category. Native review is available at `/icons/63/verify.html?save=1` through the verification server on port 4191. Catalog registration is maintained separately.

## Icons

- Turn taking (`turn-taking`) — Caller turn / Agent turn
- Barge in (`barge-in`) — Agent speaking / Caller interrupted
- End of turn detection (`end-of-turn-detection`) — Utterance open / Turn ended
- Backchannel (`backchannel`)
- Speaker diarization (`speaker-diarization`)
- Live transcript (`live-transcript`) — Transcribing / Transcript captured
- Call summary (`call-summary`) — Call details / Summary ready
- Sentiment meter (`sentiment-meter`)
- Intent confirmation (`intent-confirmation`) — Confirm intent / Intent confirmed
- Language detection (`language-detection`)
- Silence timeout (`silence-timeout`)
- DTMF input (`dtmf-input`)
- On hold (`on-hold`) — Call active / Call held
- Recording consent (`recording-consent`) — Permission pending / Recording permitted
- IVR menu (`ivr-menu`) — Choose option / Option selected
- Speech buffer (`speech-buffer`)
- Agent persona (`agent-persona`)
- Concurrent calls (`concurrent-calls`)
- Escalation flag (`escalation-flag`) — Normal priority / Escalated
- Voicemail detection (`voicemail-detection`)

## Motion and states

Each icon has distinct Light and Strong motion. Ten icons have authored forward and reverse state actions; the other ten complete a gesture. Speech buffer refills its packet only while the packet is invisible. All actions preserve common Light/Strong endpoint drawings.

Named editable parts have explicit pivots. Enclosed speech surfaces, call receivers, menu options, faces and detector lenses supply Duotone tint. Solid keeps internal openings clear. Outline supports the shared stroke-width token and 1.5 / 2 / 2.5 presets.

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/preflight.mjs motion/icon-sets/63-voice-agents
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/63-voice-agents
node motion/icon-sets/63-voice-agents/verify.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest. The set verifier adds transcript and caret clearance, DTMF tone clearance, IVR branch contacts, interruption-arrow contact and barb clearance, speech-bar clearance, mouth-to-face clearance, and clock-hand clearance checks in both profiles, both directions, Bold and Solid.

After the native browser check, run `python3 motion/icon-sets/63-voice-agents/package.py` to include current render evidence in the download.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/63/assets/set.json)
