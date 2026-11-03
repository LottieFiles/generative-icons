# Create an icon

When the library has no icon for your meaning, your coding AI can create one in your own project with the `generative-icons` CLI. You can also write it yourself; the rules and checks are the same either way. A new icon is a spec: a JSON file of shapes, roles, rigs and eased keys, which Creator MCP, LottieFiles' hosted engine, exports as native Lottie in one call. It is verified and installed beside the library icons you use.

**You need a LottieFiles sign-in to build.** Export runs on [Creator MCP](https://lottiefiles.com/mcp), so run `generative-icons login` once in your terminal; agents never sign in for you. Everything up to `build`, and `preview` of library icons, works without signing in.

## 1. Set up the project

```sh
npx generative-icons init
```

`init` installs the agent skill (`.claude/skills/generative-icons/` and `.agents/skills/generative-icons/`) and a read-only copy of the authoring kit in `generative-icons/library/` (for JS batches). Your icons' specs live in `generative-icons/icons/`. The commands below assume `generative-icons` is on your `PATH`; otherwise prefix them with `npx`.

## 2. Search and choose references

```sh
generative-icons search "package return"
generative-icons context "package return" --neighbor return-package --neighbor delivery-truck
```

Search ranks by meaning, not just spelling: similar names and opposite directions can mean different things. Use an existing icon when one fits. Otherwise `context` prints the reference packet: the family contract plus the chosen neighbors' timing and initial geometry. Read it for construction and timing; never copy the reference art.

## 3. Give your AI a complete brief

```text
Create an icon for [meaning] used in [product context].
Initial state: [...]. Completed state: [...]. Reverse/reset: [...].
Use these existing neighbors: [...].
Preserve: [...]. Avoid: [...].
Follow the generative-icons skill. Draw editable named parts with pivots,
and write distinct Light and Strong motion briefs before keyframing.
Make the motion an event (cause → action → response → final state), not a
sway or a nudge. Draw the subject once and transform it; never swap in a copy.
Run check and draft, then build and preview; open every sheet and fix what you see.
Report any check you could not run.
```

The installed skill carries the spec format, the family rules, what the build's visual check fails and the review checklist, so the agent needs no access to this repository.

## 4. Author, check, build and review

```sh
generative-icons create return-box --label "Return box" --intent "The box slides back to the sender" --neighbor return-package
# edit generative-icons/icons/return-box.icon.json
generative-icons check return-box     # offline, in milliseconds: structure, names, motion, lints
generative-icons draft return-box     # one engine call: a review sheet, nothing stored
generative-icons build return-box     # one engine call: export, package, verify, preview sheets, install
generative-icons preview return-box   # re-render the sheets after a change
```

- `create --from <library-id>` starts from a library icon's spec instead of the starter: many library icons are specs, and `search` marks them `"spec": true`. The icons package ships them under `specs/`, verified against its `release.json`, so this works with the installed package, the hosted library or a checkout.
- `check` validates, compiles and lints the spec offline, with a JSON Pointer and a hint for every finding: names, rest values, timing, paint past or near the canvas, parts too small at 24 px, details lost in Solid. Fix every error before drafting.
- `draft` renders every motion on the engine without exporting; `build` exports, verifies every action in the player at every size, style, weight and theme, installs only icons that pass, and writes preview sheets to `.generative-icons/previews/<id>/`.
- Look at every sheet: each style and weight, both themes, and Light against Strong at the same elapsed times. Name each flaw, fix it in the spec, then check, draft, build and preview again. Stop when a round finds nothing new or the same flaw repeats; report what remains.
- For what a spec cannot express yet (continuous loaders), `create <id> --batch` writes a JavaScript batch drawn with the authoring kit instead.

Review the silhouette, exposed surfaces, optical spacing and motion against the [drawing](design-principles.md) and [motion](motion-principles.md) principles. A passing check proves the file is valid, not that the icon is good.

## 5. Use it, or share it

A built icon is installed like a library icon and stays in your project. To share it in the [community](https://generativeicons.com/community/):

```sh
generative-icons submit return-box --license MIT --dry-run --output return-box.zip   # inspect the bundle
generative-icons submit return-box --license MIT --wait                              # publish it
```

`submit` accepts only verified builds of the current source, and needs your sign-in. The community builds the icon again from its spec, checks it the way it checks the icons it generates, and publishes it marked unreviewed until a maintainer reviews it. `--wait` prints its page when it is published, or why it was not. Submissions count toward the same daily limit as generating icons on the website. Batch icons can't be submitted, only bundled with `--dry-run`.

## Library sets

Icons for your app need none of this. Maintainers add icons to the library itself from a checkout of the library repository: each icon is built as a spec in a project like yours and added with `generative-icons library adopt`, then the set passes `library verify` and `verify-visual`, and a new set is scaffolded with `library new-set` and registered with `library register`; see [development](development.md#author-and-build-a-library-set). The family workflow for those sets is the [AI operating guide](../prototype/icons/guide/ai-operating-guide.md), which bundles the canonical rules in one file.
