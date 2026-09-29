// Paint contract shared by every set builder. Each exported layer is named
// `<part>__<color slot>__<style>`; this binds its fills and strokes to the
// color slot, the style opacity slot and, for Regular outlines, the stroke-width
// token. verify-paint.mjs fails the catalog when any icon misses this contract.
export const STROKE_WIDTH_TOKEN = 'style.outline.width';
export const REGULAR_WEIGHT = 2;
// Opacity per style slot: Regular outline visible by default.
export const STYLE_SLOTS = {
  'style.outline.light': 0,
  'style.outline.regular': 100,
  'style.outline.bold': 0,
  'style.solid': 0,
  'style.tint': 0,
};

const rgb = hex => [1, 3, 5].map(i => parseInt(hex.slice(i, i + 2), 16) / 255);

// Rewrites `native` (an Engine export) in place: slot defaults, sid bindings
// and clean layer names. `palettes.dark` supplies the color slots.
export function bindPaint(native, palettes) {
  native.slots = Object.fromEntries(Object.entries(palettes.dark).map(([id, hex]) => [id, { p: { a: 0, k: rgb(hex) } }]));
  for (const [id, k] of Object.entries(STYLE_SLOTS)) native.slots[id] = { p: { a: 0, k } };
  // Any outline weight renders the Regular geometry with this token; the three
  // authored weights keep their own layers for pixel-exact output.
  native.slots[STROKE_WIDTH_TOKEN] = { p: { a: 0, k: REGULAR_WEIGHT } };
  for (const layer of native.layers) {
    const [name, slot, style] = layer.nm.split('__');
    if (!slot || !palettes.dark[slot] || !Object.hasOwn(native.slots, `style.${style}`)) throw new Error(`Missing paint contract: ${layer.nm}`);
    for (const shape of layer.shapes || []) {
      if (!['fl', 'st'].includes(shape.ty)) continue;
      shape.c.sid = slot;
      shape.o.sid = `style.${style}`;
      if (shape.ty === 'st' && style === 'outline.regular') shape.w.sid = STROKE_WIDTH_TOKEN;
    }
    layer.nm = name;
  }
  return native;
}
