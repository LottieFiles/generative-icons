# @lottiefiles/generative-icons

965 animated icons from Generative Animated Icons by LottieFiles: native dotLottie assets, one descriptor per icon, and the catalog. Light and Strong motion; Outline, Solid and Duotone styles.

## Install

```sh
npm install @lottiefiles/generative-icons @lottiefiles/generative-icons-runtime @lottiefiles/generative-icons-cli
npx lottie-icons init && npx lottie-icons add heart
```

## Usage

```js
import { createIcon } from '@lottiefiles/generative-icons-runtime';
import icon, { src } from './lottie-icons/heart.mjs'; // written by `lottie-icons add`

const handle = await createIcon(document.querySelector('canvas'), { icon, src, motion: 'strong' });
handle.play();
```

Without the CLI, import one descriptor from `@lottiefiles/generative-icons/icons/heart` and resolve `@lottiefiles/generative-icons/assets/heart.lottie` with your bundler. A descriptor import loads only that icon's metadata.

Keep all `@lottiefiles/generative-icons*` packages on the same version.

## Docs

- [Install and use the packages](https://github.com/LottieFiles/lottie-generative-icons/blob/main/docs/using-packages.md)
- [Package contract](https://github.com/LottieFiles/lottie-generative-icons/blob/main/docs/package-contract.md)
- [Create a missing icon](https://github.com/LottieFiles/lottie-generative-icons/blob/main/docs/creating-icons.md)

## License

MIT, for code and icon artwork. See [LICENSE](https://github.com/LottieFiles/lottie-generative-icons/blob/main/LICENSE) and [third-party notices](https://github.com/LottieFiles/lottie-generative-icons/blob/main/THIRD_PARTY_NOTICES.md).
