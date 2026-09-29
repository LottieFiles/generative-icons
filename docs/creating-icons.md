# Create an icon

One coding AI can run the whole workflow, or you can author the source yourself. The rules and checks are the same either way.

You can author and validate source without the exporter. Open a pull request and a maintainer runs the native export (see [contributing](https://github.com/LottieFiles/lottie-generative-icons/blob/main/CONTRIBUTING.md)). The native build and registration steps need the Engine exporter described in [development](development.md#native-export).

## 1. Search and choose examples

```sh
node motion/icon-sets/library/generation-context.mjs --query "package return" --search
node motion/icon-sets/library/generation-context.mjs --query "package return" --neighbor return-package --neighbor delivery-truck
```

Inspect the meaning and geometry of each neighbor. Search is lexical: similar names and opposite directions can mean different things. Preserve existing public IDs and avoid duplicate concepts.

## 2. Scaffold a set

For a new set:

```sh
npm run icons:new -- landscaping --name "Landscaping"
```

This picks the next unused two-digit set number and creates editable source, build and verification scripts, packaging, a download README, a review record and a native renderer test page. It creates no artwork and does not register the set. An empty set intentionally fails source screening until you add icons to `definitions.mjs`.

Skip this step when revising an existing set. Read that set's README first.

## 3. Give your AI a complete brief

```text
Create an icon for [meaning] used in [product context].
Initial state: [...]. Completed state: [...]. Reverse/reset: [...].
Use these existing neighbors: [...].
Preserve: [...]. Avoid: [...].
Read AGENTS.md and the animated-icon-workflow skill.
Create editable named vector geometry, parts/pivots, and distinct
Light/Strong timing plans. Identify construction and motion risks.
Run the source checks, then export and inspect native output if the
exporter is available. Report any check you could not run.
```

Give the agent the actual reference geometry and rules, not only paths it cannot open. The [AI operating guide](https://github.com/LottieFiles/lottie-generative-icons/blob/main/prototype/icons/guide/ai-operating-guide.md) bundles the canonical rules in one file.

## 4. Review the plan

Before integrating, check the complete silhouette, exposed surfaces, optical spacing and motion plan against the [drawing](design-principles.md) and [motion](motion-principles.md) principles. Record what meets the brief, what must change and which parts and times are affected. Then integrate the source into the set's `definitions.mjs`.

## 5. Build and verify

Replace `SET-ID` with the set directory, for example `39-food-drink`.

```sh
node motion/icon-sets/library/preflight.mjs motion/icon-sets/SET-ID    # source screening, no exporter
node motion/icon-sets/library/build-checked.mjs motion/icon-sets/SET-ID # exporter
node motion/icon-sets/SET-ID/verify.mjs                                 # checks the native build
```

Always build the complete set; partial builds can truncate manifests. Then follow [verification](verification.md) for rendered review. Feed failing diagnostics and rendered frames into the next correction. If the same failure repeats without new evidence, change approach or report the blocker.

## 6. Register

After a successful complete native build and verification (exporter required):

```sh
npm run icons:register -- SET-ID
```

Registration checks the set, then adds it to `prototype/icons/catalog.json` with a named category. It is safe to repeat and leaves the catalog unchanged on failure. After the final rendered review, run the set's `package.py` again so the download includes the current catalog and evidence.

## Optional: Claude Code adapter

If you prefer to generate a text proposal through the Claude Code CLI, see the [Claude design adapter](https://github.com/LottieFiles/lottie-generative-icons/blob/main/.agents/skills/animated-icon-workflow/references/claude-design.md). It is not required.
