import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import crypto from "node:crypto";
import assert from "node:assert/strict";
const root = path.dirname(fileURLToPath(import.meta.url));
const out = path.resolve(root, "../../../prototype/icons/01/assets");
const hash = (value) => crypto.createHash("sha256").update(value).digest("hex");
const set = JSON.parse(fs.readFileSync(path.join(out, "set.json")));
assert.deepEqual(
  set.icons.map((icon) => icon.name),
  ["heart", "bell"],
);
for (const icon of set.icons.flatMap(icon => icon.motions)) {
  const proof = JSON.parse(
    fs.readFileSync(
      path.join(root, `native/${icon.name}.json.provenance.json`),
    ),
  );
  for (const [file, expected] of [
    [`native/${icon.name}.json`, proof.sha256],
    [`scenes/${icon.name}.engine.js`, proof.scriptSha256],
    [`scenes/${icon.name}.scene.json`, proof.sceneSha256],
  ])
    assert.equal(hash(fs.readFileSync(path.join(root, file))), expected);
  assert.equal(
    hash(fs.readFileSync(path.join(root, "build.mjs"))),
    icon.sourceSha256,
  );
  assert.equal(
    hash(fs.readFileSync(path.join(out, `${icon.name}.lottie`))),
    icon.sha256,
  );
  const raw = JSON.parse(
    fs.readFileSync(path.join(root, `native/${icon.name}.json`)),
  );
  const data = JSON.parse(fs.readFileSync(path.join(out, `${icon.name}.json`)));
  assert.equal(data.w, 32);
  assert.equal(data.h, 32);
  assert.equal(data.fr, 60);
  assert.equal(data.op, 144);
  assert.ok(data.layers.every((layer) => layer.ty === 4));
  assert.deepEqual(icon.styles, ["outline", "solid", "duotone"]);
  assert.deepEqual(icon.strokeWidth.presets, [1.5, 2, 2.5]);
  const strokeWidths = new Set();
  for (const layer of data.layers) for (const paint of layer.shapes.filter(s => ["st", "fl"].includes(s.ty))) {
    assert.ok(data.slots[paint.c.sid], "Unbound color");
    assert.ok(data.slots[paint.o.sid], "Unbound style");
    if (paint.ty === "st") { strokeWidths.add(paint.w.k); assert.equal(paint.lc, 2); assert.equal(paint.lj, 2); }
  }
  assert.deepEqual([...strokeWidths].sort(), [1.5, 2, 2.5]);
  for (const theme of ["dark", "light"]) {
    const rules = JSON.parse(fs.readFileSync(path.join(out, `${icon.name}.theme-${theme}.json`))).rules;
    assert.deepEqual(rules.map(r => r.id).sort(), Object.keys(data.slots).sort(), "Theme drops token registry");
  }

  assert.deepEqual(
    data.layers.map((layer) => layer.ks),
    raw.layers.map((layer) => layer.ks),
    "Packaging changed motion",
  );
  assert.deepEqual(
    data.layers.map((layer) =>
      layer.shapes.filter((shape) => shape.ty === "sh"),
    ),
    raw.layers.map((layer) =>
      layer.shapes.filter((shape) => shape.ty === "sh"),
    ),
    "Packaging changed path animation",
  );
  assert.ok(
    data.markers.some(
      (marker) => marker.cm === "rest" && marker.tm === icon.rest,
    ),
  );
  assert.ok(data.markers.some((marker) => marker.cm === "action"));
  const scene = JSON.parse(
    fs.readFileSync(path.join(root, `scenes/${icon.name}.scene.json`)),
  );
  for (const node of scene.nodes) {
    const points = node.points;
    assert.deepEqual(points[0][1], points.at(-1)[1], "Reset changes geometry");
    assert.ok(
      points.every((point) => point[1].length === points[0][1].length),
      "Morph topology changed",
    );
    for (const [, point] of points)
      for (const coordinate of point) assert.ok(Number.isFinite(coordinate));
  }
  if (icon.icon === "heart")
    for (const [, points] of scene.nodes.find(node => node.name.startsWith("heart-body")).points)
      assert.deepEqual(points.slice(0, 2), [16, 26.5], "Heart tip moves");
  if (icon.icon === "bell") {
    const body = scene.nodes.find((node) => node.name.startsWith("bell-body"));
    const origin = body.points[0][1];
    // A rigid shell preserves every control point's distance to its top pivot.
    for (const [, points] of body.points)
      for (let i = 0; i < points.length; i += 2)
        assert.ok(
          Math.abs(
            Math.hypot(points[i] - 16, points[i + 1] - 5.6) -
              Math.hypot(origin[i] - 16, origin[i + 1] - 5.6),
          ) < 0.00003,
          "Bell shell deforms",
        );
    const mount = scene.nodes.find((node) =>
      node.name.startsWith("bell-mount"),
    );
    assert.ok(
      mount.points.every(
        (point) =>
          JSON.stringify(point[1]) === JSON.stringify(mount.points[0][1]),
      ),
      "Top mount moves",
    );
  }
  console.log(
    `PASS ${icon.name}: native provenance, vector geometry, stable topology, authored mechanics, loop, markers, and package hash`,
  );
}

// Research changes motion only. Preserve both drawings and every Bell frame.
for (const icon of set.icons) {
  assert.deepEqual(icon.motions.map(m => m.motionId), ['light','strong']);
  const [light,strong] = icon.motions.map(meta => JSON.parse(fs.readFileSync(path.join(root, `scenes/${meta.name}.scene.json`))));
  const previous = JSON.parse(fs.readFileSync(path.join(root, `research-review/before/scenes/${icon.name}.scene.json`)));
  for (let i=0;i<light.nodes.length;i++) {
    assert.deepEqual(light.nodes[i].points[0][1],previous.nodes[i].points[0][1],'Research revision changed the rounded drawing');
    assert.deepEqual(light.nodes[i].commands,previous.nodes[i].commands,'Research revision changed topology');
  }
  if (icon.name === 'heart') assert.ok(icon.motions[0].active <= .5,'Frequent feedback exceeds the chosen half-second budget');
  for (let i=0;i<light.nodes.length;i++) {
    assert.deepEqual(light.nodes[i].points[0][1],strong.nodes[i].points[0][1],'Variant rest shapes differ');
    assert.deepEqual(light.nodes[i].commands,strong.nodes[i].commands,'Variant topology differs');
  }
  const moving = nodes => nodes.find(node => node.name.startsWith(icon.name+'-body'));
  const excursion = node => Math.max(...node.points.flatMap(([,points]) => points.map((p,i)=>Math.abs(p-node.points[0][1][i]))));
  assert.ok(excursion(moving(strong.nodes)) > excursion(moving(light.nodes))*1.5,'Strong motion is not substantially stronger');
  console.log(`PASS ${icon.name}: rounded drawing retained, shared rest shape, stronger geometric motion`);
}

// The current whole-library Strong revision supersedes the old Heart protection.
// Light geometry/motion remains byte-identical; family archives also contain Strong.
const baseline=JSON.parse(fs.readFileSync(path.join(root,'evidence/light-baseline.json')));
for(const name of ['heart','bell']){
 const rel=`prototype/icons/01/assets/${name}.json`;
 assert.equal(hash(fs.readFileSync(path.join(out,`${name}.json`))),baseline[rel],'Light native action changed');
}
console.log('PASS current Light preservation: Heart and Bell native actions unchanged');
