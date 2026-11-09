---
name: lottie-icons
description: Find, install and customize Generative Animated Icons (by LottieFiles) in an application, and handle requests for icons that do not exist yet. Use for this library's icon tasks, not general illustration.
---

# Generative Animated Icons in an app

## Quickstart

```sh
npx lottie-icons info            # framework, configured paths, local changes
npx lottie-icons search <meaning> # find released icons by ID, label, intent or category
npx lottie-icons add <id>         # copy the descriptor and native asset into the app
npx lottie-icons diff             # list locally modified managed files
```

In a monorepo, add `--cwd <app-folder>` to each command. Use the CLI installed in the project (`npx`, `pnpm exec` or `yarn`); do not run an unpinned remote package.

Then render the icon with `createIcon` from `@lottiefiles/generative-icons-runtime` or `AnimatedIcon` from `@lottiefiles/generative-icons-react`, passing the generated descriptor and its `src` together. Examples and state behavior: [runtime.md](references/runtime.md).

## Rules

- Read the app's own agent instructions and `lottie-icons.json` first.
- Use stable icon IDs from `search`; never invent filenames.
- `add` refuses to overwrite local edits. Do not bypass it; inspect `diff` and resolve deliberately.
- Map app theme tokens to six-digit hex colors. Choose `outline`, `solid` or `duotone`, and a weight of 1.5, 2 or 2.5 (outline only).
- Light and Strong are distinct authored motions. Do not simulate one by changing speed.
- Keep the accessible name on the button; decorative icons need no label.
- Respect reduced motion and retained app state. Destroy imperative instances when their component unmounts.

## Missing icons

Search synonyms and pick the nearest existing icon. If none fits, read [authoring.md](references/authoring.md): request the icon with a GitHub issue, or create it in a clone of the source repository and open a pull request. Icons are never authored inside the app.
