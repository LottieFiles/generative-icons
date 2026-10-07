# Install and use the packages

Add animated icons to any JavaScript app: install only the icons you need with the `generative-icons` CLI, and render them with the shared runtime or the React component.

| Package | Purpose |
| --- | --- |
| `generative-icons` | The CLI: project setup, search, icon installation, and creating icons your app is missing |
| `@lottiefiles/generative-icons-runtime` | Playback, paint, state, reduced motion and lifecycle |
| `@lottiefiles/generative-icons-react` | React and Next.js component |
| `@lottiefiles/generative-icons` | Per-icon descriptors, native dotLottie assets and the catalog, for apps that import icons without the CLI |

Keep the `@lottiefiles/generative-icons*` packages on the same version.

## Install

### 1. Install the runtime

```sh
npm install @lottiefiles/generative-icons-runtime
# React or Next.js, also:
npm install @lottiefiles/generative-icons-react
```

### 2. Set up the project

```sh
npx generative-icons init
```

In a monorepo, pass `--cwd apps/web` to target the app. If your app uses a base URL, include it in `publicPath` in `generative-icons.json`.

### 3. Add icons

```sh
npx generative-icons search heart
npx generative-icons add heart
```

Commit `generative-icons.json`, the added files and `.generative-icons/lock.json`.

### What gets installed

- `init` reads the app's package metadata and creates `generative-icons.json`, the agent skill in `.claude/skills/generative-icons/` and `.agents/skills/generative-icons/`, and a copy of the authoring kit. It does not edit `AGENTS.md`, global agent settings or components.
- `add` copies native files to `public/generative-icons/` and descriptors to `src/generative-icons/` (or `generative-icons/` without a `src` folder), and records their hashes in `.generative-icons/lock.json`.
- `diff` reports local changes, `add` refuses to overwrite modified managed files, and `remove` uninstalls icons whose files are unedited.
- A project set up before the rename (with `gen-animated-icons.json`) or by the earlier `lottie-icons` command moves over with `npx generative-icons init --migrate`; its icon files keep their paths.

## Vanilla JavaScript

```js
import { createIcon } from '@lottiefiles/generative-icons-runtime';
import icon, { src } from './generative-icons/heart.mjs';

const handle = await createIcon(document.querySelector('canvas'), {
  icon, src, color: '#008c76', style: 'outline', weight: 2, motion: 'light'
});
document.querySelector('button').addEventListener('click', () => handle.play());
// On removal: handle.destroy();
```

Without the CLI, install `@lottiefiles/generative-icons`, import a single descriptor from `@lottiefiles/generative-icons/icons/heart` and resolve `@lottiefiles/generative-icons/assets/heart.lottie` with your bundler. A descriptor import does not load the catalog or any asset bytes.

## React and Next.js

```jsx
'use client';
import { AnimatedIcon } from '@lottiefiles/generative-icons-react';
import icon, { src } from './generative-icons/heart.mjs';

export function Favorite() {
  return <AnimatedIcon icon={icon} src={src} size={24} color="#008c76" />;
}
```

Components that handle events in Next.js need `'use client'`. The adapter loads the player after mount and destroys it on unmount; server output is a static canvas.

**Accessibility:** icons are decorative by default and hidden from assistive technology, so the parent button must provide the accessible name. Set `label` only when the icon conveys information without visible text.

## Appearance and behavior

| Setting | Values |
| --- | --- |
| Size | CSS pixels, default `32` |
| Color | Six-digit hex; resolve theme tokens before passing them |
| Style | `outline`, `solid`, `duotone` (React prop: `variant`) |
| Weight | Any value from `1` to `3` (outline only). `1.5`, `2` and `2.5` are the authored Light, Regular and Bold layers; other values render the Regular geometry at that stroke width. The files carry a `style.outline.width` token for it; until dotLottie players apply stroke-width tokens, the runtime writes the width into the file before loading it |
| Motion | `light`, `strong` |
| Theme | `light`, `dark` |
| Reduced motion | Follows the OS preference; override with a boolean |

- **State:** read `startState` and `endState` from the descriptor. Use `handle.setState(name)` or the React `state` prop to keep app state. `play('reverse')` runs the authored reverse action.
- **Preview:** `preview()` is for demos and may reset a stateful icon afterwards.
- **Reduced motion** keeps the chosen state without playback. Continuous icons stop after one cycle unless you start another.
- `handle.update()` changes paint and motion without reloading. `stop()` returns to the rest pose. Pass an `AbortSignal` while loading and call `destroy()` on cleanup.
- `wasmUrl` points to a self-hosted WASM file from the pinned dotLottie player.

**Shared defaults:** edit `defaults` in `generative-icons.json` and run `add` again. Import `defaults` from the generated `generative-icons/settings.mjs` and spread it into runtime options (map `style` to `variant` in React).

Runtime color settings are applied at playback and are not baked into the file. Downloaded native files keep their authored themes and style slots.

## iOS and Android

The same commands work in iOS and Android apps: `add` installs the `.lottie` file into the app's assets and a JSON descriptor into `generative-icons/descriptors/`. Play the files with dotLottie for iOS or Android; the installed skill's runtime reference has SwiftUI, Compose and Views examples. Native players show the 1.5, 2 and 2.5 weights; `add --bake-weight <w>` bakes any other weight into the file.

## Missing an icon?

Create it with your coding AI in your own project (`generative-icons create`, then `build`; building needs a LottieFiles sign-in), or [request it](https://github.com/LottieFiles/gen-animated-icons/issues/new?template=icon_request.yml). See [creating icons](creating-icons.md).
