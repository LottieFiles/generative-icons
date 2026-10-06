# @lottiefiles/generative-icons

1,141 animated icons from Generative Animated Icons by LottieFiles: native dotLottie assets, one descriptor per icon, and the catalog. Light and Strong motion; Outline, Solid and Duotone styles.

## Install

```sh
npm install @lottiefiles/generative-icons-runtime
npx gen-animated-icons init && npx gen-animated-icons add heart
```

The `gen-animated-icons` CLI copies only the icons you use into your app. Install this package when you want to import icons straight from `node_modules`; the CLI also reads it as its library when it is installed.

## Usage

```js
import { createIcon } from '@lottiefiles/generative-icons-runtime';
import icon, { src } from './gen-animated-icons/heart.mjs'; // written by `gen-animated-icons add`

const handle = await createIcon(document.querySelector('canvas'), { icon, src, motion: 'strong' });
handle.play();
```

Without the CLI, import one descriptor from `@lottiefiles/generative-icons/icons/heart` and resolve `@lottiefiles/generative-icons/assets/heart.lottie` with your bundler. A descriptor import loads only that icon's metadata.

Icons reanimated as specs also ship their editable spec, `@lottiefiles/generative-icons/specs/<id>.icon.json`; `gen-animated-icons create <new-id> --from <id>` starts a new icon from one.

Keep all `@lottiefiles/generative-icons*` packages on the same version.

## Docs

- [Install and use the packages](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/using-packages.md)
- [Package contract](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/package-contract.md)
- [Create a missing icon](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/creating-icons.md)

## License

MIT, for code and icon artwork. See [LICENSE](https://github.com/LottieFiles/gen-animated-icons/blob/main/LICENSE) and [third-party notices](https://github.com/LottieFiles/gen-animated-icons/blob/main/THIRD_PARTY_NOTICES.md).
