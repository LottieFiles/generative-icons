# npm distribution

- `gen-animated-icons/`: the package people run with `npx gen-animated-icons`. It holds a
  small Node shim and lists one optional dependency per platform.
- `platforms/<target>/`: one package per release target (macOS and Linux for now; Windows
  follows once its build passes the parity tests). The Release workflow
  (`.github/workflows/release.yml`) copies that target's binary into `bin/`, adds the
  release tarball's `LICENSE`, `THIRD_PARTY_NOTICES.md` and `THIRD_PARTY_LICENSES.txt`
  (the license texts of everything the binary links, generated per target), and packs
  every package; publishing stays a manual step (see `docs/maintainers/releasing.md`).

npm installs only the platform package that matches the machine (`os`, `cpu`, `libc`), and
nothing runs at install time. Every package in a release carries the same version as the
Cargo workspace; the shim's `optionalDependencies` pin it exactly, and the Release workflow
refuses to pack when they disagree.

Set `GEN_ANIMATED_ICONS_BINARY` to run a local build through the shim. Binaries copied into
`platforms/*/bin/` are ignored by Git.

## Addresses before launch

The manifests, the package README and the shim use the intended launch addresses. None of
them works yet:

- the repository `github.com/LottieFiles/gen-animated-icons` is private;
- the site `gen-animated-icons.lottiefiles.com` is not live, including the `/install` page
  the shim names when it cannot run the binary;
- none of these packages is on npm.

Before the first publish, make the repository public, put the site and its `/install` page
live, and check every address in this folder.
