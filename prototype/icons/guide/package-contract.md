# Package contract

The `@lottiefiles/generative-icons*` packages share one version and are released together. Schema version 1 is the current contract. Any schema change needs an explicit compatibility decision and a new schema version; matching filenames do not imply compatibility.

| Surface | Contract |
| --- | --- |
| Icon descriptor | Stable `id`; readable `label` and `intent`; family and category; start and end state; Light/Strong actions and optional authored reverse; supported styles and authored weights (any weight from 1 to 3 renders through the `style.outline.width` token); native asset path, bytes and SHA-256 |
| Catalog | `schemaVersion`, package `version`, categories and the complete descriptor list |
| Project (`gen-animated-icons.json`, written by the CLI) | `schemaVersion`, platform, detected framework and package manager, relative asset and descriptor folders, public URL base (web), default appearance, agent skill folders and local icon settings. `schemas/project.schema.json` describes only the original web fields; it does not yet cover the newer fields, native configs or weights between 1 and 3. |
| Lock (`.gen-animated-icons/lock.json`) | Selected IDs and versions, installed paths and hashes of managed files; local edits block replacement |
| Release (`release.json`) | Source commit and dirty status, catalog/asset/docs hashes, runtime, CLI and player versions, and the `exporter` note (export is needed for authoring only and is not bundled) |

## How packages are built

`gen-animated-icons library packages` builds the catalog and assets from registered source sets. It rejects duplicate IDs, checks source package hashes and confirms the required animation IDs inside each native bundle. Each icon gets its own descriptor module. A Strong bundle contains both Light and Strong actions, plus reverse actions where applicable; the runtime selects the requested action. Native bundles are copied unchanged.

The runtime package ships the gallery's controller as its canonical implementation; the build copies that exact file and parity tests guard it. Apps should use the runtime package rather than copying the controller.

The website consumes the published packages and validates the release hashes when importing the catalog, assets and documentation. No engine code is included in the packages or the CLI: native export runs on LottieFiles' hosted engine.

Projects set up by the earlier `lottie-icons` command (`lottie-icons.json` and `.lottie-icons/lock.json`) keep working with `gen-animated-icons add`, `info`, `diff` and `remove`, which write the same bytes there; `gen-animated-icons init --migrate` moves them to the current files.
