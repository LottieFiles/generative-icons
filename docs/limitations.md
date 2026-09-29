# Known limitations

## Native export needs the Engine exporter

Rebuilding native Lottie files requires the Engine exporter, which is not publicly distributed. Existing icons, the packages and the local gallery work without it, and you can author and validate source without it. Maintainers run the native export for pull requests; see [development](development.md#native-export).

## Legacy set coverage

Heart/Bell (set 01) and Search (set 02) use legacy inline builders and are not covered by the separate-definition preflight. Their set-specific checks apply instead. Some legacy review scripts refer to historical comparison evidence; missing evidence is not a pass.

## Verification scope

CI covers controller behavior, the generation harness, catalog and package integrity, shared paint rules and generated documentation. It does not cover every native frame or replace rendered review; see [verification](verification.md).

## Formats

Packages ship native dotLottie files and per-icon descriptors. Animated SVG, SMIL, PNG and other export formats are not yet available.
