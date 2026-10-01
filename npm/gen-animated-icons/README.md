# gen-animated-icons

Find, install and create animated icons in web, iOS and Android apps with your coding AI. Generative Animated Icons by [LottieFiles](https://lottiefiles.com): native Lottie icons with Light and Strong motion, Outline, Solid and Duotone styles.

```sh
npx gen-animated-icons init          # set up the app: config, agent skills, authoring kit
npx gen-animated-icons search heart  # find icons by meaning
npx gen-animated-icons add heart     # install the .lottie asset and its descriptor
```

When an icon is missing, your agent creates it in your project: `create`, then `check`, `build` and `preview`. Building exports on LottieFiles' hosted engine and needs a LottieFiles sign-in (`gen-animated-icons login`).

This package is a small Node shim. npm installs the binary for your machine (macOS or Linux, arm64 or x64) as an optional dependency, so don't install with `--omit=optional`. Set `GEN_ANIMATED_ICONS_BINARY` to run another build of the binary.

Docs: [the CLI reference](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/cli.md). License: MIT.
