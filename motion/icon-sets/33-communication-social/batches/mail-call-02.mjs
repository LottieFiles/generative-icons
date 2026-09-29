// motion/icon-sets/31-communication-social/batch-a.mjs  (revision 02 draft)
import {make, F, P, rigid, posed, tube, fill, stroke, bothFill, framed, rect, circle, polygon, holes, geometricRearm} from './helpers.mjs';

// ---------- 1. INBOX  (state/output; preview reset via geometricRearm; duotoneFill true)
// Geometry unchanged. The card lands ON the shelf, above the front lip; it is never occluded.
// Landing gaps: Solid card bottom 19.7 vs shelf paint top 20.7 = 1.0; Outline Bold 19.65 vs 20.75 = 1.1.
// Strong 'hit' key is the extremum (in-tangent contributes only negative Hermite weight), so peak dy = 6.6.
const cardDrop = F(
  [[0, 0], [.08, 0], [.6, 6.6, 'smooth']],                        // Light: one quiet 6.6 glide, 0.08 to 0.60 s
  [[0, 0], [.14, -.8], [.4, 6.6, 'hit'], [.52, 5.9], [.66, 6.6]]); // Strong: 0.8 lift, fast drop 0.14 to 0.40 s, one rebound

const inbox = geometricRearm(make('inbox', 'Inbox',
  'An incoming card lifts slightly, drops into the tray and lands on the shelf above the front lip; Strong prepares, falls faster and settles with one bounded rebound. The card stays received in the tray.',
  [
    ...posed([
      ...framed('card', rect(9.5, 4.2, 13, 7.6, 1.4), rect(8.2, 2.9, 15.6, 10.2, 2.7)),
      stroke('card-flap', 'M9.5 4.2 L16 8.2 L22.5 4.2'),
    ], rigid((t, s) => ({dy: cardDrop(t, s)}))),
    fill('front-interior', polygon([[5, 22], [10.5, 22], [12.2, 23.7], [19.8, 23.7], [21.5, 22], [27, 22], [27, 27.5], [5, 27.5]], [0, 0, .6, .6, 0, 0, 1.5, 1.5]), 'tint'),
    ...tube('walls', 'M5 15.5 L5 26 C5 26.8 5.7 27.5 6.5 27.5 L25.5 27.5 C26.3 27.5 27 26.8 27 26 L27 15.5'),
    ...tube('shelf', 'M5 22 L10.5 22 L12.2 23.7 L19.8 23.7 L21.5 22 L27 22'),
    fill('front-solid', polygon([[3.7, 20.7], [10, 20.7], [11.7, 22.4], [20.3, 22.4], [22, 20.7], [28.3, 20.7], [28.3, 28.8], [3.7, 28.8]], [0, 0, .6, .6, 0, 0, 2.8, 2.8]), 'solid'),
  ], .62, .7));
inbox.duotoneFill = true;

// ---------- 2. ADDRESS BOOK  (gesture; duotoneFill true, the cover is a real enclosed surface)
// Channel: Solid window y 6.5..26, x 9..25. Outline cover inner Bold edge y 5.25..26.75.
// Compact contact painted extent at rest: 9.25 (Bold head top) .. 22.3 (shoulder tube bottom).
// Visible travel: Strong -1.5 .. +2.4  -> painted 7.75 .. 24.7, gap >= 1.25 to the window.
//                 Light  -1.0 .. +1.6  -> painted 8.25 .. 23.9. Recycle jumps happen only at opacity 0.
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
      stroke('head', circle(17, 12.3, 1.8)),
      fill('head-solid', circle(17, 12.3, 2.1), 'solid'),
      ...tube('shoulders', 'M12.5 21 C12.5 18.2 14.5 17.2 17 17.2 C19.5 17.2 21.5 18.2 21.5 21'),
    ].map(p => ({...p, visibility: browseV})), rigid((t, s) => ({dy: browseY(t, s)}))),
  ], .54, .64);
addressBook.duotoneFill = true;

// ---------- 3. VOICEMAIL  (accepted; unchanged geometry, profiles and 360 cycle declaration)
const spin = F(
  [[0, 0], [.9, 360, 'smooth']],
  [[0, 0], [.14, -18], [.62, 360, 'launch']]);
const reel = (n, cx) => posed([...tube(n + '-rim', circle(cx, 16, 4.5)), ...tube(n + '-mark', `M${cx} 15 L${cx} 11.5`)],
  rigid((t, s) => ({angle: spin(t, s), pivot: [cx, 16]})));
const voicemail = {...make('voicemail', 'Voicemail',
  'Both tape reels turn one full playback revolution with an index mark showing the motion; Strong recoils and spins decisively faster.',
  [...tube('tape', 'M9 20.5 L23 20.5'), ...reel('reel-left', 9), ...reel('reel-right', 23)], .9, .65),
  cycleDegrees: 360, duotoneFill: false};

