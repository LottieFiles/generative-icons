# Development

## Requirements

- Node 22+ and Python 3
- `unzip`, for verified context packets
- The Engine exporter, only for native builds (see below)

```sh
npm ci           # workspace packages
npm run setup    # authoring dependencies under motion/
npm run dev      # local gallery on http://127.0.0.1:4189/icons/01/
```

The gallery vendors its player and fonts and needs no build step. `npm run dev` serves only the `prototype/` directory on localhost; pass `-- --port 4190` to change the port.

## Native export

Building native Lottie files needs the **Engine exporter**, an external executable that is not publicly distributed or included in npm dependencies. Maintainers have it. Everyone else can author and validate source without it: open a pull request and a maintainer runs the native export.

To configure it, point `CREATOR_MCP_BIN` at the executable and run the doctor:

```sh
export CREATOR_MCP_BIN=/path/to/creator-mcp
npm run doctor
```

The doctor checks Node, Python, `unzip` and authoring dependencies, then exports a small temporary scene over stdio MCP and cleans up. The pipeline was developed against Engine commit `27a04e8598719d5357af502007ffb66b5d94edbd`; verify compatibility before using another build.

`ENGINE_MCP_ROOT` is a compatibility alias for a checkout containing `target/debug/creator-mcp`. `CREATOR_MCP_BIN` wins if both are set. `.env.example` documents the values but is not loaded automatically.

## Author and build a set

The step-by-step workflow, including every command, is in [creating icons](creating-icons.md). A few builder notes:

- Registration requires a built, nonempty manifest, passing source screening and passing native verification. Registering a missing manifest would stop catalog checks from running.
- Legacy sets 01 (Heart/Bell) and 02 (Search) use inline builders with their own checks.
- Builders replace set outputs. Build complete sets and confirm unrelated asset hashes are unchanged. Do not rebuild just to preview existing files.
- Intermediate `scenes/` and `native/` directories are rebuildable and ignored by Git; final assets under `prototype/icons/NN/assets/` are tracked.

## Packages

```sh
npm run build:packages   # build packages/*/dist from registered sets
npm run test:packages    # package unit tests
npm run pack:release     # pack tarballs
npm run test:consumers   # install tarballs into vanilla, React and Next.js fixtures
```

See the [package contract](package-contract.md) and the [release runbook](maintainers/releasing.md).

## AI guide

`npm run docs:build` regenerates `prototype/icons/guide/` (the downloadable AI operating guide and its source hashes) from the canonical documents. Edit the canonical source, never the generated copy, then run `npm run check:docs`.
