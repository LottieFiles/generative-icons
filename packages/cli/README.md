# @lottiefiles/generative-icons-cli

> **Superseded by [`gen-animated-icons`](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/cli.md).** Run `npx gen-animated-icons init --migrate` in a project set up by this package; the new CLI also creates icons your app is missing. This package is kept only for projects that already use it.

The `lottie-icons` command: set up Generative Animated Icons in your app, search the library and install only the icons you use.

## Install

```sh
npm install @lottiefiles/generative-icons @lottiefiles/generative-icons-runtime @lottiefiles/generative-icons-cli
```

## Usage

```sh
npx lottie-icons init            # detect the app, write lottie-icons.json and an agent skill
npx lottie-icons search heart    # find icons
npx lottie-icons add heart       # copy assets and descriptors, record hashes
npx lottie-icons diff            # show local changes to managed files
npx lottie-icons info            # project and library details
```

Pass `--cwd apps/web` in a monorepo. `add` never overwrites managed files you have edited. Commit `lottie-icons.json`, the installed files and `.lottie-icons/lock.json`.

Keep all `@lottiefiles/generative-icons*` packages on the same version.

## Docs

- [Install and use the packages](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/using-packages.md)
- [Create a missing icon](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/creating-icons.md)

## License

MIT. See [LICENSE](https://github.com/LottieFiles/gen-animated-icons/blob/main/LICENSE).
