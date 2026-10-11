# Generative Icons

**Animated icons. Ready to use. Built to extend.** By [LottieFiles](https://lottiefiles.com).

An open family of native Lottie icons with Light and Strong motion, Outline, Solid and Duotone styles, and editable vector source. Install the icons you need into a web, iOS or Android app, or let your coding AI create a missing one in your own project with the `generative-icons` CLI.

[Use icons](#use-icons-in-your-app) · [Create an icon](#create-a-missing-icon-with-your-ai) · [CLI reference](docs/cli.md) · [Docs](docs/README.md) · [Contribute](CONTRIBUTING.md)

## What's included

- **1,161 icons in 59 sets**, from essentials and arrows to AI, commerce, weather, smart home and emoji.
- **Light and Strong motion** for every icon, with explicit reverse or reset actions for stateful icons.
- **Outline, Solid and Duotone styles**, three outline weights (1.5, 2, 2.5), light and dark themes, and a recolorable primary slot.
- **Native dotLottie and Lottie JSON** on a 32 × 32 grid at 60 fps.
- **`generative-icons`**, one binary for macOS and Linux that finds, installs, creates, checks and previews icons, with an agent skill and an MCP server.
- **npm packages** for the web: per-icon data, a shared runtime and a React adapter.

Light and Strong describe motion intensity. Light, Regular and Bold describe outline weight. The two are independent.

## Use icons in your app

Run these in your app folder:

```sh
npx generative-icons init          # detect the app; write the config, agent skills and the authoring kit
npx generative-icons search heart  # find icons by meaning
npx generative-icons add heart     # install the .lottie asset and its descriptor
```

`init` detects web apps (`package.json`), iOS apps (an Xcode project or Swift package) and Android apps (a Gradle app module). On the web, render the icon with the runtime or the React component:

```jsx
'use client';
import { AnimatedIcon } from '@lottiefiles/generative-icons-react';
import icon, { src } from './generative-icons/heart.mjs';

export const Favorite = () => <AnimatedIcon icon={icon} src={src} size={24} />;
```

iOS and Android apps play the installed `.lottie` files with dotLottie for iOS and Android. See [installing the packages](docs/using-packages.md) for the web and the [CLI reference](docs/cli.md) for every command and platform.

`npx` runs the npm wrapper, which installs the prebuilt binary for your machine (macOS or Linux, arm64 or x64). Homebrew and a curl installer are planned.

## Create a missing icon with your AI

`init` installs a `generative-icons` skill for Claude Code (`.claude/skills/`) and other agents (`.agents/skills/`). Ask your agent for the icon you need; the skill walks it through this loop (prefix each command with `npx` unless the binary is on your `PATH`):

```sh
generative-icons context "mailbox: a letter drops in"   # family rules and the nearest references
generative-icons create mailbox --label "Mailbox"       # scaffold the icon's spec in your project
# the agent draws named parts and authors Light and Strong motion in generative-icons/icons/mailbox.icon.json
generative-icons check mailbox                          # offline checks, in milliseconds
generative-icons draft mailbox                          # a review sheet from the engine, nothing stored
generative-icons build mailbox                          # export, package, verify, preview and install
generative-icons preview mailbox                        # contact sheets the agent opens and reviews
```

**`draft` and `build` need a LottieFiles sign-in.** Native Lottie export runs on LottieFiles' hosted engine, so run `generative-icons login` once in your terminal. `draft` and `build` send the icon's spec (its shapes and motion) to the engine; your app's code stays on your machine. The hosted engine is internal until launch, so building may not work for your account yet. Searching, installing, `check` and `preview` need no sign-in.

Icons you create stay in your project. To offer one to the library, `generative-icons submit <id>` sends the verified build for review. See [creating icons](docs/creating-icons.md).

## Before launch

The npm packages, the hosted library behind `search` and `add`, and the submission service go live at launch. Until then, build the CLI from this repository (see [development](docs/development.md)), build the packages in the checkout with `generative-icons library packages`, and point the CLI at it with `--library <checkout>`. Details: [known limitations](docs/limitations.md).

## Documentation

| Guide | For |
| --- | --- |
| [CLI reference](docs/cli.md) | Every `generative-icons` command, file and setting |
| [Installing the packages](docs/using-packages.md) | Web app developers |
| [Using icons](docs/using-icons.md) | Direct file use, styles and accessibility |
| [Creating icons](docs/creating-icons.md) | Developers and coding agents |
| [Drawing](docs/design-principles.md) and [motion](docs/motion-principles.md) principles | The family rules |
| [Development](docs/development.md) and [verification](docs/verification.md) | Contributors and maintainers |
| [All documentation](docs/README.md) | Everything else |

## Project layout

```text
crates/            The generative-icons CLI (Rust workspace) and its embedded agent skill
npm/               npm wrapper and per-platform packages for the CLI binary
packages/          Web npm packages: icons, runtime, react (and the retired lottie-icons CLI)
motion/            Editable icon source, family contract and the legacy JS/Python pipeline
prototype/         Local gallery, native assets and bundled player
.agents/skills/    Maintainer workflow and review rules for this repository
docs/              Usage, design, development and maintainer guides
scripts/           Gallery, package builds and repository checks
```

## License

Code, documentation and icon artwork are released under the [MIT License](LICENSE). Bundled dependencies keep their own licenses; see [third-party notices](THIRD_PARTY_NOTICES.md). Each `generative-icons` release archive and npm platform package also carries `THIRD_PARTY_LICENSES.txt`, the license texts of everything the binary links.
