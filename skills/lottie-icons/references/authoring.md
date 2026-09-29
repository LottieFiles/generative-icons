# Missing icons

This reference is maintained by hand in `skills/lottie-icons/references/authoring.md`; it is not generated.

## 1. Search first

```sh
npx lottie-icons search <meaning>
npx lottie-icons search <synonym>
```

Search matches IDs, labels, intent and categories. Try synonyms and the underlying object (for example `trash` for "delete"). If a close icon exists, use it with `npx lottie-icons add <id>` and tell the user which ID you chose and why. Do not rename or redraw a released icon inside the app.

## 2. Request a new icon

If nothing fits, open an issue at https://github.com/LottieFiles/lottie-generative-icons/issues with the meaning, where it appears in the product, and its states (for example idle and completed). Meanwhile, use the nearest existing icon or a static placeholder.

## 3. Create it yourself

New icons are authored in a clone of the source repository, not inside the app:

```sh
git clone https://github.com/LottieFiles/lottie-generative-icons.git
cd lottie-generative-icons
npm run setup
```

Then follow that repository's `AGENTS.md` and the animated-icon-workflow skill in `.agents/skills/animated-icon-workflow/`. You can author and preflight source without the native exporter. Open a pull request; a maintainer runs the native export, reviews the rendered output and merges. The icon becomes available to apps in a later package release.

If the user has the Engine exporter configured, `npx lottie-icons author <set-slug> --repo <path-to-clone>` checks the exporter and scaffolds a new set in that clone.

Never copy app code, credentials or private product details into the source repository or the pull request.
