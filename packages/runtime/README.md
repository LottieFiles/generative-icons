# @lottiefiles/generative-icons-runtime

Playback runtime for Generative Animated Icons: paint, Light/Strong motion, state, reduced motion and lifecycle, built on dotLottie Web.

## Install

```sh
npm install @lottiefiles/generative-icons @lottiefiles/generative-icons-runtime @lottiefiles/generative-icons-cli
npx lottie-icons init && npx lottie-icons add heart
```

## Usage

```js
import { createIcon } from '@lottiefiles/generative-icons-runtime';
import icon, { src } from './lottie-icons/heart.mjs';

const handle = await createIcon(document.querySelector('canvas'), {
  icon, src, color: '#008c76', style: 'outline', weight: 2, motion: 'light'
});
button.addEventListener('click', () => handle.play());
// later: handle.destroy();
```

The handle exposes `play()`, `setState()`, `update()`, `stop()` and `destroy()`. Reduced motion follows the OS preference by default.

Keep all `@lottiefiles/generative-icons*` packages on the same version.

## Docs

- [Install and use the packages](https://github.com/LottieFiles/lottie-generative-icons/blob/main/docs/using-packages.md)
- [Appearance and behavior options](https://github.com/LottieFiles/lottie-generative-icons/blob/main/docs/using-packages.md#appearance-and-behavior)

## License

MIT. See [LICENSE](https://github.com/LottieFiles/lottie-generative-icons/blob/main/LICENSE).
