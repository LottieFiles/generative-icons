# @lottiefiles/generative-icons-runtime

Playback runtime for Generative Icons: paint, Light/Strong motion, state, reduced motion and lifecycle, built on dotLottie Web.

## Install

```sh
npm install @lottiefiles/generative-icons-runtime
npx generative-icons init && npx generative-icons add heart
```

## Usage

```js
import { createIcon } from '@lottiefiles/generative-icons-runtime';
import icon, { src } from './generative-icons/heart.mjs';

const handle = await createIcon(document.querySelector('canvas'), {
  icon, src, color: '#008c76', style: 'outline', weight: 2, motion: 'light'
});
button.addEventListener('click', () => handle.play());
// later: handle.destroy();
```

The handle exposes `play()`, `setState()`, `update()`, `stop()` and `destroy()`. Reduced motion follows the OS preference by default.

## Animated SVG

`@lottiefiles/generative-icons-runtime/svg` turns an icon into an SVG that browsers animate by themselves (SMIL): no player, script or WASM.

```js
import { dotLottieToSvg } from '@lottiefiles/generative-icons-runtime/svg';

const svg = await dotLottieToSvg(icon, bytes, { style: 'outline', weight: 2, trigger: 'auto' });
```

`trigger: 'auto'` toggles stateful icons on click, loops loaders and plays the rest on hover. `npx generative-icons export <id>` writes the same SVG.

Keep all `@lottiefiles/generative-icons*` packages on the same version.

## Docs

- [Install and use the packages](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/using-packages.md)
- [Appearance and behavior options](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/using-packages.md#appearance-and-behavior)

## License

MIT. See [LICENSE](https://github.com/LottieFiles/gen-animated-icons/blob/main/LICENSE).
