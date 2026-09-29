// motion/icon-sets/31-communication-social/transfer-call-03.mjs  (revision 03: dot radius only)
import {make, F, P, rigid, posed, tube, bothFill, circle} from './helpers.mjs';

// Route length = 15 straight + quarter arc r3 (4.712) + 10.5 riser = 30.212, ending at the head tip (24, 11.5).
// Head splay 3.5 keeps the handset mouth >= 1.2 clear of the left barb at the -6 degree tilt (unchanged).
// Rev 03: call disk r 1.6 -> 2.15 (dia 4.3 vs Bold 2.5 / Solid 2.6). Same centre, route, timing, visibility.
// Bounds r 2.15: x 3.85..26.15, y 9.35..27.15. Mouth paint bottom <= 20.2 through the tilt; dot top on the
// straight 22.85 -> min gap 2.65. Dot overlaps the barbs only on the designed landing at the tip.
const ROUTE = 30.212;
const routeAt = u => {
  const d = u * ROUTE;
  if (d <= 15) return [6 + d, 25];
  if (d <= 19.712) { const a = (d - 15) / 4.712 * Math.PI / 2; return [21 + 3 * Math.sin(a), 22 + 3 * Math.cos(a)]; }
  return [24, 22 - (d - 19.712)];
};
const callU = F(
  [[0, 0], [.06, 0], [.66, 1, 'smooth']],            // Light: one smooth transfer, arrives at 0.66 s
  [[0, 0], [.14, 0, 'hold'], [.44, 1]]);             // Strong: launched by the tilt, arrives at 0.44 s
const callV = P(
  [[0, 0], [.06, 0], [.14, 1, 'out'], [.72, 1], [.84, 0, 'out']],  // holds 0.06 s at the tip, then retires
  [[0, 0], [.12, 0], [.18, 1, 'out'], [.52, 1], [.62, 0, 'out']]);  // holds 0.08 s at the tip, then retires
const tilt = F([[0, 0]], [[0, 0], [.14, -6], [.3, 2, 'hit'], [.46, 0]]);
const CALL_R = 2.15;  // one static disk for both profiles and all weights

const transferCall = make('transfer-call', 'Transfer call',
  'The call leaves the handset as a dot, follows the route, turns up the bend and lands on the recipient arrow tip before retiring; Strong tips the handset first, then sends the call in one fast decisive run.',
  [
    ...posed([...tube('bowl', 'M5 6.5 L5 9.5 C5 13.6 8.4 17 12.5 17 L15.5 17'), ...tube('ear', 'M3.5 6.5 L6.5 6.5'), ...tube('mouth', 'M15.5 15 L15.5 18.5')],
      rigid((t, s) => ({angle: tilt(t, s), pivot: [5, 6.5]}))),
    ...tube('route', 'M6 25 L21 25 C22.7 25 24 23.7 24 22 L24 11.5'),
    ...tube('head', 'M20.5 15 L24 11.5 L27.5 15'),
    ...posed(bothFill('call', circle(6, 25, CALL_R)).map(p => ({...p, visibility: callV})),
      rigid((t, s) => { const [x, y] = routeAt(callU(t, s)); return {dx: x - 6, dy: y - 25}; })),
  ], .84, .62);
transferCall.duotoneFill = false;

export const icons = [transferCall];
