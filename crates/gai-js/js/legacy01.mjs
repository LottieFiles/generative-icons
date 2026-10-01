// gai:legacy/01: the authoring half of motion/icon-sets/01-heart-bell/build.mjs, which has no definitions.mjs.
// Lines 23-119 follow verbatim (a drift test compares them); strongOverrides and pose restate lines 175-184 and 194-198.
import svgpath from 'svgpath';
// Each key is an authored extremum. Cosine spacing gives zero speed at turns
// and peak speed between them. The sampled path keys are then interpolated once.
function phrase(keys, time) {
  if (time <= keys[0][0]) return keys[0][1];
  if (time >= keys.at(-1)[0]) return keys.at(-1)[1];
  const index = keys.findIndex((key) => key[0] > time);
  const a = keys[index - 1],
    b = keys[index];
  const u = (1 - Math.cos((Math.PI * (time - a[0])) / (b[0] - a[0]))) / 2;
  return a[1] + (b[1] - a[1]) * u;
}
const heartKeys = [
  [0, 0],
  [0.045, -0.2],
  [0.13, 1],
  [0.30, -0.06],
  [0.40, 0],
];
const shellKeys = [[0,0],[.06,-2],[.17,9],[.32,-3.8],[.46,1.1],[.58,0]];
// Strong is extrema-only: squeeze, decisive open, two shrinking corrections, settle. No holds.
const heartStrongKeys = [[0,0],[0.05,-0.30],[0.15,1],[0.27,-0.16],[0.37,0.05],[0.46,0]];
const shellStrongKeys = [[0,0],[.06,-5],[.16,22],[.30,-11],[.43,4.5],[.54,-1.5],[.62,0]];
function heartPose(x, y, t, strong = false) {
  const right = Math.max(0, Math.min(1, (x - 14) / 4));
  const strength = phrase(strong ? heartStrongKeys : heartKeys, t - right * (strong ? 0.03 : 0.02));
  const upper = Math.max(0, Math.min(1, (26.5 - y) / 20));
  // Strong opens the crown upward and outward; Light keeps its quiet squeeze.
  // The lower tip and paint widths stay fixed, with only 20/30 ms of lobe follow.
  const spread = 1 + strength * (strong ? 0.26 : 0.034) * upper;
  const height = strong ? 1 + strength * 0.15 * upper : 1 - strength * 0.014 * upper;
  const notch = Math.exp(-(((x - 16) / 2.4) ** 2)) * upper;
  return [
    16 + (x - 16) * spread,
    26.5 + (y - 26.5) * height + strength * 0.15 * notch,
  ];
}
function rotate(x, y, angle, pivot) {
  const a = (angle * Math.PI) / 180,
    c = Math.cos(a),
    s = Math.sin(a);
  return [
    pivot[0] + (x - pivot[0]) * c - (y - pivot[1]) * s,
    pivot[1] + (x - pivot[0]) * s + (y - pivot[1]) * c,
  ];
}
function bellPose(x, y, t, clapper = false, strong = false) {
  const keys = strong ? shellStrongKeys : shellKeys;
  const angle = phrase(keys, t);
  if (clapper) {
    const follow = phrase(keys, t - (strong ? 0.09 : 0.065)) * (strong ? 0.88 : 0.72);
    [x, y] = rotate(x, y, follow - angle, [16, 17.2]);
  }
  return rotate(x, y, angle, [16, 5.6]);
}
const circle = (x, y, r) =>
  `M${x} ${y - r} C${x + r * 0.552285} ${y - r} ${x + r} ${y - r * 0.552285} ${x + r} ${y} C${x + r} ${y + r * 0.552285} ${x + r * 0.552285} ${y + r} ${x} ${y + r} C${x - r * 0.552285} ${y + r} ${x - r} ${y + r * 0.552285} ${x - r} ${y} C${x - r} ${y - r * 0.552285} ${x - r * 0.552285} ${y - r} ${x} ${y - r} Z`;

