import { IconController } from './controller.mjs';
import { rewriteStrokeWidth } from './weight.mjs';

export const schemaVersion = 1;
const styles = ['outline', 'solid', 'duotone'];
// Authored outline layers. Any other weight in WEIGHT_RANGE renders the Regular
// geometry through the style.outline.width stroke-width token.
const authoredWeights = [1.5, 2, 2.5];
const WEIGHT_RANGE = [1, 3];
const STROKE_WIDTH_TOKEN = 'style.outline.width';

export function validateIcon(icon) {
  if (icon?.schemaVersion !== schemaVersion || !/^[a-z][a-z0-9-]*$/.test(icon.id)) throw Error('Unsupported icon descriptor');
  if (icon.motions?.length !== 2 || !['light', 'strong'].every(id => icon.motions.some(m => m.motionId === id))) throw Error('Light and Strong motions are required');
  for (const motion of icon.motions) {
    for (const action of [motion, ...(motion.reverse ? [motion.reverse] : [])]) {
      if (!action.name || !Number.isFinite(action.fps) || action.fps <= 0 || !Number.isFinite(action.frames) || action.frames <= 0 || !Number.isFinite(action.rest) || action.rest < 0 || action.rest >= action.frames || !Number.isFinite(action.active) || action.active < 0 || action.active * action.fps > action.frames) throw Error('Invalid animation timing');
    }
    if (icon.stateful && (!motion.reverse || motion.startState !== icon.startState || motion.endState !== icon.endState || motion.reverse.endState !== icon.startState)) throw Error('Invalid state transition');
  }
  return icon;
}

