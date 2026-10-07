# Use installed icons

`add` writes the asset to the config's `assets` folder and a descriptor to `descriptors`. The descriptor lists the icon's two motions: `motions[].name` is the animation id inside the `.lottie` (`<id>` is Light, `<id>-strong` is Strong), and a stateful icon has `startState`, `endState` and a `reverse` action per motion. Commit the config, assets, descriptors and `.generative-icons/lock.json`.

## Web

Install the runtime (and the React adapter for React or Next.js) with the project's package manager:

```sh
npm install @lottiefiles/generative-icons-runtime @lottiefiles/generative-icons-react
```

Until launch these packages are not on npm. Build them in a library checkout with `generative-icons library packages`, then install them from its `packages/runtime` and `packages/react` folders (`npm install <checkout>/packages/runtime <checkout>/packages/react`).

`src/generative-icons/<id>.mjs` exports the descriptor (`default`) and `src`, the URL of the asset under `publicPath`. `settings.mjs` exports the project `defaults`; edit `generative-icons.json`, not this file, then rerun `add`.

```js
import { createIcon } from '@lottiefiles/generative-icons-runtime';
import icon, { src } from './generative-icons/heart.mjs';
import { defaults } from './generative-icons/settings.mjs';

const handle = await createIcon(canvas, { icon, src, ...defaults });
button.addEventListener('click', () => handle.play());
// When the element is removed: handle.destroy();
```

```jsx
'use client';
import { AnimatedIcon } from '@lottiefiles/generative-icons-react';
import icon, { src } from './generative-icons/heart.mjs';

export function Favorite() {
  return <AnimatedIcon icon={icon} src={src} size={24} color="#008c76" variant="outline" motion="light" />;
}
```

React takes `variant` for `style`. Stateful icons follow app state through the React `state` prop or `handle.setState(name)`; `play('reverse')` runs the authored reverse. The runtime handles reduced motion and any weight from 1 to 3.

## iOS (SwiftUI)

Add the Swift package `https://github.com/LottieFiles/dotlottie-ios`. Assets land in `<Target>/GenerativeIcons/`; with an Xcode 16 synchronized folder they are bundled automatically, otherwise add the folder to the target's resources.

```swift
import SwiftUI
import DotLottie

struct HeartIcon: View {
  @State private var icon: DotLottieAnimation = {
    var config = AnimationConfig()
    config.animationId = "heart"          // "heart-strong" for Strong
    return DotLottieAnimation(fileName: "heart", config: config)
  }()

  var body: some View {
    icon.view().frame(width: 32, height: 32).onTapGesture { icon.play() }
  }
}
```

## Android (Compose)

Add `implementation("com.lottiefiles:dotlottie-android:0.16.0")`. Assets land in `app/src/main/assets/`.

```kotlin
@Composable
fun HeartIcon() {
  val controller = remember { DotLottieController() }
  DotLottieAnimation(
    modifier = Modifier.size(32.dp).clickable { controller.play() },
    source = DotLottieSource.Asset("heart.lottie"),
    controller = controller,
  )
}
```

## Android (Views)

`init` reports `framework: views` for an app without Compose. Use the same dependency and assets with the `DotLottieAnimation` view:

```xml
<com.lottiefiles.dotlottie.core.widget.DotLottieAnimation
    android:id="@+id/heart_icon"
    android:layout_width="32dp"
    android:layout_height="32dp" />
```

```kotlin
val icon = findViewById<DotLottieAnimation>(R.id.heart_icon)
icon.load(
  Config.Builder()
    .source(DotLottieSource.Asset("heart.lottie"))
    .autoplay(false)
    .build()
)
icon.setOnClickListener { icon.play() }
```

On both Android APIs, files open on Strong. For Light, call `loadAnimation("<id>")` (on the controller, or on the view) in the `onLoad` listener; load a motion's `reverse.name` to play a stateful icon back.

## Paint on native players

Set slots once per load, in the player's load callback, and create a new player to change them: the current native cores can crash when a set slot is changed again.

| Setting | Slot and value |
| --- | --- |
| Color | `color.primary` = the hex color as RGB 0–1 |
| Outline weight 1.5 / 2 / 2.5 | `style.outline.light` / `.regular` / `.bold` = 100, the other two 0 (files default to regular) |
| Solid | `style.solid` = 100 and all three outline slots 0 |
| Duotone | outline slots as for Outline, plus `style.tint` = 18 |

Native players ignore `style.outline.width`, so other weights snap to the nearest of 1.5, 2 and 2.5. To ship an exact in-between weight, install with `generative-icons add <id> --bake-weight <w>`.
