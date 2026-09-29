// Original icon geometry and pose animation. Engine MCP owns the Lottie export.
// Run from any directory: node motion/icon-sets/02-search/build.mjs
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import svgpath from "svgpath";
import { bindPaint } from "../library/paint-tokens.mjs";

const root = path.dirname(fileURLToPath(import.meta.url));
const library = path.resolve(root, "../..");
const out = path.resolve(library, "../prototype/icons/02/assets");
const fps = 60;
const duration = 2.4;
const primary = "#00ddb3";
const round = (value) => Math.round(value * 1e5) / 1e5;
const sha = (value) => crypto.createHash("sha256").update(value).digest("hex");
for (const dir of ["scenes", "native", "evidence"])
  fs.mkdirSync(path.join(root, dir), { recursive: true });
fs.mkdirSync(out, { recursive: true });

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
function geometry(d) {
  const commands = [],
    points = [];
  let last = [0, 0];
  for (const [cmd, ...args] of svgpath(d).abs().unarc().unshort().segments) {
    if (cmd === "M" || cmd === "L") {
      commands.push(cmd === "M" ? 0 : 1);
      points.push(...args);
      last = args;
    } else if (cmd === "C") {
      commands.push(2);
      points.push(...args);
      last = args.slice(-2);
    } else if (cmd === "H") {
      commands.push(1);
      last = [args[0], last[1]];
      points.push(...last);
    } else if (cmd === "V") {
      commands.push(1);
      last = [last[0], args[0]];
      points.push(...last);
    } else if (cmd === "Z") commands.push(3);
    else throw new Error(`Unsupported command ${cmd}`);
  }
  return { commands, points };
}
function compact(frames) {
  const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
  return frames.filter(
    (frame, i) =>
      i === 0 ||
      i === frames.length - 1 ||
      !same(frame[1], frames[i - 1][1]) ||
      !same(frame[1], frames[i + 1][1]),
  );
}
const template = fs
  .readFileSync(path.join(library, "src/engine-scene.js"), "utf8")
  .replace("/* SCENE_DATA */", "const scene = params.scene;")
  .replace(
    /return engine.renderFilmstrip\([\s\S]*$/,
    "return engine.renderFilmstrip({ tiles: 12, width: 32, height: 32 });\n",
  );

// The pinned player accepts a scalar width slot without changing painted width.
// Export three native stroke weights instead; select their paint opacity at runtime.
// These are reviewed presets, not a continuous optical-size or width-profile axis.
for (const icon of definitions) icon.parts = icon.parts.flatMap(part =>
  part.style === "outline" ? [["light", 1.5], ["regular", 2], ["bold", 2.5]].map(([weight, width]) =>
    ({ ...part, name: `${part.name}-${weight}`, style: `outline.${weight}`, width })) : [part]);
const packageRecords = [];
for (const base of definitions) for (const motionId of ['light','strong']) {
  const strong = motionId === 'strong';
  const icon = { ...base, icon: base.name, motionId, motionLabel: strong ? 'Strong motion' : 'Light motion', name: base.name + (strong ? '-strong' : '') };
  if (strong) Object.assign(icon, {
    rest:.70,active:.66,keyPoses:[0,.065,.15,.235,.38,.50,.66],
    intent:'The magnifier looks left, sweeps decisively right with a springy overshoot, then springs back through center to its resting pose.',
    heading:'A wider sweep.',description:'Look further. Sweep with spring. Snap back to center.'
  });
  const nodes = icon.parts.map((part) => {
    const geo = geometry(part.d);
    const points = compact(
      Array.from({ length: Math.round(duration * fps) + 1 }, (_, frame) => {
        const time = frame / fps,
          p = [];
        for (let i = 0; i < geo.points.length; i += 2) {
          const x = geo.points[i],
            y = geo.points[i + 1];
          const result = searchPose(x,y,time,strong);
          p.push(...result.map(round));
        }
        return [time, p];
      }),
    );
    return {
      name: `${part.name}__${part.slot}__${part.style}`,
      commands: geo.commands,
      points,
      type: part.type || "fill",
      color: primary,
      opacity: [
        [0, part.style === "outline.regular" ? 1 : 0],
        [duration, part.style === "outline.regular" ? 1 : 0],
      ],
      width: [
        [0, part.width || 1],
        [duration, part.width || 1],
      ],
    };
  });
  const scene = {
    name: icon.name,
    width: 32,
    height: 32,
    fps,
    duration,
    rest: icon.rest,
    nodes,
  };
  const scriptFile = path.join(root, `scenes/${icon.name}.engine.js`);
  const sceneFile = path.join(root, `scenes/${icon.name}.scene.json`);
  const nativeFile = path.join(root, `native/${icon.name}.json`);
  fs.writeFileSync(scriptFile, template);
  fs.writeFileSync(sceneFile, JSON.stringify(scene));
  const exported = spawnSync(
    process.execPath,
    [path.join(library, "src/export.mjs"), scriptFile, nativeFile, sceneFile],
    { encoding: "utf8" },
  );
  if (exported.status !== 0)
    throw new Error(exported.stderr || exported.stdout);
  const native = JSON.parse(fs.readFileSync(nativeFile));
  const palettes = {
    dark: {
      "color.primary": primary,

    },
    light: {
      "color.primary": "#008c76",

    },
  };
  const rgb = (hex) =>
    [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  native.nm = `Motion / Set 02 / ${icon.label}`;
  bindPaint(native, palettes);
  native.markers = [
    { cm: "action", tm: 0, dr: icon.active * fps },
    { cm: "rest", tm: icon.rest * fps, dr: 1 },
    { cm: "showcase", tm: 0, dr: duration * fps },
  ];
  native.meta = {
    generator: "Engine MCP native export",
    set: "02-search",
    restFrame: icon.rest * fps,
  };
  fs.writeFileSync(path.join(out, `${icon.name}.json`), JSON.stringify(native));
  const manifest = {
    version: "2",
    generator: "LottieFiles Motion / Engine MCP",
    initial: { animation: icon.name },
    animations: [
      { id: icon.name, initialTheme: "dark", themes: ["dark", "light"] },
    ],
    themes: [{ id: "dark" }, { id: "light" }],
  };
  fs.writeFileSync(
    path.join(out, `${icon.name}.manifest.json`),
    JSON.stringify(manifest),
  );
  for (const [theme, palette] of Object.entries(palettes))
    fs.writeFileSync(
      path.join(out, `${icon.name}.theme-${theme}.json`),
      JSON.stringify({
        rules: [
          ...Object.entries(palette).map(([id, hex]) => ({ id, type: "Color", value: rgb(hex) })),
          ...Object.entries(native.slots).filter(([id]) => id !== "color.primary").map(([id, slot]) => ({ id, type: "Scalar", value: slot.p.k })),
        ],
      }),
    );
  packageRecords.push({
    name: icon.name,
    icon: icon.icon,
    motionId: icon.motionId,
    revision: icon.revision,
    motionLabel: icon.motionLabel,
    heading: icon.heading,
    description: icon.description,
    label: icon.label,
    intent: icon.intent,
    width: 32,
    height: 32,
    fps,
    frames: duration * fps,
    duration,
    active: icon.active,
    rest: icon.rest * fps,
    keyPoses: icon.keyPoses,
    layers: native.layers.length,
    slots: Object.keys(native.slots),
    nativeSha256: sha(fs.readFileSync(nativeFile)),
    sourceSha256: sha(fs.readFileSync(fileURLToPath(import.meta.url))),
    primary: { dark: primary, light: "#008c76" },
    styles: ["outline", "solid", "duotone"],
    strokeWidth: { min: 1.5, default: 2, max: 2.5, presets: [1.5, 2, 2.5], units: "32-unit construction grid" },
  });
  console.log(`${icon.label} / ${icon.motionLabel}: ${native.layers.length} native layers exported`);
}
fs.writeFileSync(
  path.join(out, "set.json"),
  JSON.stringify(
    {
      name: "Search",
      id: "02-search",
      version: "2.0",
      defaultStyle: "outline",
      defaultMotion: "light",
      runtime: "@lottiefiles/dotlottie-web@0.80.0",
      icons: definitions.map(icon => {
        const motions = packageRecords.filter(record => record.icon === icon.name);
        return { ...motions[0], motions };
      }),
    },
    null,
    2,
  ) + "\n",
);
const packaged = spawnSync("python3", [path.join(root, "package.py")], {
  stdio: "inherit",
});
if (packaged.status !== 0) process.exit(packaged.status);
