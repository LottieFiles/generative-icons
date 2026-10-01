# Create an icon

When the library has no icon for your meaning, your coding AI can create one in your own project with the `gen-animated-icons` CLI. You can also author the source yourself; the rules and checks are the same either way. The new icon is drawn with the family's authoring kit, exported as native Lottie, verified and installed beside the library icons you use.

**You need a LottieFiles sign-in to build.** Export runs on LottieFiles' hosted engine. Run `gen-animated-icons login` once in your terminal; agents never sign in for you. The hosted engine is internal until launch, so building may not work for your account yet. Everything up to `build`, and `preview` of library icons, works without signing in.

## 1. Set up the project

```sh
npx gen-animated-icons init
```

`init` installs the agent skill (`.claude/skills/gen-animated-icons/` and `.agents/skills/gen-animated-icons/`) and a read-only copy of the authoring kit in `gen-animated-icons/library/`. Your icons' source lives in `gen-animated-icons/icons/`. The commands below assume `gen-animated-icons` is on your `PATH`; otherwise prefix them with `npx`.

## 2. Search and choose references

```sh
gen-animated-icons search "package return"
gen-animated-icons context "package return" --neighbor return-package --neighbor delivery-truck
```

Search ranks by meaning, not just spelling: similar names and opposite directions can mean different things. Use an existing icon when one fits. Otherwise `context` prints the reference packet: the family contract plus the chosen neighbors' timing and initial geometry. Read it for construction and timing; never copy the reference art.

## 3. Give your AI a complete brief

```text
Create an icon for [meaning] used in [product context].
Initial state: [...]. Completed state: [...]. Reverse/reset: [...].
Use these existing neighbors: [...].
Preserve: [...]. Avoid: [...].
Follow the gen-animated-icons skill. Draw editable named parts with pivots,
and write distinct Light and Strong motion briefs before keyframing.
Run check, build and preview; open every preview sheet and fix what you see.
Report any check you could not run.
```

The installed skill carries the authoring kit API, the family rules and the review checklist, so the agent needs no access to this repository.

## 4. Author, check, build and review

```sh
gen-animated-icons create return-box --label "Return box" --intent "The box slides back to the sender" --neighbor return-package
# draw and animate in gen-animated-icons/icons/batches/return-box.mjs
gen-animated-icons check return-box     # offline source checks
gen-animated-icons build return-box     # hosted export, package, verify, preview sheets, install
gen-animated-icons preview return-box   # re-render the sheets after a change
```

- `create --from <library-id>` starts from a library icon's working source instead of a template. It needs a checkout of this repository (`--library <checkout>`), because the hosted library carries no sources.
- `check` samples every part at every frame, in both directions and at every outline weight, and runs the checks verification applies after export: holds after rest, gestures that end where they start, matching Light and Strong endpoints, and reverse actions that undo their action. Fix every error before building.
- `build` runs the same source checks first and exports nothing while one fails. It installs only icons that verify, and writes preview sheets to `.gen-animated-icons/previews/<id>/`.
- Look at every sheet: each style and weight, both themes, and Light against Strong at the same elapsed times. Name each flaw, fix it in the batch file, then check, build and preview again. Stop when a round finds nothing new or the same flaw repeats; report what remains.

Review the silhouette, exposed surfaces, optical spacing and motion against the [drawing](design-principles.md) and [motion](motion-principles.md) principles. A passing check proves the file is valid, not that the icon is good.

## 5. Use it, or offer it to the library

A built icon is installed like a library icon and stays in your project. To offer it to the library:

```sh
gen-animated-icons submit return-box --license MIT --dry-run --output return-box.zip   # inspect the bundle
gen-animated-icons submit return-box --license MIT                                     # send it for review
```

`submit` accepts only verified builds of the current source, and every submission is reviewed. The submission service goes live at launch.

## Library sets in this repository

Maintainers add whole sets to the library with `gen-animated-icons library new-set`, `build`, `verify`, `verify-visual` and `register`; see [development](development.md#author-and-build-a-library-set). The family workflow for those sets is the [AI operating guide](https://github.com/LottieFiles/lottie-generative-icons/blob/main/prototype/icons/guide/ai-operating-guide.md), which bundles the canonical rules in one file.
