# Final native visual review

Status: verified locally. User acceptance is separate.

The required browser gate passed all 58 actions at 24/32/48/192 px, all styles, three outline weights and both themes. Each action completed native playback at speed 1. The agent inspected native style images, true-size 24/32/48 px sheets, and equal-time Light/Strong still sequences. This is sampled visual review plus automated all-frame coverage, not human approval or a visual review of every frame.

Primary visual records: native-small-sizes-1.png through native-small-sizes-4.png, native-equal-time-1.png through native-equal-time-4.png, and native-review-capture.json. Exact package hashes are below and in browser-verification.json. Round 1 evidence is retained under round-1/.

Source verification: 4,640 drawing-specific checks; zero pen and fold contact error; minimum painted gaps 1.44 (wave), 1.07 (smile/fold), 1.40 (caret/border) grid units. No checks were weakened. No new shared rule was needed.

## composing-message

Caret travel stays inside the border in Strong; two text rows stay fixed. State policy: gesture. Reviewed in sheet 1. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- composing-message: `d754204b633d9eadb3f625488ee252fcb49756e6be634d23c03438bb65f6236a`
- composing-message-strong: `ea3ac4ce7884db5b320ec62a0e6109c53823dc895c87cdd6de1b83aa18b30a7e`

## message-sent

One check seats below a fixed message. Strong completes the primary arrival earlier. State policy: retained state and authored reverse. Reviewed in sheet 1. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- message-sent: `a39eb6197e405143ee39b6b6b2aa48ade2dd3a382af8bf6e4ec3b2b485e8ebc6`
- message-sent-reverse: `e7bd6cb409a3028e7e9be32971cab100a32a624aa8059474902d29c088b1e302`
- message-sent-strong: `dc2c29baca2fdd8c086d5d370467cd84aec5fc7d73f09b4b257bfd4f1028bc04`
- message-sent-reverse-strong: `1cf46f00c0d6e60e2d1b5df49a7baa70db9c08396b802a416f0aa2f40d9f46c7`

## message-delivered

The second receipt joins the first. The first receipt stays visible in both states. State policy: retained state and authored reverse. Reviewed in sheet 1. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- message-delivered: `d2230ba790dc7746674dfa15428d619a2f16bcb1892c86ecf447771e5a67b2da`
- message-delivered-reverse: `e89b93439da96946cd28506de944dffa7e36203be1f74227856135e86aaf9680`
- message-delivered-strong: `fd9605e4ee6da5b579c493f880fc9d86aaf52c7cf33303f6b11e289107588272`
- message-delivered-reverse-strong: `b568e5caadb098aef38bb213b3998484499f690c6f5adb225b5bf67a353afe59`

## message-read

The eye opens from a closed lid; the pupil stays inside the open eye. State policy: retained state and authored reverse. Reviewed in sheet 1. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- message-read: `d1fb04a6699e40f7b330db88a76305b4ce4fea6127f94edcc88358b274843952`
- message-read-reverse: `5d8a8c4db455d1fde78ef716b3ebf7b6c14917b1812d91caa32baf0761eb6fac`
- message-read-strong: `39f10861885585a105f2831f34373480aaf0edd94e924f8ca741e73bdc55682e`
- message-read-reverse-strong: `232d42809c077d16c386d9aadd57bd2c068a0b36f21948ed1ec568667789b406`

## message-reaction

The heart rises 1.6 units in both profiles; Strong adds the larger visible angular gesture and earlier arrival. State policy: retained state and authored reverse. Reviewed in sheet 1. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- message-reaction: `0a7fef79434177f72aca60a4616dc360054fde11782cf08801198baa6cd3a636`
- message-reaction-reverse: `6fc4c240399453ee7221631d333ac05362c48e3277b0c58febf40d3b04a768f1`
- message-reaction-strong: `aa57762b05ae059bace49029aefd0c42f9aa3681b03f302f46786fb815aedbd5`
- message-reaction-reverse-strong: `534354aa472ca5f6149dd0098ec1492179eaccfdb4d2c1d8a44bbf98b63f2ac0`