function settings(options, previous = {}) {
  const result = { color: '#008c76', theme: 'light', style: 'outline', weight: 2, motion: 'light', speed: 1, ...previous, ...options };
  if (!/^#[0-9a-f]{6}$/i.test(result.color)) throw Error('color must be a six-digit hex value; resolve your theme token before passing it');
  if (!styles.includes(result.style)) throw Error('Unsupported style');
  if (!Number.isFinite(result.weight) || result.weight < WEIGHT_RANGE[0] || result.weight > WEIGHT_RANGE[1]) throw Error(`weight must be a number from ${WEIGHT_RANGE[0]} to ${WEIGHT_RANGE[1]}`);
  if (!['light', 'dark'].includes(result.theme) || !['light', 'strong'].includes(result.motion)) throw Error('Unsupported theme or motion');
  if (!Number.isFinite(result.speed) || result.speed <= 0 || result.speed > 4) throw Error('speed must be greater than 0 and at most 4');
  if (result.reducedMotion !== undefined && typeof result.reducedMotion !== 'boolean') throw Error('reducedMotion must be boolean');
  return result;
}

export function paintIcon(player, options) {
  const value = settings(options);
  player.setTheme(value.theme);
  player.setColorSlot('color.primary', [1, 3, 5].map(i => parseInt(value.color.slice(i, i + 2), 16) / 255).concat(1));
  const authored = authoredWeights.includes(value.weight);
  for (const [name, weight] of [['light', 1.5], ['regular', 2], ['bold', 2.5]]) player.setScalarSlot(`style.outline.${name}`, value.style !== 'solid' && (authored ? value.weight === weight : name === 'regular') ? 100 : 0);
  // Players without stroke-width token support show the Regular weight for in-between values.
  player.setScalarSlot(STROKE_WIDTH_TOKEN, authored ? 2 : value.weight);
  player.setScalarSlot('style.solid', value.style === 'solid' ? 100 : 0);
  player.setScalarSlot('style.tint', value.style === 'duotone' ? 18 : 0);
}

// Player injection also allows state/lifecycle tests without a browser or WASM.
// `asset` ({ original, reload }) lets in-between weights rewrite and reload the file.
export function connectIcon(icon, player, options = {}, media, asset) {
  validateIcon(icon);
  let config = settings(options), disposed = false;
  const reduced = () => config.reducedMotion ?? media?.matches ?? false;
  const controller = new IconController(icon, [player], { profile: config.motion, reduced: reduced(), paint: p => paintIcon(p, config) });
  const applyState = (state, animate) => {
    if (disposed) throw Error('Icon is destroyed');
    if (![icon.startState, icon.endState].includes(state)) throw Error(`Unknown state: ${state}`);
    if (state === controller.state) return Promise.resolve(true);
    const direction = state === icon.startState ? 'reverse' : 'forward';
    if (animate) return controller.play(direction, { force: true });
    controller.cancel();
    const motion = icon.motions.find(m => m.motionId === controller.profile);
    controller.direction = direction; controller.state = state;
    controller.load(direction === 'reverse' ? motion.reverse : motion); controller.rest();
    return Promise.resolve(true);
  };
  if (options.state !== undefined) {
    if (![icon.startState, icon.endState].includes(options.state)) { controller.destroy(); throw Error('Unknown initial state'); }
    applyState(options.state, false);
  }
  controller.setSpeed(config.speed);
  // Application state must survive a reduced-motion preference change. The
  // gallery's stopPreview() deliberately resets demos, so do not call it here.
  const syncReduced = () => { controller.reduced = reduced(); if (controller.reduced) { controller.cancel(); controller.rest(); } };
  media?.addEventListener('change', syncReduced);
  // Weights between the authored stops are written into the file; changing
  // them reloads the player. Rapid changes coalesce into the latest value.
  const bridged = weight => authoredWeights.includes(weight) ? null : weight;
  let current = bridged(config.weight), wanted = current, reloading = null;
  const syncAsset = () => {
    wanted = bridged(config.weight);
    if (!asset || wanted === current || reloading) return;
    reloading = (async () => {
      while (wanted !== current && !disposed) {
        const target = wanted;
        const bytes = target === null ? asset.original : await rewriteStrokeWidth(asset.original, target);
        if (disposed) return;
        controller.cancel();
        await asset.reload(bytes);
        if (disposed) return;
        current = target;
        controller.load(controller.meta); controller.rest();
      }
    })().catch(error => console.error(error)).finally(() => { reloading = null; });
  };
  return {
    get state() { return controller.state; },
    get configuration() { return { ...config }; },
    setState(state, { animate = true } = {}) { return applyState(state, animate); },
    play(direction = 'forward') {
      if (!['forward', 'reverse'].includes(direction)) throw Error('Unknown direction');
      if (direction === 'reverse' && !icon.motions.find(m => m.motionId === config.motion).reverse) return Promise.resolve(false);
      return controller.play(direction, { force: true });
    },
    preview() { return controller.preview({ force: true }); },
    stop() { controller.cancel(); controller.rest(); },
    update(next) {
      if (disposed) throw Error('Icon is destroyed');
      config = settings(next, config);
      if (controller.profile !== config.motion) controller.setMotion(config.motion);
      controller.setSpeed(config.speed); syncReduced(); controller.repaint(); syncAsset();
    },
    destroy() { if (disposed) return; disposed = true; media?.removeEventListener('change', syncReduced); controller.destroy(); }
  };
}

export async function createIcon(canvas, { icon, src, data, signal, wasmUrl, size = 32, ...options }) {
  validateIcon(icon); const initial = settings(options);
  if (!canvas?.getContext) throw Error('A canvas element is required');
  if (!Number.isFinite(size) || size <= 0 || size > 1024) throw Error('size must be greater than 0 and at most 1024 CSS pixels');
  if (!src && !data) throw Error('Provide an icon asset URL or ArrayBuffer');
  signal?.throwIfAborted();
  // Browser-only import keeps importing this module safe during SSR.
  const { DotLottie } = await import('@lottiefiles/dotlottie-web');
  signal?.throwIfAborted();
  if (wasmUrl) DotLottie.setWasmUrl(wasmUrl);
  // The asset is fetched here so an in-between weight can be written into it first.
  let original = data;
  if (!original) {
    const response = await fetch(src, { signal });
    if (!response.ok) throw Error(`Could not load icon asset (${response.status})`);
    original = await response.arrayBuffer();
  }
  signal?.throwIfAborted();
  const bytes = authoredWeights.includes(initial.weight) ? original : await rewriteStrokeWidth(original, initial.weight);
  const renderConfig = { autoResize: true, devicePixelRatio: Math.min(globalThis.devicePixelRatio || 1, 2) };
  // A canvas without CSS dimensions follows its backing-buffer attributes.
  // DPR resizing then enlarges layout and triggers another resize indefinitely.
  // The public size option owns the CSS box independently of that buffer.
  canvas.style.width = `${size}px`;
  canvas.style.height = `${size * icon.height / icon.width}px`;
  // Pass the asset into construction: load() is ignored until WASM is ready.
  // The constructor queues the asset and emits load asynchronously.
  const player = new DotLottie({ canvas, data: bytes, autoplay: false, loop: false, renderConfig });
  const loaded = () => new Promise((resolve, reject) => {
    let timer;
    const cleanup = () => { clearTimeout(timer); player.removeEventListener('load', done); player.removeEventListener('loadError', failed); signal?.removeEventListener('abort', aborted); };
    const done = () => { cleanup(); resolve(); };
    const failed = event => { cleanup(); reject(Error(event?.message || 'Could not load icon')); };
    const aborted = () => { cleanup(); reject(signal.reason || new DOMException('Aborted', 'AbortError')); };
    player.addEventListener('load', done); player.addEventListener('loadError', failed); signal?.addEventListener('abort', aborted, { once: true });
    timer = setTimeout(() => failed({ message: 'Icon loading timed out' }), 20000);
  });
  try {
    await loaded();
    const reload = next => { const promise = loaded(); player.load({ data: next, autoplay: false, loop: false, renderConfig }); return promise; };
    return connectIcon(icon, player, options, globalThis.matchMedia?.('(prefers-reduced-motion: reduce)'), { original, reload });
  } catch (error) { player.destroy(); throw error; }
}
