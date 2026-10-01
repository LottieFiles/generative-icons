# Review previews

Look at the PNGs `preview` writes: one sheet per motion, rows for the looks (Outline at 1.5, 2 and 2.5, Solid, Duotone on dark, then Regular on light), columns for the same elapsed times in both sheets, ending on the rest pose. Use `--times` to put columns on the preparation, peak, contact, settle, hold and reset moments. A passing `check` or `build` proves the file is valid, not that the icon is good.

## Reject and fix

| What you see | Fix |
| --- | --- |
| Sharp or hooked contours despite round caps | Author the curve in the path; check inner and outer turns in Solid and at 2.5 weight |
| Equal stroke numbers but uneven visible weight | Use a single centerline for simple bars, or respace; never scale a stroked part |
| A lid, handle or flap shows loose ends when it moves | Draw the rim, hinge and contact surfaces; attach endpoints to the moving boundary |
| A counter or gap fills in at 24 px | Measure the swept paint at the widest weight; widen the opening or move the pivot |
| A Solid detail vanishes or a joined fill shows a seam | Give Solid its own contour; overlap paints that form one surface |
| Duotone tints an open tray, doorway or gap | Tint only enclosed surfaces; with none, set `duotoneFill = false` |
| An arrow barb merges into its curve or leaves a dot | Give the head a clear straight approach; hide zero-length round-cap paths |
| Parts drift apart, rigid hardware bends, or the path jumps | Share one driver; move rigid parts with `rigid`; keep path commands identical over time |

## Strong against Light

- Compare both at the same elapsed times. If you cannot tell them apart without their labels, revise Strong.
- Strong needs a stronger primary action: more travel, a wider turn or a more decisive stroke. Reject a difference that is only a follower, a pause, extra keys or a longer file.
- Check the first visible change. Do not spend the opening on a part that is already complete.
- Keep progress monotone, keep rigid parts rigid, and keep holds where the meaning needs them.

## Report

For each icon, state: its meaning and state policy, what differs between Light and Strong, which previews you opened, the findings you fixed, and anything left unresolved or unchecked.
