# Use an icon file

For app integration, `generative-icons add` and the [npm packages](using-packages.md) are the easiest path. This page covers using the native files directly.

## Choose and download

`generative-icons search <meaning>` finds icons, `generative-icons preview <id>` renders an icon's motions to PNG contact sheets, and `generative-icons add <id>` installs its dotLottie file. In a checkout of the source repository, the native files live under `prototype/icons/NN/assets/` (one dotLottie file per icon and profile, plus one-action Lottie JSON files), and the local gallery (`npm run dev`) downloads the selected profile from each card.

A dotLottie file contains the icon's sibling actions and themes; its manifest selects the initial action. A loose JSON file holds a single action. For stateful controls, use the authored reverse or reset action instead of playing backwards.

## Render

Use any compatible Lottie or dotLottie player. The gallery's [app.js](https://github.com/LottieFiles/generative-icons/blob/main/prototype/icons/01/app.js) is a working dotLottie Web integration, and the shared [controller](https://github.com/LottieFiles/generative-icons/blob/main/prototype/icons/03/controller.mjs) handles interaction and state. The bundled player is pinned to dotLottie Web 0.80.0.

Raw Lottie JSON support does not imply dotLottie theme or slot support.

## Customize

Each dotLottie file exposes a `color.primary` slot and these style slots:

- `style.outline.light`
- `style.outline.regular`
- `style.outline.bold`
- `style.solid`
- `style.tint` (Duotone)
- `style.outline.width` (stroke-width token on every Regular outline stroke, for any weight from 1 to 3; the runtime bridges players that do not apply it yet)

Outline weight and motion intensity are independent. Duotone intentionally matches Outline for shapes without an enclosed surface.

## Interaction and accessibility

- Treat the animation as a supplement to the control's accessible label.
- Show a static, useful pose when reduced motion is requested.
- Keep stateful actions at their resulting state; play the explicit reverse when the product state changes.
- Gallery looping is a preview, not a required product interaction.
- Check legibility at real size beside text and neighboring controls, and load only the icons and variants your app needs.
