# Install and use the packages

Add animated icons to any JavaScript app: install the packages, copy the icons you need with the CLI, and render them with the shared runtime or the React component.

| Package | Purpose |
| --- | --- |
| `@lottiefiles/generative-icons` | Per-icon descriptors, native dotLottie assets and catalog |
| `@lottiefiles/generative-icons-runtime` | Playback, paint, state, reduced motion and lifecycle |
| `@lottiefiles/generative-icons-react` | React and Next.js component |
| `@lottiefiles/generative-icons-cli` | `lottie-icons` command: project setup, search and asset installation |

Keep all four on the same version.

## Install

```sh
npm install @lottiefiles/generative-icons @lottiefiles/generative-icons-runtime @lottiefiles/generative-icons-cli
# React or Next.js:
npm install @lottiefiles/generative-icons-react

npx lottie-icons init
npx lottie-icons search heart
npx lottie-icons add heart
```

In a monorepo, pass `--cwd apps/web` to target the app.

`init` reads the app's package metadata and creates `lottie-icons.json` plus a local agent skill in `.agents/skills/lottie-icons/`. It does not edit AGENTS.md, global agent settings or components. If your app uses a base URL, include it in `publicPath`.

`add` copies native files to `public/lottie-icons/` and descriptors to `src/lottie-icons/` (or `lottie-icons/` without a `src` folder), and records hashes in `.lottie-icons/lock.json`. Commit the config, assets, descriptors and lock. `diff` reports local changes, and `add` refuses to overwrite modified managed files.

## Vanilla JavaScript

```js
import { createIcon } from '@lottiefiles/generative-icons-runtime';
import icon, { src } from './lottie-icons/heart.mjs';

const handle = await createIcon(document.querySelector('canvas'), {
  icon, src, color: '#008c76', style: 'outline', weight: 2, motion: 'light'
});
document.querySelector('button').addEventListener('click', () => handle.play());
// On removal: handle.destroy();
```

Without the CLI, import a single descriptor from `@lottiefiles/generative-icons/icons/heart` and resolve `@lottiefiles/generative-icons/assets/heart.lottie` with your bundler. A descriptor import does not load the catalog or any asset bytes.

## React and Next.js

```jsx
'use client';
import { AnimatedIcon } from '@lottiefiles/generative-icons-react';
import icon, { src } from './lottie-icons/heart.mjs';

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
| Weight | `1.5`, `2`, `2.5` (outline only) |
| Motion | `light`, `strong` |
| Theme | `light`, `dark` |
| Reduced motion | Follows the OS preference; override with a boolean |

- **State:** read `startState` and `endState` from the descriptor. Use `handle.setState(name)` or the React `state` prop to keep app state. `play('reverse')` runs the authored reverse action.
- **Preview:** `preview()` is for demos and may reset a stateful icon afterwards.
- **Reduced motion** keeps the chosen state without playback. Continuous icons stop after one cycle unless you start another.
- `handle.update()` changes paint and motion without reloading. `stop()` returns to the rest pose. Pass an `AbortSignal` while loading and call `destroy()` on cleanup.
- `wasmUrl` points to a self-hosted WASM file from the pinned dotLottie player.

**Shared defaults:** edit `defaults` in `lottie-icons.json` and run `add` again. Import `defaults` from the generated `lottie-icons/settings.mjs` and spread it into runtime options (map `style` to `variant` in React).

Runtime color settings are applied at playback and are not baked into the file. Downloaded native files keep their authored themes and style slots.

## Missing an icon?

[Request it](https://github.com/LottieFiles/lottie-generative-icons/issues/new?template=icon_request.yml), or create it with your coding AI and open a pull request. See [creating icons](creating-icons.md).
