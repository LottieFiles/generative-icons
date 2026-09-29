// Bridge for outline weights between the authored stops. Every Regular stroke
// carries the style.outline.width token, but the bundled dotLottie player does
// not yet apply stroke-width tokens, so the width is written into the .lottie
// bytes before the player loads them. Remove once players honour the token.
export const STROKE_WIDTH_TOKEN = 'style.outline.width';

const encoder = new TextEncoder(), decoder = new TextDecoder();

// Rewrites every token-bound stroke width (and the token's defaults) to `weight`.
export async function rewriteStrokeWidth(bytes, weight) {
  const entries = await readZip(bytes);
  for (const entry of entries) {
    if (!/\.json$/.test(entry.name)) continue;
    const json = JSON.parse(decoder.decode(entry.data));
    if (entry.name.startsWith('a/')) {
      for (const layer of json.layers || []) rewriteShapes(layer.shapes, weight);
      if (json.slots?.[STROKE_WIDTH_TOKEN]) json.slots[STROKE_WIDTH_TOKEN] = { p: { a: 0, k: weight } };
    } else if (entry.name.startsWith('t/')) {
      for (const rule of json.rules || []) if (rule.id === STROKE_WIDTH_TOKEN) rule.value = weight;
    } else continue;
    entry.data = encoder.encode(JSON.stringify(json));
  }
  return writeZip(entries);
}

function rewriteShapes(shapes, weight) {
  for (const shape of shapes || []) {
    if (shape.ty === 'gr') rewriteShapes(shape.it, weight);
    // Family strokes are constant-width; an animated width keeps its keyframes untouched.
    else if (shape.ty === 'st' && shape.w?.sid === STROKE_WIDTH_TOKEN && !shape.w.a) shape.w = { ...shape.w, k: weight };
  }
}

// --- Minimal zip support for .lottie archives (stored or deflated entries) ---
export async function readZip(bytes) {
  const view = new DataView(bytes instanceof ArrayBuffer ? bytes : bytes.buffer, bytes.byteOffset || 0, bytes.byteLength);
  const array = new Uint8Array(view.buffer, view.byteOffset, view.byteLength);
  let eocd = view.byteLength - 22;
  while (eocd >= 0 && view.getUint32(eocd, true) !== 0x06054b50) eocd--;
  if (eocd < 0) throw Error('Not a .lottie archive');
  const count = view.getUint16(eocd + 10, true);
  let offset = view.getUint32(eocd + 16, true);
  const entries = [];
  for (let i = 0; i < count; i++) {
    if (view.getUint32(offset, true) !== 0x02014b50) throw Error('Corrupt .lottie archive');
    const method = view.getUint16(offset + 10, true), compressed = view.getUint32(offset + 20, true);
    const nameLength = view.getUint16(offset + 28, true), extraLength = view.getUint16(offset + 30, true), commentLength = view.getUint16(offset + 32, true);
    const local = view.getUint32(offset + 42, true);
    const name = decoder.decode(array.subarray(offset + 46, offset + 46 + nameLength));
    const dataStart = local + 30 + view.getUint16(local + 26, true) + view.getUint16(local + 28, true);
    const raw = array.slice(dataStart, dataStart + compressed);
    let data;
    if (method === 0) data = raw;
    else if (method === 8) data = new Uint8Array(await new Response(new Blob([raw]).stream().pipeThrough(new DecompressionStream('deflate-raw'))).arrayBuffer());
    else throw Error(`Unsupported .lottie compression method ${method}`);
    entries.push({ name, data });
    offset += 46 + nameLength + extraLength + commentLength;
  }
  return entries;
}

const crcTable = new Uint32Array(256).map((_, n) => { let c = n; for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1; return c >>> 0; });
function crc32(data) { let crc = 0xffffffff; for (const byte of data) crc = crcTable[(crc ^ byte) & 0xff] ^ (crc >>> 8); return (crc ^ 0xffffffff) >>> 0; }

// Writes entries uncompressed; the player inflates nothing and files stay small.
export function writeZip(entries) {
  const locals = [], centrals = [];
  let offset = 0;
  for (const { name, data } of entries) {
    const nameBytes = encoder.encode(name), crc = crc32(data);
    const local = new Uint8Array(30 + nameBytes.length + data.length), lv = new DataView(local.buffer);
    lv.setUint32(0, 0x04034b50, true); lv.setUint16(4, 20, true); lv.setUint16(8, 0, true); lv.setUint32(14, crc, true); lv.setUint32(18, data.length, true); lv.setUint32(22, data.length, true); lv.setUint16(26, nameBytes.length, true);
    local.set(nameBytes, 30); local.set(data, 30 + nameBytes.length);
    const central = new Uint8Array(46 + nameBytes.length), cv = new DataView(central.buffer);
    cv.setUint32(0, 0x02014b50, true); cv.setUint16(4, 20, true); cv.setUint16(6, 20, true); cv.setUint16(10, 0, true); cv.setUint32(16, crc, true); cv.setUint32(20, data.length, true); cv.setUint32(24, data.length, true); cv.setUint16(28, nameBytes.length, true); cv.setUint32(42, offset, true);
    central.set(nameBytes, 46);
    locals.push(local); centrals.push(central); offset += local.length;
  }
  const centralSize = centrals.reduce((n, c) => n + c.length, 0);
  const end = new Uint8Array(22), ev = new DataView(end.buffer);
  ev.setUint32(0, 0x06054b50, true); ev.setUint16(8, entries.length, true); ev.setUint16(10, entries.length, true); ev.setUint32(12, centralSize, true); ev.setUint32(16, offset, true);
  const out = new Uint8Array(offset + centralSize + 22);
  let position = 0;
  for (const chunk of [...locals, ...centrals, end]) { out.set(chunk, position); position += chunk.length; }
  return out.buffer;
}
