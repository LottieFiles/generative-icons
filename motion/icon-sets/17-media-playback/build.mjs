// Set-owned native builder for mixed continuous, gesture and media-state icons.
// Earlier gesture builders and native artifacts are preserved.
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import svgpath from "svgpath";
const root = path.dirname(fileURLToPath(import.meta.url));
const config = JSON.parse(fs.readFileSync(path.join(root,"set-config.json")));
const { icons: definitions } = await import(path.join(root,"definitions.mjs"));
if (process.env.ICON_NAMES) throw Error("Build the complete set; partial manifests are not allowed.");
if (new Set(definitions.map(i=>i.name)).size !== definitions.length) throw Error("Duplicate icon name");
const library = path.resolve(root, "../..");
const out = path.resolve(library, `../prototype/icons/${config.id.slice(0,2)}/assets`);
const fps = 60;
const duration = 2.4;
const primary = "#00ddb3";
const round = (value) => Math.round(value * 1e5) / 1e5;
const sha = (value) => crypto.createHash("sha256").update(value).digest("hex");
for (const dir of ["scenes", "native", "evidence"])
  fs.mkdirSync(path.join(root, dir), { recursive: true });
fs.mkdirSync(out, { recursive: true });

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
  .replace('item.points[0][1],', 'item.basePoints || item.points[0][1],')
  .replace('keys(node, PropertyType.PathPoints, item.points, "points");', `if (item.transform) {
    const tr = item.transform;
    engine.setStaticVec2(node, PropertyType.AnchorPoint, {x:tr.anchor[0], y:tr.anchor[1]});
    if (tr.position.every(f => JSON.stringify(f[1]) === JSON.stringify(tr.position[0][1]))) {
      engine.setStaticVec2(node, PropertyType.Position, {x:tr.position[0][1][0], y:tr.position[0][1][1]});
    } else {
      const track = engine.insertTrack(node, PropertyType.Position, 0);
      for (const [time, value] of tr.position) {
        const key = engine.addKeyFrame(node, PropertyType.Position, track, time);
        engine.setKeyFrameVec2(key, {x:value[0], y:value[1]});
      }
    }
    keys(node, PropertyType.Rotation, tr.rotation);
  } else keys(node, PropertyType.PathPoints, item.points, "points");`)
  .replace("keys(paint, PropertyType.Opacity, item.opacity);", "keys(paint, PropertyType.Opacity, item.opacity);\n  keys(node, PropertyType.Opacity, item.visibility);")
  .replace(
    /return engine.renderFilmstrip\([\s\S]*$/,
    "return engine.renderFilmstrip({ tiles: 12, width: 32, height: 32 });\n",
  );

for(const icon of definitions)if(icon.duotoneFill!==undefined && icon.duotoneFill!==icon.parts.some(p=>p.style==='tint'))throw Error('Declared Duotone policy disagrees with drawing: '+icon.name);

// The pinned player accepts a scalar width slot without changing painted width.
// Export three native stroke weights instead; select their paint opacity at runtime.
// These are reviewed presets, not a continuous optical-size or width-profile axis.
for (const icon of definitions) icon.parts = icon.parts.flatMap(part =>
  part.style === "outline" ? [["light", 1.5], ["regular", 2], ["bold", 2.5]].map(([weight, width]) =>
    ({ ...part, name: `${part.name}-${weight}`, style: `outline.${weight}`, width })) : [part]);
const packageRecords = [];
for (const base of definitions) for (const motionId of ['light','strong']) for (const reverse of base.stateful ? [false,true] : [false]) {
  const strong=motionId==='strong', timing=(reverse?base.reverseTimings:base.timings)[motionId];
  const icon={...base,...timing,icon:base.name,motionId,motionLabel:strong?'Strong motion':'Light motion',direction:reverse?'reverse':'forward',name:base.name+(reverse?base.reverseSuffix:'')+(strong?'-strong':'')};
  // Named rest markers sit on whole frames after the authored action completes.
  icon.rest=Math.ceil(icon.rest*fps)/fps;
  const nodes=icon.parts.map(part=>{
    const sourceAt=time=>geometry(typeof part.d==='function'?part.d(time,strong,reverse,part.width):part.d);
    const first=sourceAt(0);
    const points=compact(Array.from({length:Math.round(duration*fps)+1},(_,frame)=>{
      const time=frame/fps,geo=sourceAt(time),points=[];
      if(JSON.stringify(geo.commands)!==JSON.stringify(first.commands))throw new Error('Changing topology: '+part.name);
      for(let i=0;i<geo.points.length;i+=2)points.push(...(part.pose?part.pose(geo.points[i],geo.points[i+1],time,strong,reverse):geo.points.slice(i,i+2)).map(round));
      return [time,points];
    }));
    let nativeTransform = {};
    // Preserve rigid geometry as an editable native transform. Keep evaluated
    // world points in the source scene as an independent verification reference.
    if (part.pose?.transform && typeof part.d === 'string') {
      const samples = Array.from({length:Math.round(duration*fps)+1},(_,frame)=>{
        const time=frame/fps, {dx=0,dy=0,angle=0,pivot=[16,16]}=part.pose.transform(time,strong,reverse);
        return {time,dx,dy,angle,pivot};
      });
      const anchor=samples[0].pivot;
      if(samples.some(s=>JSON.stringify(s.pivot)!==JSON.stringify(anchor)))throw Error('Moving rigid pivot');
      nativeTransform={basePoints:first.points.map(round),transform:{anchor,
        position:compact(samples.map(s=>[s.time,[anchor[0]+s.dx,anchor[1]+s.dy]])),
        // Installed Engine exporter negates its radians when writing Lottie degrees.
        rotation:compact(samples.map(s=>[s.time,-s.angle*Math.PI/180]))}};
    }
    return {name:`${part.name}__color.primary__${part.style}`,commands:first.commands,points,...nativeTransform,type:part.type||'fill',color:primary,opacity:[[0,part.style==='outline.regular'?1:0],[duration,part.style==='outline.regular'?1:0]],width:[[0,part.width||1],[duration,part.width||1]],visibility:compact(Array.from({length:Math.round(duration*fps)+1},(_,frame)=>[frame/fps,round(part.visibility?part.visibility(frame/fps,strong,reverse):1)]))};
  });
  const scene={name:icon.name,width:32,height:32,fps,duration,rest:icon.rest,nodes};
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
  native.nm = `Motion / ${config.name} / ${icon.label}`;
  native.slots = Object.fromEntries(
    Object.entries(palettes.dark).map(([id, hex]) => [
      id,
      { p: { a: 0, k: rgb(hex) } },
    ]),
  );
  Object.assign(native.slots, {
    "style.outline.light": { p: { a: 0, k: 0 } },
    "style.outline.regular": { p: { a: 0, k: 100 } },
    "style.outline.bold": { p: { a: 0, k: 0 } },
    "style.solid": { p: { a: 0, k: 0 } },
    "style.tint": { p: { a: 0, k: 0 } },
  });
  for (const layer of native.layers) {
    const [name, slot, style] = layer.nm.split("__");
    if (!slot || !palettes.dark[slot] || !Object.hasOwn(native.slots, `style.${style}`))
      throw new Error(`Missing paint contract: ${layer.nm}`);
    for (const shape of layer.shapes || []) {
      if (!["fl", "st"].includes(shape.ty)) continue;
      shape.c.sid = slot;
      shape.o.sid = `style.${style}`;
    }
    layer.nm = name;
  }
  native.markers = [
    { cm: "action", tm: 0, dr: icon.active * fps },
    { cm: "rest", tm: icon.rest * fps, dr: 1 },
    { cm: "showcase", tm: 0, dr: duration * fps },
  ];
  native.meta = {
    generator: "Engine MCP native export",
    set: config.id,
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
    continuous: Boolean(base.continuous),
    previewReset: icon.previewReset || false,
    cycleDegrees: icon.cycleDegrees || 0,
    directionSources: Object.fromEntries((config.sharedSources || ["drawing.mjs", "direction.mjs"]).map(n=>[n,sha(fs.readFileSync(path.join(root,"../library",n)))])),
    motionLabel: icon.motionLabel,
    direction: icon.direction,
    stateful: icon.stateful,
    startState: icon.stateful?(reverse?icon.finalState:icon.initialState):"rest",
    endState: icon.stateful?(reverse?icon.initialState:icon.finalState):"rest",
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
    sourceSha256: sha(fs.readFileSync(path.join(root,"build.mjs"))),
    setBuilderSha256: sha(fs.readFileSync(fileURLToPath(import.meta.url))),
    authoringSources: Object.fromEntries(["set-config.json", ...fs.readdirSync(path.join(root,"batches")).filter(n=>n.endsWith(".mjs")).map(n=>"batches/"+n)].map(n=>[n,sha(fs.readFileSync(path.join(root,n)))])),
    brief: icon.brief,
    constraints: icon.constraints || [],
    category: config.name,
    definitionsSha256: sha(fs.readFileSync(path.join(root,"definitions.mjs"))),
    primary: { dark: primary, light: "#008c76" },
    duotoneFill: icon.parts.some(part => part.style === "tint"),
    styles: ["outline", "solid", "duotone"],
    strokeWidth: { min: 1.5, default: 2, max: 2.5, presets: [1.5, 2, 2.5], units: "32-unit construction grid" },
  });
  console.log(`${icon.label} / ${icon.motionLabel}: ${native.layers.length} native layers exported`);
}
fs.writeFileSync(path.join(out,'set.json'),JSON.stringify({name:config.name,id:config.id,category:config.name,version:'1.0',defaultStyle:'outline',defaultMotion:'strong',runtime:'@lottiefiles/dotlottie-web@0.80.0',icons:definitions.map(icon=>{
  const animations=packageRecords.filter(r=>r.icon===icon.name),motions=animations.filter(r=>r.direction==='forward').map(r=>({...r,...(icon.stateful?{reverse:animations.find(a=>a.motionId===r.motionId&&a.direction==='reverse')}: {})}));
  return {...motions[0],number:icon.number,forwardLabel:icon.forwardLabel,reverseLabel:icon.reverseLabel,motions,animations};
})},null,2)+'\n');
const packaged=spawnSync('python3',[path.join(root,'package.py')],{stdio:'inherit'});if(packaged.status!==0)process.exit(packaged.status);
