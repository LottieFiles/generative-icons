/* SCENE_DATA */
const comp = engine.createComp(
  scene.width,
  scene.height,
  scene.fps,
  0,
  scene.duration,
);
engine.focusOn(comp);
engine.setName(comp, scene.name);
function rgb(hex) {
  const n = parseInt(hex.slice(1), 16);
  return {
    red: ((n >> 16) & 255) / 255,
    green: ((n >> 8) & 255) / 255,
    blue: (n & 255) / 255,
    alpha: 1,
  };
}
function keys(node, property, frames, kind) {
  if (
    frames.every((f) => JSON.stringify(f[1]) === JSON.stringify(frames[0][1]))
  ) {
    if (kind === "points") return;
    engine.setStaticF32(node, property, frames[0][1]);
    return;
  }
  const track = engine.insertTrack(node, property, 0);
  for (const [time, value] of frames) {
    const key = engine.addKeyFrame(node, property, track, time);
    if (kind === "points") engine.setKeyFrameVec2Slice(key, value);
    else engine.setKeyFrameF32(key, value);
  }
}
// Flat painted paths avoid unsupported nested paint inheritance in PR #322.
// The full eased parent transforms have already been evaluated in source data.
for (const item of scene.nodes) {
  const node = engine.insertPath(
    comp,
    0,
    0,
    item.commands,
    item.points[0][1],
    rgb(item.color),
  );
  engine.setName(node, item.name);
  engine.setNodeData(node, item.name);
  let paint = engine.getPaints(node)[0];
  if (item.type === "stroke") {
    engine.removePaint(paint);
    paint = engine.addStroke(node, rgb(item.color), item.width[0][1]);
    engine.setStrokeCap(paint, 2);
    engine.setStrokeJoin(paint, 2);
    keys(paint, PropertyType.Width, item.width);
  }
  if (item.gradient) {
    const g = item.gradient;
    engine.convertPaintToGradient(paint, g.kind);
    engine.addGradientPoint(paint, 1);
    engine.setStaticF32Slice(paint, PropertyType.Offsets, g.offsets);
    engine.setStaticSrgbColorSlice(paint, PropertyType.Colors, g.colors);
    engine.setStaticVec2(paint, PropertyType.AbsoluteStartPoint, {
      x: g.start[0],
      y: g.start[1],
    });
    engine.setStaticVec2(paint, PropertyType.AbsoluteEndPoint, {
      x: g.end[0],
      y: g.end[1],
    });
  }
  keys(node, PropertyType.PathPoints, item.points, "points");
  keys(paint, PropertyType.Opacity, item.opacity);
}
return engine.renderFilmstrip({
  tiles: 8,
  width: Math.min(scene.width, 320),
  height: Math.round((scene.height * Math.min(scene.width, 320)) / scene.width),
});
