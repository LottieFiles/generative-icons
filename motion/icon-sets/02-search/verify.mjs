import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import crypto from "node:crypto";
import assert from "node:assert/strict";
const root = path.dirname(fileURLToPath(import.meta.url));
const out = path.resolve(root, "../../../prototype/icons/02/assets");
const hash = (value) => crypto.createHash("sha256").update(value).digest("hex");
const set = JSON.parse(fs.readFileSync(path.join(out, "set.json")));
assert.deepEqual(
  set.icons.map((icon) => icon.name),
  ["search"],
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
  // A single rigid transform must preserve every pairwise distance, including
  // the connection between the separate lens and handle paths.
  const moving=scene.nodes.filter(n=>n.name.startsWith('search-body-regular')||n.name.startsWith('search-handle-regular'));
  const origin=moving.flatMap(n=>n.points[0][1]);
  for(let frame=0;frame<moving[0].points.length;frame++) {
    const current=moving.flatMap(n=>n.points[frame][1]);
    for(let i=0;i<current.length;i+=2) for(let j=i+2;j<current.length;j+=2)
      assert.ok(Math.abs(Math.hypot(current[i]-current[j],current[i+1]-current[j+1])-Math.hypot(origin[i]-origin[j],origin[i+1]-origin[j+1]))<.00004,'Search assembly bends or separates');
  }
  console.log(
    `PASS ${icon.name}: native provenance, vector geometry, stable topology, authored mechanics, loop, markers, and package hash`,
  );
}

// Preserve the rounded drawing while refining the scan.
for (const icon of set.icons) {
  assert.deepEqual(icon.motions.map(m => m.motionId), ['light','strong']);
  const [light,strong] = icon.motions.map(meta => JSON.parse(fs.readFileSync(path.join(root, `scenes/${meta.name}.scene.json`))));
  const previous=JSON.parse(fs.readFileSync(path.join(root,`research-review/before/scenes/${icon.name}.scene.json`)));
  assert.ok(icon.motions[0].active <= .5,'Frequent feedback exceeds the chosen half-second budget');
  for (let i=0;i<light.nodes.length;i++) {
    assert.deepEqual(light.nodes[i].points[0][1],previous.nodes[i].points[0][1],'Research revision changed the rounded drawing');
    assert.deepEqual(light.nodes[i].commands,previous.nodes[i].commands,'Research revision changed topology');
    assert.deepEqual(light.nodes[i].points[0][1],strong.nodes[i].points[0][1],'Variant rest shapes differ');
    assert.deepEqual(light.nodes[i].commands,strong.nodes[i].commands,'Variant topology differs');
  }
  const moving = nodes => nodes.find(node => node.name.startsWith(icon.name+'-body'));
  const excursion = node => Math.max(...node.points.flatMap(([,points]) => points.map((p,i)=>Math.abs(p-node.points[0][1][i]))));
  assert.ok(excursion(moving(strong.nodes)) > excursion(moving(light.nodes))*1.5,'Strong motion is not substantially stronger');
  console.log(`PASS ${icon.name}: shared rest shape, stronger rigid scan`);
}
