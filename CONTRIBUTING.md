# Contributing

Thanks for helping grow Generative Icons. Code and library changes arrive as GitHub pull requests; icons made in an app can also arrive through `generative-icons submit`.

## Before you start

- **Missing icon?** Search with `npx generative-icons search "your meaning"`. If nothing fits, create it in your own project (see [creating icons](docs/creating-icons.md)) or open an [icon request](https://github.com/LottieFiles/gen-animated-icons/issues/new?template=icon_request.yml).
- **Something broken?** Open a [bug report](https://github.com/LottieFiles/gen-animated-icons/issues/new?template=bug_report.yml) with the icon ID, motion profile, style and steps to reproduce, or the CLI command, its output and `generative-icons --version`.
- Keep unrelated changes in separate pull requests.

**Building icons needs a LottieFiles sign-in.** Native export runs on LottieFiles' hosted engine (`generative-icons login`). The hosted engine is internal until launch, so building may not work for your account yet. Authoring, the source checks and the test suites work without it.

## Offer an icon from your app

Create and build the icon in your project, then:

```sh
generative-icons submit <id> --license MIT --dry-run --output bundle.zip   # inspect what would be sent
generative-icons submit <id> --license MIT
```

Only verified builds of the current source are accepted, and every submission is reviewed before it joins the library. The submission service opens at launch.

## Add or revise a library set

1. **Fork and build the CLI.** You need Rust 1.89+, clang, Node 22+ and Python 3; see [development](docs/development.md).

   ```sh
   git clone https://github.com/<you>/generative-icons.git
   cd generative-icons
   cargo build --release -p generative-icons
   ```

2. **Author with your AI.** Your coding agent reads [AGENTS.md](AGENTS.md) and the family rules, then writes named geometry and Light/Strong motion in the set's `definitions.mjs` and `batches/`. Start a new set with `generative-icons library new-set <slug>`. Preserve existing icon IDs.

3. **Run the checks.**

   ```sh
   generative-icons library preflight SET-ID
   generative-icons library build SET-ID          # needs a sign-in
   generative-icons library verify SET-ID
   generative-icons library verify-visual SET-ID
   ```

   Report any check you could not run.

4. **Open a pull request** using the template. Describe the meaning, product context and states, attach the preview sheets or recordings, and list the checks you ran.

A maintainer reviews the rendered output (Light and Strong, every style, reduced motion) and merges. See [governance](docs/maintainers/governance.md).

## Code and docs changes

The CLI lives in `crates/`; read [crates/README.md](crates/README.md) first. Every change must keep the catalog rebuilding byte for byte: run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`. Tests must never use a real sign-in; see [development](docs/development.md#test).

Update documentation when behavior changes. After editing canonical guides, run `npm run docs:build` and `npm run check:docs`.

## Conduct and security

Follow the [Code of Conduct](CODE_OF_CONDUCT.md). Report vulnerabilities privately as described in [SECURITY.md](SECURITY.md).

## License

Contributions are released under the [MIT License](LICENSE). Identify any new dependency or borrowed material in your pull request.
