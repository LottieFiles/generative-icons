# Generative Animated Icons

**Animated icons. Ready to use. Built to extend.** By [LottieFiles](https://lottiefiles.com).

An open family of native Lottie icons with Light and Strong motion, Outline, Solid and Duotone styles, and editable vector source. Use the icons as they are, or give your coding AI the family rules and reference geometry to create a missing one.

[Use in your app](#use-icons-in-your-app) · [Preview locally](#preview-and-download-locally) · [Create an icon](#create-a-missing-icon-with-your-ai) · [Docs](docs/README.md) · [Contribute](CONTRIBUTING.md)

## What's included

- **1,005 icons in 58 sets**, from essentials and arrows to AI, commerce, weather and smart home.
- **Light and Strong motion** for every icon, with explicit reverse or reset actions for stateful icons.
- **Outline, Solid and Duotone styles**, three outline weights (1.5, 2, 2.5), light and dark themes, and a recolorable primary slot.
- **Native dotLottie and Lottie JSON** on a 32 × 32 grid at 60 fps.
- **npm packages**: per-icon data, a shared runtime, a React adapter and a CLI.
- **Editable source**, an AI operating guide, family examples and validation tools.

Light and Strong describe motion intensity. Light, Regular and Bold describe outline weight. The two are independent.

## Use icons in your app

```sh
npm install @lottiefiles/generative-icons @lottiefiles/generative-icons-runtime @lottiefiles/generative-icons-cli
npx lottie-icons init
npx lottie-icons add heart
```

```jsx
'use client';
import { AnimatedIcon } from '@lottiefiles/generative-icons-react';
import icon, { src } from './lottie-icons/heart.mjs';

export const Favorite = () => <AnimatedIcon icon={icon} src={src} size={24} />;
```

Add `@lottiefiles/generative-icons-react` for React or Next.js. See [installing the packages](docs/using-packages.md) for vanilla JavaScript, state handling and accessibility.

## Preview and download locally

You need Python 3. With Node installed, `npm run dev` starts the gallery; without it, run `python3 scripts/serve.py`.

```sh
git clone https://github.com/LottieFiles/lottie-generative-icons.git
cd lottie-generative-icons
npm run dev
```

Open **http://127.0.0.1:4189/icons/01/**, pick an icon, style, weight and motion profile, and download its dotLottie file. See [using icons](docs/using-icons.md) to render the files with any dotLottie player.

## Create a missing icon with your AI

Point your coding agent at [AGENTS.md](AGENTS.md) and the [creation guide](docs/creating-icons.md). It searches the library, picks family neighbors, draws named parts, authors Light and Strong motion, and runs the checks.

```sh
npm ci && npm run setup
npm run icons:new -- landscaping --name "Landscaping"
```

You can author and validate source without the native exporter. Open a pull request and a maintainer runs the native export and rendered review. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Documentation

| Guide | For |
| --- | --- |
| [Installing the packages](docs/using-packages.md) | App developers |
| [Using icons](docs/using-icons.md) | Direct file use, styles and accessibility |
| [Creating icons](docs/creating-icons.md) | Designers, developers and coding agents |
| [Drawing](docs/design-principles.md) and [motion](docs/motion-principles.md) principles | The family rules |
| [Development](docs/development.md) and [verification](docs/verification.md) | Contributors |
| [All documentation](docs/README.md) | Everything else |

## Project layout

```text
packages/          npm packages: icons, runtime, react, cli
motion/            Editable icon source, builders and set verification
prototype/         Local gallery, native assets and bundled player
.agents/skills/    AI authoring workflow and review rules
skills/            Consumer skill installed by the CLI
docs/              Usage, design, development and maintainer guides
scripts/           Preview, build and repository checks
```

## License

Code, documentation and icon artwork are released under the [MIT License](LICENSE). Bundled dependencies keep their own licenses; see [third-party notices](THIRD_PARTY_NOTICES.md).