## quoted-message

Quote marks indent while the underline and bubble remain fixed. State policy: gesture. Reviewed in sheet 2. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- quoted-message: `1ede20feff649860df46878098c891ebb08086a326503feff13707614e2eff40`
- quoted-message-strong: `9797d589c76d89e68b794497f41e10ac66e1895c3064cd0633b969e12b513f6c`

## forward-message

Arrow and shaft translate together; Strong uses a larger drive and preparation. State policy: gesture. Reviewed in sheet 2. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- forward-message: `0b1732be60b88891d0c76a88cf5384e9c5b374b3c59445451f3d4bee006b630d`
- forward-message-strong: `2248845acb137fe6b21a48023ddb1d615bd5877de7d09dbac713f087a0d91e33`

## pin-message

A filled pin head and needle insert together. Strong turns the pin during insertion. State policy: retained state and authored reverse. Reviewed in sheet 2. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- pin-message: `5bcd3984bb800ea70b034c2c06ef55e60d814940981b53e91d4502b7f59a75f2`
- pin-message-reverse: `c7f02d9ac1904f1ef78d2896f3e4adeb86480a09f1e85eb43228d55635de027b`
- pin-message-strong: `7e26c9f2bd81a87d8b0eed345bef87fa9d729406f89f42bd62c53b4b8ac733f8`
- pin-message-reverse-strong: `9eb3d46f2750cb75e81676e40b7cba5eddf8aacd30e2cc5ee4cd79eb5da90b48`

## conversation-history

Transcript rows scroll under the top message. The rail endpoint stays clear in Solid. State policy: gesture. Reviewed in sheet 2. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- conversation-history: `9bb3b3b42b000f921a90f7ce4bc0bb5acd7b5f3c2027044756b3a3f56e439039`
- conversation-history-strong: `6853a388c0aaf31c8bc7d4c9a54ca8074d82ebed9868e8f54d805969737c6bba`

## mute-chat

The slash draws and remains clear of the lower bubble border at Bold and Solid. State policy: retained state and authored reverse. Reviewed in sheet 2. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- mute-chat: `82f52f67a5979350b2a09044eadb8b39a68541268bb04b4b067ec3cf7858b5cb`
- mute-chat-reverse: `ae50fcf0209fb9d7acad6f8e5a6e6cfc52b73e0c0390ac5687d4c8c169190883`
- mute-chat-strong: `44593373c945b7662a9af530b1091be93668a6407ffcae60f451e9e9fdae576d`
- mute-chat-reverse-strong: `e2f0faa576f6135f14dcceac0f6c6f49da5477eb7afbbf6d98fd8af3110a59b1`

## group-chat

Two portraits nod in sequence; Strong has visibly greater travel. State policy: gesture. Reviewed in sheet 3. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- group-chat: `35969b3206289579e22044e850e96ccb1c4b5673e2e3ab1b1e51f9e4fcee74e9`
- group-chat-strong: `9df9d494c5a2833744d36ee48567acca642efd3d54ac1d6ed05041306e0eda6d`

## voice-note

The playback triangle stays fixed while three waveform bars articulate in a saved clip. State policy: gesture. Reviewed in sheet 3. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- voice-note: `ec0a175e693e78b21e9e15119b9ac844fb9d25bd1738d264a5d62ba032b5a5ae`
- voice-note-strong: `8b7b29612ecb3cdb87e207eea9de53b6a56d245d432d2de6c51fa28edd9d39f8`

## sticker

The folding corner follows both paper endpoints and clears the smile throughout Strong. State policy: gesture. Reviewed in sheet 3. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- sticker: `464e04c2e449023998f042f91470373c5ab6f0be9e28993c8cf69f039563bea2`
- sticker-strong: `630465100a405488917f777eb53e4996c6cc8fc22c88e5c7503651131b066707`

