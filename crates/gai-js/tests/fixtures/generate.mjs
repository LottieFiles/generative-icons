// Generates the math/*.f64 files and to-fixed.txt into <dir>: node generate.mjs <dir> (Node 26.7.0,
// V8 14.6.202.34, arm64). The .f64 files are little-endian f64 rows: unary [x, r], binary [a, b, r], hypot
// [argc, args..., r]. The pow and powop results were then replaced by correctly rounded ones
// (libbf, 300 bits); Node's platform pow differs from them on one row.
import fs from 'node:fs';
const out = process.argv[2];
fs.mkdirSync(out, { recursive: true });
let s = 0x9e3779b97f4a7c15n;
const next = () => { s = (s + 0x9e3779b97f4a7c15n) & 0xffffffffffffffffn; let z = s; z = ((z ^ (z >> 30n)) * 0xbf58476d1ce4e5b9n) & 0xffffffffffffffffn; z = ((z ^ (z >> 27n)) * 0x94d049bb133111ebn) & 0xffffffffffffffffn; return z ^ (z >> 31n); };
const uniform = () => Number(next() >> 11n) / 2 ** 53;
const sign = () => (next() & 1n ? -1 : 1);
const logUniform = (lo, hi) => sign() * 10 ** (lo + (hi - lo) * uniform());
const bits = () => { const b = new BigUint64Array([next()]); return new Float64Array(b.buffer)[0]; };
const specials = [0, -0, Infinity, -Infinity, NaN, 1, -1, 0.5, -0.5, 2, -2, Math.PI, -Math.PI, Math.PI / 2, Math.PI / 4, 1e-300, -1e-300, 5e-324, -5e-324, 2.2250738585072014e-308, Number.MAX_VALUE, -Number.MAX_VALUE, 1e22, -1e22, 1e300, 710, -745, 0.9999999999999999, 1.0000000000000002, 3, 10, 100];
function write(name, rows) { fs.writeFileSync(`${out}/${name}.f64`, Buffer.from(new Float64Array(rows.flat()).buffer)); }
const unary = {
  sin: [() => logUniform(-8, 3), () => logUniform(3, 300)], cos: [() => logUniform(-8, 3), () => logUniform(3, 300)], tan: [() => logUniform(-8, 3), () => logUniform(3, 300)],
  asin: [() => sign() * uniform(), () => logUniform(-12, 0)], acos: [() => sign() * uniform(), () => sign() * (1 - logUniform(-16, -1) * sign())],
  atan: [() => logUniform(-8, 8), () => logUniform(8, 300)], exp: [() => sign() * 745 * uniform(), () => logUniform(-12, 1)],
  log: [() => Math.abs(logUniform(-300, 300)), () => 1 + logUniform(-12, -1)], log1p: [() => Math.abs(logUniform(-12, 300)) - (uniform() < .3 ? 0.9 * uniform() : 0), () => logUniform(-16, -2)],
  log2: [() => Math.abs(logUniform(-300, 300)), () => 2 ** Math.round(sign() * 60 * uniform()) * (uniform() < .5 ? 1 : 1 + logUniform(-15, -8))],
  log10: [() => Math.abs(logUniform(-300, 300)), () => 10 ** Math.round(sign() * 20 * uniform())],
  cosh: [() => sign() * 712 * uniform(), () => logUniform(-8, 1)], sinh: [() => sign() * 712 * uniform(), () => logUniform(-8, 1)], tanh: [() => logUniform(-8, 1.5), () => sign() * 40 * uniform()],
  acosh: [() => 1 + Math.abs(logUniform(-12, 300)), () => 1 + logUniform(-16, -2)], asinh: [() => logUniform(-8, 300), () => logUniform(-16, -8)],
  atanh: [() => sign() * uniform(), () => sign() * (1 - Math.abs(logUniform(-16, -1)))], expm1: [() => sign() * 710 * uniform(), () => logUniform(-16, -1)],
  cbrt: [() => logUniform(-300, 300), bits], sqrt: [() => Math.abs(logUniform(-300, 300)), bits],
};
for (const [name, gens] of Object.entries(unary)) {
  const inputs = [...specials];
  for (const gen of gens) for (let i = 0; i < 150; i++) inputs.push(gen());
  write(name, inputs.map(x => [x, Math[name](x)]));
}
const pairs = [];
for (const a of [0, -0, Infinity, -Infinity, NaN, 1, -1, 2.5, -2.5]) for (const b of [0, -0, Infinity, -Infinity, NaN, 1, -1, 2.5, -2.5]) pairs.push([a, b]);
const atan2 = [...pairs]; for (let i = 0; i < 300; i++) atan2.push([logUniform(-10, 10), logUniform(-10, 10)]);
write('atan2', atan2.map(([a, b]) => [a, b, Math.atan2(a, b)]));
const pow = [...pairs];
for (const y of [2, 0.5, 3, -1, -2, 0.25, 1 / 3, 1e10, -1e10]) for (const x of [-8, -2, -0.5, 0.5, 2, 8, 1.0000001, 0.9999999]) pow.push([x, y]);
for (let i = 0; i < 300; i++) pow.push([Math.abs(logUniform(-5, 5)), sign() * 20 * uniform()]);
for (let i = 0; i < 100; i++) pow.push([sign() * Math.abs(logUniform(-3, 3)), Math.round(sign() * 30 * uniform())]);
for (let i = 0; i < 100; i++) pow.push([Math.abs(logUniform(-300, 300)), logUniform(-3, 0)]);
write('pow', pow.map(([a, b]) => [a, b, Math.pow(a, b)]));
write('powop', pow.map(([a, b]) => [a, b, a ** b]));
const hypot = [[0, Math.hypot()]];
const arg = () => { const r = uniform(); return r < .05 ? 0 : r < .08 ? -0 : r < .1 ? NaN : r < .12 ? (uniform() < .5 ? Infinity : -Infinity) : r < .5 ? logUniform(-3, 3) : logUniform(-300, 300); };
for (let argc = 1; argc <= 6; argc++) for (let i = 0; i < 120; i++) { const args = Array.from({ length: argc }, arg); hypot.push([argc, ...args, Math.hypot(...args)]); }
for (let i = 0; i < 60; i++) { const args = Array.from({ length: 3 + (i % 4) }, () => sign() * (1 + 1e-8 * uniform())); hypot.push([args.length, ...args, Math.hypot(...args)]); }
write('hypot', hypot);
// toFixed(3): values like the corpus plus exact dyadic ties.
const fixed = [];
for (let i = 0; i < 1000; i++) fixed.push(sign() * 32 * uniform());
for (let k = -4000; k <= 4000; k += 47) fixed.push(k / 16, k / 32 + 1 / 64, k / 2048);
fs.writeFileSync(`${out}/to-fixed.txt`, fixed.map(x => `${Buffer.from(new Float64Array([x]).buffer).readBigUInt64LE().toString(16).padStart(16, '0')} ${x.toFixed(3)}`).join('\n') + '\n');
console.log(JSON.stringify({ node: process.version, v8: process.versions.v8, arch: process.arch }));
