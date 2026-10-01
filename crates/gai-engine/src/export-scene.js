// removeComp, not deleteNodes: deleting a composition node fails the document
// snapshot ("snapshot serialization failed") on creator-mcp 0.2.0.
for (const c of creator.describeComps()) creator.removeComp(c.id);
const scene = params.scene;
const comp = creator.createComp(scene.width, scene.height, scene.fps, 0, scene.duration);
creator.focusOn(comp);
creator.setName(comp, scene.name);
function rgb(hex) {
  const n = parseInt(hex.slice(1), 16);
  return { red: ((n >> 16) & 255) / 255, green: ((n >> 8) & 255) / 255, blue: (n & 255) / 255, alpha: 1 };
}
const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
function keys(node, property, frames, kind) {
  if (frames.every((f) => same(f[1], frames[0][1]))) {
    if (kind === "points") return;
    creator.setStaticF32(node, property, frames[0][1]);
    return;
  }
  const track = creator.insertTrack(node, property, 0);
  for (const [time, value] of frames) {
    const key = creator.addKeyFrame(node, property, track, time);
    if (kind === "points") creator.setKeyFrameVec2Slice(key, value);
    else creator.setKeyFrameF32(key, value);
  }
}
// One composition-level group per part: the group carries the rigid transform
// and the part's visibility; the path inside carries geometry and paint.
for (const item of scene.nodes) {
  const group = creator.createGroup(comp, 0, 0);
  creator.setName(group, item.name);
  const node = creator.insertPath(comp, 0, 0, item.commands, item.basePoints || item.points[0][1], rgb(item.color));
  creator.setParent(node, group);
  creator.setName(node, item.name);
  creator.setNodeData(node, item.name);
  let paint = creator.getPaints(node)[0];
  if (item.type === "stroke") {
    creator.removePaint(paint);
    paint = creator.addStroke(node, rgb(item.color), item.width[0][1]);
    creator.setStrokeCap(paint, 2);
    creator.setStrokeJoin(paint, 2);
    keys(paint, PropertyType.Width, item.width);
  }
  if (item.transform) {
    const tr = item.transform;
    creator.setStaticVec2(group, PropertyType.AnchorPoint, { x: tr.anchor[0], y: tr.anchor[1] });
    if (tr.position.every((f) => same(f[1], tr.position[0][1]))) {
      creator.setStaticVec2(group, PropertyType.Position, { x: tr.position[0][1][0], y: tr.position[0][1][1] });
    } else {
      const track = creator.insertTrack(group, PropertyType.Position, 0);
      for (const [time, value] of tr.position) {
        const key = creator.addKeyFrame(group, PropertyType.Position, track, time);
        creator.setKeyFrameVec2(key, { x: value[0], y: value[1] });
      }
    }
    keys(group, PropertyType.Rotation, tr.rotation);
  } else keys(node, PropertyType.PathPoints, item.points, "points");
  keys(paint, PropertyType.Opacity, item.opacity);
  keys(group, PropertyType.Opacity, item.visibility);
}
return creator.export(params.file, "lottie");
