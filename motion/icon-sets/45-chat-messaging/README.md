# Chat & messaging

Source set `45-chat-messaging` in the Chat & messaging category. This set is not registered by this task. Its native review page is `http://127.0.0.1:4191/icons/45/verify.html`.

## Icons

- Composing message (`composing-message`)
- Message sent (`message-sent`)
- Message delivered (`message-delivered`)
- Message read (`message-read`)
- Message reaction (`message-reaction`)
- Quoted message (`quoted-message`)
- Forward message (`forward-message`)
- Pin message (`pin-message`)
- Conversation history (`conversation-history`)
- Mute chat (`mute-chat`)
- Group chat (`group-chat`)
- Voice note (`voice-note`)
- Sticker (`sticker`)
- GIF picker (`gif-picker`)
- Edit message (`edit-message`)
- Delete message (`delete-message`)
- Unread divider (`unread-divider`)
- Archive chat (`archive-chat`)
- Disappearing message (`disappearing-message`)
- Chat search (`chat-search`)

Stateful icons with authored reverse actions: `message-sent`, `message-delivered`, `message-read`, `message-reaction`, `pin-message`, `mute-chat`, `edit-message`, `delete-message`, `archive-chat`.

## Rebuild

From the repository root, with the Engine exporter configured (`npm run doctor`):

```sh
node motion/icon-sets/library/preflight.mjs motion/icon-sets/45-chat-messaging
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/45-chat-messaging
node motion/icon-sets/45-chat-messaging/verify.mjs
```

Artwork and motion live in `definitions.mjs` and `batches/`. Always rebuild the complete set; the builder rewrites the whole manifest. Verification includes the set-owned pen contact, sticker-fold contacts and waveform clearance checks.

- [Creating icons](../../../docs/creating-icons.md)
- [Development setup](../../../docs/development.md)
- [Verification](../../../docs/verification.md)
- [Native asset manifest](../../../prototype/icons/45/assets/set.json)