## gif-picker

The tile lifts above its fixed stack line; the G counter remains readable at heavy weights. State policy: gesture. Reviewed in sheet 3. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- gif-picker: `5399002ae20aa292170e852cc7979be70eec4f63a7304cadf5fc604e42ec0a3d`
- gif-picker-strong: `27cd2d4522d0298c7c74e9ca0335fe31f24ccfbb9afca04721afe356aafdc398`

## edit-message

The pencil has a separate nib and barrel; ink ends at the nib throughout both directions. State policy: retained state and authored reverse. Reviewed in sheet 3. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- edit-message: `1d5425534b362a7b9a3e7d4167a43f0366ebd220ad9aba561ec67f9ff09ef9e5`
- edit-message-reverse: `632deb0fd487d3b737c314b6988b02ea72fbb6abdee203d738bc16bf842c5319`
- edit-message-strong: `3ae6093dfd26866d13ab226f825db3790eed2c961cfa8a2bfd7b55f2e8b92498`
- edit-message-reverse-strong: `41682c3cb7db4c1bdc832690d7e32253efff0320053830bd486968b8c89fd7a0`

## delete-message

Content lines move and fade into a retained removal mark; the housing never disappears. State policy: retained state and authored reverse. Reviewed in sheet 4. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- delete-message: `dba22a661361523fcde7f93d424f0440a5579340caaeec2ae68e17c95d3f02cd`
- delete-message-reverse: `ae0f68e04a186b2e56bf2e9bc1d2ee96be4e6dd214cd2f0bc3979798ad84bb6a`
- delete-message-strong: `bd095a67096904b65316aa1df82bcdc1210781b9b7e14770307253733b1fc8a9`
- delete-message-reverse-strong: `68acb5e6d30f79db7fa28bece7439df0cd9b48c47fde94878d5222cb49d0991b`

## unread-divider

The separator moves as one rigid assembly between two fixed message rows. State policy: gesture. Reviewed in sheet 4. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- unread-divider: `cccbba892553983c0b1df6392062bee5803595f14f62e760a84b8cfaa151dc2f`
- unread-divider-strong: `a6ea2f0cbcd4842da83def9eaaf1409b6faac78fb564ab2a79092ab22ae758e6`

## archive-chat

The message lowers into an open tray, with no invented tint surface across its open top. State policy: retained state and authored reverse. Reviewed in sheet 4. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- archive-chat: `ac830e72c8c29d21ee0f97b6799492685a6d94ea6896d366b5774e1f08f0b955`
- archive-chat-reverse: `cf3e561ca14b4c4ca20cd0d9dd35eee56532f8a36cf0374181c60bdd0e368207`
- archive-chat-strong: `dcf25d70025197763ae3a47dd18966d245074ecc86162f22f134a037704f85e6`
- archive-chat-reverse-strong: `ef4a39650301c241d7e890c6cd8480f586cab82ec3a2cda883c20014249c11bd`

## disappearing-message

A timer-setting gesture transfers sand; Strong adds a clear timer tilt. It does not delete a message. State policy: gesture. Reviewed in sheet 4. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- disappearing-message: `ff982b299b6dfbbcd730dd511f89d3aefb480aaaa57c762ced1eb547edb58c8c`
- disappearing-message-strong: `dacc27d1c9171f596971ff3030fe9b1559689d30392fde4ba39ab39959b29e7c`

## chat-search

The lens and its attached handle sweep together; Strong doubles the useful scan travel. State policy: gesture. Reviewed in sheet 4. Source, token, frame-sweep, boundary, style and native completion checks passed. No unresolved geometry screening flags.

- chat-search: `8654976a00b0bc6b0f37370b4898728aeebd5004c65386a336c27b63e8fe5ed7`
- chat-search-strong: `9ecf2703c9b00a2fcc7e7d0eb8d9fbe3bc9ced877c80f3888fd1298f0f0f41e3`

