// gai:legacy/02: the authoring half of motion/icon-sets/02-search/build.mjs, which has no definitions.mjs.
// Lines 23-79 follow verbatim (a drift test compares them); strongOverrides and pose restate lines 133-138 and 148.
import svgpath from 'svgpath';
function phrase(keys, time) {
  if (time <= keys[0][0]) return keys[0][1];
  if (time >= keys.at(-1)[0]) return keys.at(-1)[1];
  const index = keys.findIndex((key) => key[0] > time);
  const a = keys[index - 1],
    b = keys[index];
  const u = (1 - Math.cos((Math.PI * (time - a[0])) / (b[0] - a[0]))) / 2;
  return a[1] + (b[1] - a[1]) * u;
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
const circle = (x, y, r) =>
  `M${x} ${y - r} C${x + r * 0.552285} ${y - r} ${x + r} ${y - r * 0.552285} ${x + r} ${y} C${x + r} ${y + r * 0.552285} ${x + r * 0.552285} ${y + r} ${x} ${y + r} C${x - r * 0.552285} ${y + r} ${x - r} ${y + r * 0.552285} ${x - r} ${y} C${x - r} ${y - r * 0.552285} ${x - r * 0.552285} ${y - r} ${x} ${y - r} Z`;

// A rigid magnifier, constructed from our own circle and capsule geometry.
// The ring and handle use one transform: their attachment never separates.
// The lens and handle share a single physical sweep.
const lightScan = [[0,0],[.07,-.5],[.19,1],[.26,1],[.41,0]];
// Strong is extrema-only: look left, decisive sweep with overshoot, one center correction, settle.
const strongScan = [[0,0],[.065,-1],[.235,1.14],[.50,-.16],[.65,0]];
function searchPose(x,y,t,strong=false) {
  const phase=phrase(strong ? strongScan : lightScan,t);
  const distance=strong ? 2.1 : .8;
  const angle=phase*(strong ? 8 : 2.5);
  // Rotate about the lens while it translates. A grip-centered rotation added
  // to the scan translation and pushed the bold rim too close to the edge.
  [x,y]=rotate(x,y,angle,[13.8,13.8]);
  // A quadratic arc has a continuous tangent as the scan crosses its center.
  return [x+phase*distance,y-phase*phase*(strong ? .30 : .16)];
}
// The Engine path API requires Close to be the last command. Join the outer
// and reverse inner contours by a zero-width seam to keep a transparent hole
// in one closed vector path. Both contours remain complete cubic circles.
const ring=circle(13.8,13.8,7.6);
const solidRing=circle(13.8,13.8,8.15).replace(/Z$/,'')+' '+svgpath(circle(13.8,13.8,5.65)).scale(-1,1).translate(27.6,0).toString().replace(/^M/,'L');
const handle='M19.174 19.174 L25.15 25.15';
const solidHandle=svgpath('M0 -1.3 L8.451 -1.3 C9.169 -1.3 9.751 -.718 9.751 0 C9.751 .718 9.169 1.3 8.451 1.3 L0 1.3 C-.718 1.3 -1.3 .718 -1.3 0 C-1.3 -.718 -.718 -1.3 0 -1.3 Z').rotate(45).translate(19.174,19.174).toString();
const definitions=[{
 name:'search',label:'Search',revision:'directed-v2',rest:.48,active:.44,
 intent:'The magnifier makes one short left-to-right sweep, then returns to its resting pose.',
 heading:'One clear sweep.',description:'Look left. Sweep right. Return to center.',
 keyPoses:[0,.07,.15,.24,.34,.44,.46],
 parts:[
  {name:'search-tint',d:ring,style:'tint'},
  {name:'search-handle',d:handle,style:'outline',type:'stroke',width:2},
  {name:'search-body',d:ring,style:'outline',type:'stroke',width:2},
  {name:'search-solid-handle',d:solidHandle,style:'solid'},
  {name:'search-solid-body',d:solidRing,style:'solid'},
 ].map(p=>({...p,slot:'color.primary'}))
}];
export { definitions };

// Variant fields the builder assigns on top of the base icon.
export function strongOverrides(base, strong) {
  return strong ? {
    rest:.70,active:.66,keyPoses:[0,.065,.15,.235,.38,.50,.66],
    intent:'The magnifier looks left, sweeps decisively right with a springy overshoot, then springs back through center to its resting pose.',
    heading:'A wider sweep.',description:'Look further. Sweep with spring. Snap back to center.'
  } : {};
}
// The builder's per-point pose for a part.
export function pose(icon, part, x, y, time, strong) { return searchPose(x, y, time, strong); }
