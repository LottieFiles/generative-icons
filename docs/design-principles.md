# Drawing principles

## Shared construction

Use a 32 × 32 vector coordinate system. Outline weights are 1.5, 2 and 2.5 units; Regular is 2. The three weights share one geometry, and every Regular stroke also carries the `style.outline.width` token, so a runtime can render any weight from 1 to 3 without new artwork. Keep caps, joins and useful corners rounded. Sharp geometry is appropriate when required for recognition or a supplied brand shape.

The numeric source of truth is [family-contract.json](https://github.com/LottieFiles/gen-animated-icons/blob/main/motion/icon-sets/library/family-contract.json). Do not scale a whole group to simulate weight or quantize animated coordinates to the grid. Matching numbers do not guarantee equal visual weight.

## Styles

- **Outline:** use the shared stroke presets and preserve open counters at Bold.
- **Solid:** construct the intended filled silhouette. Where a stroke joins a filled surface, bury its cap inside the surface to prevent raster seams.
- **Duotone:** apply 18% tint only to a real enclosed surface. Open lines, brackets and signal arcs remain unfilled and intentionally match Outline. Declare `duotoneFill: false` when applicable.

## Geometry that survives motion

Draw the sections motion exposes: complete rims, hidden page edges, hinges and supporting rails. Parts that meet should share anchors or geometry. Background-colored patches are not occlusion. Transparent outlines cannot hide other outlines.

Preserve meaningful negative space throughout the full action, especially at Bold and in Solid. Inspect arrow junctions as painted shapes; a barb approaching a tight curve may close the opening even when centerlines look separated. Opaque painted surfaces may need slight overlap at a shared edge to prevent an antialiased seam.

## Optical review

Compare the icon with relevant neighbors at 24, 32 and 48 px. Review at 192 px for construction faults. Check actual measured CSS and backing dimensions, not just labels. Small-size readability, visual center and perceived stroke density require rendered judgment.

Keep source findings, rendered measurements and visual preferences separate. Record a reusable correction in these principles or the [polish checklist](https://github.com/LottieFiles/gen-animated-icons/blob/main/.agents/skills/animated-icon-workflow/references/polish-review.md) only after verifying it. Asset-specific dimensions and timing belong with the asset.
