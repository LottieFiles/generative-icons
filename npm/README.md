# npm distribution

- `generative-icons/`: the package people run with `npx generative-icons`. It holds a
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

## Addresses

The manifests, the package README and the shim name the site, `https://generativeicons.com`
(the shim prints its `/install` page when it cannot run the binary), and the repository,
`github.com/LottieFiles/generative-icons`. A published version keeps them for good, so check
them before each release when either has moved.