// ---------- 4. TRANSFER CALL  (gesture; duotoneFill false)
// Route length = 15 straight + quarter arc r3 (4.712) + 10.5 riser = 30.212, ending at the head tip (24, 11.5).
// Head splay widened to 3.5 so the handset mouth stays >= 1.2 clear of the left barb at the -6 degree tilt.
const ROUTE = 30.212;
const routeAt = u => {
  const d = u * ROUTE;
  if (d <= 15) return [6 + d, 25];
  if (d <= 19.712) { const a = (d - 15) / 4.712 * Math.PI / 2; return [21 + 3 * Math.sin(a), 22 + 3 * Math.cos(a)]; }
  return [24, 22 - (d - 19.712)];
};
const callU = F(
  [[0, 0], [.06, 0], [.66, 1, 'smooth']],            // Light: arrives at 0.66 s
  [[0, 0], [.14, 0, 'hold'], [.44, 1]]);             // Strong: launched by the tilt, arrives at 0.44 s
const callV = P(
  [[0, 0], [.06, 0], [.14, 1, 'out'], [.72, 1], [.84, 0, 'out']],  // visible 0.06 s past arrival, then retires
  [[0, 0], [.12, 0], [.18, 1, 'out'], [.52, 1], [.62, 0, 'out']]);  // visible 0.08 s past arrival, then retires
const tilt = F([[0, 0]], [[0, 0], [.14, -6], [.3, 2, 'hit'], [.46, 0]]);

const transferCall = make('transfer-call', 'Transfer call',
  'The call leaves the handset as a dot, follows the route, turns up the bend and lands on the recipient arrow tip before retiring; Strong tips the handset first, then sends the call in one fast decisive run.',
  [
    ...posed([...tube('bowl', 'M5 6.5 L5 9.5 C5 13.6 8.4 17 12.5 17 L15.5 17'), ...tube('ear', 'M3.5 6.5 L6.5 6.5'), ...tube('mouth', 'M15.5 15 L15.5 18.5')],
      rigid((t, s) => ({angle: tilt(t, s), pivot: [5, 6.5]}))),
    ...tube('route', 'M6 25 L21 25 C22.7 25 24 23.7 24 22 L24 11.5'),
    ...tube('head', 'M20.5 15 L24 11.5 L27.5 15'),
    ...posed(bothFill('call', circle(6, 25, 1.6)).map(p => ({...p, visibility: callV})),
      rigid((t, s) => { const [x, y] = routeAt(callU(t, s)); return {dx: x - 6, dy: y - 25}; })),
  ], .84, .62);
transferCall.duotoneFill = false;

// ---------- 5. DIAL PAD  (gesture; duotoneFill false; filled dots, constant frame weights)
// Rows y 6/12/18/24, columns x 9/16/23, dot r 1.4. Ripple ring r 1.9..2.2 around the pressed key's rest centre.
// Solid ripple outer paint 3.5 vs vertical neighbour edge 4.6 -> gap 1.1 (Bold 1.15). Bottom ripple paint 27.5.
// Light presses ONE key (16, 12) by 0.5. Strong dials rows 12, 18, 24 in sequence, each press 1.0 with a 'hit'.
const keyPress = row => F(
  row === 1 ? [[0, 0], [.12, .5, 'smooth'], [.36, 0, 'smooth']] : [[0, 0]],
  [[0, 0], [(row - 1) * .22, 0, 'hold'], [(row - 1) * .22 + .06, 1, 'hit'], [(row - 1) * .22 + .2, 0]]);
const rippleR = P(
  [[0, 0], [.08, 0], [.4, 1, 'out']],
  [[0, 0], [.03, 0], [.19, 1, 'out'], [.2, 1], [.21, 0], [.25, 0], [.41, 1, 'out'], [.42, 1], [.43, 0], [.47, 0], [.63, 1, 'out']]);
const rippleV = P(
  [[0, 0], [.06, 0], [.1, 1, 'out'], [.4, 0, 'out']],
  [[0, 0], [.03, 0], [.06, 1, 'out'], [.19, 0, 'out'], [.25, 0], [.28, 1, 'out'], [.41, 0, 'out'], [.47, 0], [.5, 1, 'out'], [.63, 0, 'out']]);
const rippleY = P([[0, 0]], [[0, 0], [.2, 0], [.22, 6], [.42, 6], [.44, 12], [.65, 12], [.68, 0]]);  // jumps only inside opacity-0 spans
const rippleD = (t, s) => circle(16, 12, 1.9 + .3 * rippleR(t, s));

const keyParts = [9, 16, 23].flatMap((x, c) => [6, 12, 18, 24].flatMap((y, r) => {
  const parts = bothFill(`key-${r}-${c}`, circle(x, y, 1.4));
  return c === 1 && r > 0 ? posed(parts, rigid((t, s) => ({dy: keyPress(r)(t, s)}))) : parts;
}));

const dialPad = make('dial-pad', 'Dial pad',
  'One key presses in and a ripple spreads from it; Strong presses harder and dials three keys down the middle column in sequence.',
  [
    ...keyParts,
    ...posed([{name: 'ripple', d: rippleD, style: 'outline', type: 'stroke', width: 2, visibility: rippleV},
              {name: 'ripple-solid', d: rippleD, style: 'solid', type: 'stroke', width: 2.6, visibility: rippleV}],
      rigid((t, s) => ({dy: rippleY(t, s)}))),
  ], .44, .7);
dialPad.duotoneFill = false;

export const icons = [inbox, addressBook, voicemail, transferCall, dialPad];