// Original geometry. References inform construction rules, never path coordinates.
// Style is a paint contract; each visible contour shares the same authored mechanics.
// Curved notch and tip, with horizontal tangents at each turning point.
// The lower anchor stays at (16, 26.5); only its approach becomes gently rounded.
const heartContour = "M16 26.5 C15.72 26.5 15.38 26.17 14.82 25.75 C12.4 23.95 5.8 18.79 5.8 12.5 C5.8 9.18 8.04 6.8 11.1 6.8 C13.13 6.8 14.75 7.88 15.59 9.21 C15.73 9.43 15.85 9.55 16 9.55 C16.15 9.55 16.27 9.43 16.41 9.21 C17.25 7.88 18.87 6.8 20.9 6.8 C23.96 6.8 26.2 9.18 26.2 12.5 C26.2 18.79 19.6 23.95 17.18 25.75 C16.62 26.17 16.28 26.5 16 26.5 Z";
// The filled master is slightly narrower and has a deeper notch to balance its mass.
const heartSolid = "M16 26.5 C15.72 26.5 15.36 26.16 14.8 25.74 C12.39 23.93 5.95 18.85 5.95 12.5 C5.95 9.2 8.12 6.95 11.08 6.95 C13.14 6.95 14.82 8.21 15.61 9.62 C15.74 9.85 15.86 9.99 16 9.99 C16.14 9.99 16.26 9.85 16.39 9.62 C17.18 8.21 18.86 6.95 20.92 6.95 C23.88 6.95 26.05 9.2 26.05 12.5 C26.05 18.85 19.61 23.93 17.2 25.74 C16.64 26.16 16.28 26.5 16 26.5 Z";
// The rim curves carry their own radius, including the inside of the bold stroke.
const bellContour = "M16 8 C12.3 8 10 10.64 10 14 L10 16.2 C10 17.6 9.7 18 8.58995 19.11005 C7.70796 19.99204 8.3326 21.5 9.5799 21.5 L22.4201 21.5 C23.6674 21.5 24.29204 19.99204 23.41005 19.11005 C22.3 18 22 17.6 22 16.2 L22 14 C22 10.64 19.7 8 16 8 Z";
const bellSolid = "M16 7.55 C12.14 7.55 9.48 10.28 9.48 14 L9.48 16.2 C9.48 18.25 8.63 19.17 7.63388 20.16612 C6.8464 20.9536 7.4041 22.3 8.51777 22.3 L23.48223 22.3 C24.5959 22.3 25.1536 20.9536 24.36612 20.16612 C23.37 19.17 22.52 18.25 22.52 16.2 L22.52 14 C22.52 10.28 19.86 7.55 16 7.55 Z";
const stroke = (name, d, extra = {}) => ({ name, d, slot: "color.primary", style: "outline", type: "stroke", width: 2, ...extra });
const fill = (name, d, style, extra = {}) => ({ name, d, slot: "color.primary", style, ...extra });
const definitions = [
  {
    name: "heart", label: "Heart", rest: 0.45, active: 0.42,
    intent: "The heart gives one short acknowledgement beat and returns to its anchored resting pose.",
    keyPoses: [0, 0.045, 0.13, 0.15, 0.23, 0.30, 0.42],
    parts: [
      fill("heart-tint", heartContour, "tint"),
      stroke("heart-body", heartContour),
      fill("heart-solid", heartSolid, "solid"),
    ],
  },
  {
    name: "bell", label: "Bell", revision: "directed-v2", rest: .68, active: .65,
    intent: "The bell rings from its top mount while the clapper trails and settles.",
    keyPoses: [0,.06,.17,.32,.46,.58,.65],
    parts: [
      fill("bell-tint", bellContour, "tint"),
      stroke("bell-neck", "M16 5.6 L16 8"),
      stroke("bell-body", bellContour),
      stroke("bell-clapper", "M13.8 25.8 C14.5 27.25 17.5 27.25 18.2 25.8", { clapper: true }),
      stroke("bell-mount", "M15.75 5.6 L16.25 5.6", { fixed: true }),
      fill("bell-solid-neck", "M15.4 5.6 L16.6 5.6 C16.82 5.6 17 5.78 17 6 L17 8.1 C17 8.32 16.82 8.5 16.6 8.5 L15.4 8.5 C15.18 8.5 15 8.32 15 8.1 L15 6 C15 5.78 15.18 5.6 15.4 5.6 Z", "solid"),
      fill("bell-solid-body", bellSolid, "solid"),
      fill("bell-solid-clapper", "M14.25 24.8 L17.75 24.8 C18.07 24.8 18.3 25.02 18.26 25.34 C18.08 26.53 17.18 27.25 16 27.25 C14.82 27.25 13.92 26.53 13.74 25.34 C13.7 25.02 13.93 24.8 14.25 24.8 Z", "solid", { clapper: true }),
      fill("bell-solid-mount", circle(16, 5.6, 1), "solid", { fixed: true }),
    ],
  },
];
export { definitions };

// Variant fields the builder assigns on top of the base icon.
export function strongOverrides(base, strong) {
  if (strong) return base.name === 'heart' ? {
    rest: .52, active: .49, keyPoses: [0,.05,.15,.27,.37,.46,.49],
    intent: 'The heart squeezes, springs upward and outward into one decisive pulse, corrects twice with shrinking amplitude, and settles to its anchored resting pose.',
    heading: 'A heart you can feel.', description: 'A deeper squeeze. A bigger beat. A soft return.'
  } : {
    rest: .76, active: .72, keyPoses: [0,.06,.16,.30,.43,.54,.62,.72],
    intent: 'The bell swings wider while its clapper follows with more energy, then settles upright.',
    heading: 'A fuller ring.', description: 'A wider swing. A stronger response. Back to still.'
  };
  return base.name === 'heart' ? { heading: 'One clear beat.', description: 'A short response. A gentle return.' } : { heading: 'A considered ring.', description: 'The shell leads. The clapper follows.' };
}
// The builder's per-point pose for a part.
export function pose(icon, part, x, y, time, strong) {
  return part.fixed ? [x, y] : icon.icon === 'heart' ? heartPose(x, y, time, strong) : bellPose(x, y, time, part.clapper, strong);
}
