# @lottiefiles/generative-icons-react

React and Next.js component for Generative Animated Icons.

## Install

```sh
npm install @lottiefiles/generative-icons-runtime @lottiefiles/generative-icons-react
npx gen-animated-icons init && npx gen-animated-icons add heart
```

Requires React 18 or 19.

## Usage

```jsx
'use client';
import { AnimatedIcon } from '@lottiefiles/generative-icons-react';
import icon, { src } from './gen-animated-icons/heart.mjs';

export function Favorite() {
  return <AnimatedIcon icon={icon} src={src} size={24} variant="outline" motion="strong" />;
}
```

Icons are decorative by default; give the parent button an accessible name, or pass `label` when the icon carries meaning on its own. Use the `state` prop for stateful icons.

Keep all `@lottiefiles/generative-icons*` packages on the same version.

## Docs

- [Install and use the packages](https://github.com/LottieFiles/gen-animated-icons/blob/main/docs/using-packages.md#react-and-nextjs)

## License

MIT. See [LICENSE](https://github.com/LottieFiles/gen-animated-icons/blob/main/LICENSE).
