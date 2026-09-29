# Contributing

Thanks for helping grow Generative Animated Icons. Every contribution arrives as a GitHub pull request.

## Before you start

- **Missing icon?** Search the [gallery](README.md#preview-and-download-locally) or run `node motion/icon-sets/library/generation-context.mjs --query "your meaning" --search`. If nothing fits, open an [icon request](https://github.com/LottieFiles/lottie-generative-icons/issues/new?template=icon_request.yml) or build it yourself.
- **Something broken?** Open a [bug report](https://github.com/LottieFiles/lottie-generative-icons/issues/new?template=bug_report.yml) with the icon ID, motion profile, style and steps to reproduce.
- Keep unrelated changes in separate pull requests.

## Add or revise an icon

1. **Fork and install.** Node 22+ and Python 3.

   ```sh
   git clone https://github.com/<you>/lottie-generative-icons.git
   cd lottie-generative-icons
   npm ci && npm run setup
   ```

2. **Author with your AI.** Follow [creating icons](docs/creating-icons.md). Your coding agent reads [AGENTS.md](AGENTS.md) and the family rules, then writes named geometry and Light/Strong motion in the set's `definitions.mjs`. Preserve existing icon IDs.

3. **Run the source checks.** These work without the native exporter:

   ```sh
   node motion/icon-sets/library/preflight.mjs motion/icon-sets/SET-ID
   npm test
   npm run check:source
   ```

   If you have the Engine exporter, also build and verify the set as described in [creating icons](docs/creating-icons.md#5-build-and-verify).

4. **Open a pull request** using the template. Describe the meaning, product context and states. Attach screenshots or recordings if you have them, and list any checks you could not run.

**Maintainers handle native export.** A maintainer builds the native assets, reviews the rendered output (Light and Strong, every style, reduced motion) and merges. See [governance](docs/maintainers/governance.md).

## Docs and code changes

Update documentation when behavior changes. After editing canonical guides, run `npm run docs:build` and `npm run check:docs`.

## Conduct and security

Follow the [Code of Conduct](CODE_OF_CONDUCT.md). Report vulnerabilities privately as described in [SECURITY.md](SECURITY.md).

## License

Contributions are released under the [MIT License](LICENSE). Identify any new dependency or borrowed material in your pull request.
