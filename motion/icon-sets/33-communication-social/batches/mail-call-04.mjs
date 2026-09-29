// motion/icon-sets/31-communication-social/batch-a.mjs  (revision 03: address-book + dial-pad only)
import {make, F, P, rigid, posed, tube, fill, stroke, framed, rect, circle, holes} from './helpers.mjs';

// ---------- ADDRESS BOOK  (gesture; duotoneFill true, the cover is a real enclosed surface)
// Fix: head raised 0.6 (12.3 -> 11.7) in Outline and Solid. Cover, rings, shoulder path, travel and opacity unchanged.
// Bold head paint bottom 11.7+1.8+1.25 = 14.75 vs Bold shoulder tube top 17.2-1.25 = 15.95 -> gap 1.2 (was 0.6),
//   constant through the shared rigid move. Solid head (r 2.1) bottom 13.8 vs solid tube top 15.9 -> gap 2.1.
// Strong prep peak dy -1.5: Solid head top 9.6-1.5 = 8.1 vs Solid window top 6.5 -> 1.6;
//   Outline Bold head top 8.65-1.5 = 7.15 vs cover inner Bold edge 5.25 -> 1.9.
// Strong entry from +2.4: Bold shoulder bottom 22.25+2.4 = 24.65 vs window bottom 26 -> 1.35 (Solid 24.7 -> 1.3).
// Light range -1.0..+1.6 stays inside those bounds. Recycle jumps only at opacity 0 (Light .20-.28, Strong .24-.32).
const browseY = F(
  [[0, 0], [.2, -1, 'in'], [.22, 1.6, 'linear'], [.28, 1.6], [.52, 0, 'smooth']],
  [[0, 0], [.08, .6], [.24, -1.5, 'launch'], [.26, 2.4, 'hold'], [.32, 2.4, 'hold'], [.5, -.4], [.62, 0]]);
const browseV = P(
  [[0, 1], [.2, 0, 'out'], [.28, 0], [.52, 1, 'smooth']],
  [[0, 1], [.1, 1], [.24, 0, 'out'], [.32, 0], [.46, 1, 'out']]);

const addressBook = make('address-book', 'Address book',
  'The contact card slides up and fades inside the book window while the next contact rises in from below; Strong dips first, flicks the card away faster and lands the next contact with a small overshoot, all inside the window.',
  [
    ...framed('cover', rect(7, 4, 19, 24, 2), holes(rect(5.7, 2.7, 21.6, 26.6, 3.3), rect(9, 6.5, 16, 19.5, 2.2))),
    ...tube('ring-top', 'M4.2 9 L8.5 9'),
    ...tube('ring-bottom', 'M4.2 23 L8.5 23'),
    ...posed([
      stroke('head', circle(17, 11.7, 1.8)),
      fill('head-solid', circle(17, 11.7, 2.1), 'solid'),
      ...tube('shoulders', 'M12.5 21 C12.5 18.2 14.5 17.2 17 17.2 C19.5 17.2 21.5 18.2 21.5 21'),
    ].map(p => ({...p, visibility: browseV})), rigid((t, s) => ({dy: browseY(t, s)}))),
  ], .54, .64);
addressBook.duotoneFill = true;

// ---------- DIAL PAD  (gesture; duotoneFill false; weight-dependent filled dots)
// Outline dot r = 0.7*w -> 1.05 / 1.4 / 1.75 (Regular keeps 1.4). Solid dot r = 2.0 (diameter 4.0 vs 2.1/2.8/3.5).
// Rows y 5/12/19/26, columns x 9/16/23: spacing 7 both ways. At 6 spacing a width-2.6 Solid ring cannot clear a r 2.0 neighbour by 1.
// Halo centreline r = rd + 0.2 + b*p (rd = that style's dot radius; b = 0.6 Outline, 0.35 Solid). Paint overlaps the pressed dot at start,
//   so no seam. Outer paint max vs neighbour edge (7 - rd): Light 2.85 vs 5.95 -> 3.10; Regular 3.2 vs 5.6 -> 2.40;
//   Bold 3.8 vs 5.25 -> 1.45; Solid 3.85 vs 5.0 -> 1.15. Bottom-row halo (centre y 26): Bold 29.8, Solid 29.85.
//   Pressed bottom Solid dot 27+2 = 29.0; top Solid dot edge 3.0. Everything inside [2,30].
// Light presses only key (16,12) by 0.5. Strong presses rows 12/19/26 by 1.0 with a 'hit'; halo recycles 0 -> 7 -> 14 -> 0 only inside opacity-0 spans.
const ROWS = [5, 12, 19, 26], COLS = [9, 16, 23], SOLID_R = 2, outR = w => .7 * (w || 2);
const keyPress = row => F(
  row === 1 ? [[0, 0], [.12, .5, 'smooth'], [.36, 0, 'smooth']] : [[0, 0]],
  [[0, 0], [(row - 1) * .22, 0, 'hold'], [(row - 1) * .22 + .06, 1, 'hit'], [(row - 1) * .22 + .2, 0]]);
const rippleR = P(
  [[0, 0], [.08, 0], [.4, 1, 'out']],
  [[0, 0], [.03, 0], [.19, 1, 'out'], [.2, 1], [.21, 0], [.25, 0], [.41, 1, 'out'], [.42, 1], [.43, 0], [.47, 0], [.63, 1, 'out']]);
const rippleV = P(
  [[0, 0], [.06, 0], [.1, 1, 'out'], [.4, 0, 'out']],
  [[0, 0], [.03, 0], [.06, 1, 'out'], [.19, 0, 'out'], [.25, 0], [.28, 1, 'out'], [.41, 0, 'out'], [.47, 0], [.5, 1, 'out'], [.63, 0, 'out']]);
const rippleY = P([[0, 0]], [[0, 0], [.2, 0], [.22, 7], [.42, 7], [.44, 14], [.65, 14], [.68, 0]]);  // jumps only inside opacity-0 spans
const halo = (rd, b, t, s) => circle(16, 12, rd + .2 + b * rippleR(t, s));

const keyParts = COLS.flatMap((x, c) => ROWS.flatMap((y, r) => {
  const parts = [
    {name: `key-${r}-${c}`, d: (t, s, rev, w) => circle(x, y, outR(w)), style: 'outline', type: 'fill'},
    {name: `key-${r}-${c}-solid`, d: circle(x, y, SOLID_R), style: 'solid', type: 'fill'},
  ];
  return c === 1 && r > 0 ? posed(parts, rigid((t, s) => ({dy: keyPress(r)(t, s)}))) : parts;
}));

const dialPad = make('dial-pad', 'Dial pad',
  'One key presses in and a halo spreads from it; Strong presses harder and dials three keys down the middle column in sequence.',
  [
    ...keyParts,
    ...posed([
      {name: 'ripple', d: (t, s, rev, w) => halo(outR(w), .6, t, s), style: 'outline', type: 'stroke', width: 2, visibility: rippleV},
      {name: 'ripple-solid', d: (t, s) => halo(SOLID_R, .35, t, s), style: 'solid', type: 'stroke', width: 2.6, visibility: rippleV},
    ], rigid((t, s) => ({dy: rippleY(t, s)}))),
  ], .44, .7);
dialPad.duotoneFill = false;

export const icons = [addressBook, dialPad];
