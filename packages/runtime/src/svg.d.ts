import type { IconDescriptor, Motion, Style } from './index.js';

/**
 * How the SVG starts playing. `auto` picks `toggle` for stateful icons, `loop` for
 * loaders and `hover` otherwise. `hover`, `click`, `toggle` and `js` need an inline
 * `<svg>`; an `<img>` or CSS background only plays `loop` and `load`.
 */
export type SvgTrigger = 'auto' | 'hover' | 'click' | 'toggle' | 'loop' | 'load' | 'js';

export interface SvgOptions {
  /** Default `light`. */
  motion?: Motion;
  /** Default `outline`. */
  style?: Style;
  /** Outline weight from 1 to 3 (default 2); any value works, as `stroke-width`. */
  weight?: number;
  /** `currentColor` (default) or a hex value. */
  color?: string;
  /** Default `auto`. */
  trigger?: SvgTrigger;
  /**
   * Prefix of the SVG's element ids (default: the icon id, with `-` written as `_`). Give
   * each inline copy on a page its own. With the `js` trigger, call `beginElement()` on
   * `<id>_m` to play; for a stateful icon, `<id>_a` leaves the initial state and `<id>_b`
   * returns to it.
   */
  id?: string;
  /** Width in CSS pixels; the height follows the icon. Default: the icon's own width. */
  size?: number;
  /** Stateful icons: the state shown first (default: the icon's `endState`, as the runtime). */
  state?: string;
}

/** The trigger `auto` resolves to for this icon. */
export declare function defaultTrigger(icon: IconDescriptor, motion?: Motion): Exclude<SvgTrigger, 'auto'>;

/**
 * The icon as an animated SVG (SMIL), playing as the runtime does: the action from frame
 * 0, then the rest frame. `animations` maps animation names to Lottie JSON. Static
 * attributes hold the rest pose, so renderers without SMIL show the icon at rest.
 */
export declare function iconToSvg(icon: IconDescriptor, animations: Record<string, object> | ((name: string) => object), options?: SvgOptions): string;

/** `iconToSvg` from the bytes of the icon's `.lottie`. */
export declare function dotLottieToSvg(icon: IconDescriptor, bytes: ArrayBuffer | Uint8Array, options?: SvgOptions): Promise<string>;

/** One Lottie animation as an animated SVG: `window` is the frame range played, `rest` the frame shown before and after. */
export declare function lottieToSvg(animation: object, options?: Omit<SvgOptions, 'motion' | 'state'> & { trigger?: 'hover' | 'click' | 'loop' | 'load' | 'js'; window?: [number, number]; rest?: number }): string;

export declare const SVG_TRIGGERS: SvgTrigger[];
