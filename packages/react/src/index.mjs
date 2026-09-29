'use client';
import { createElement, useEffect, useRef } from 'react';
import { createIcon } from '@lottiefiles/generative-icons-runtime';

export function AnimatedIcon({ icon, src, state, size = 32, color = '#008c76', theme = 'light', variant = 'outline', weight = 2, motion = 'light', speed = 1, reducedMotion, wasmUrl, label, onReady, onError, ...canvasProps }) {
  const canvas = useRef(null), handle = useRef(null);
  const latest = useRef(null);
  latest.current = { state, color, theme, style: variant, weight, motion, speed, reducedMotion, onReady, onError };
  useEffect(() => {
    const abort = new AbortController(); let instance;
    createIcon(canvas.current, { icon, src, wasmUrl, size, ...latest.current, signal: abort.signal }).then(result => {
      if (abort.signal.aborted) { result.destroy(); return; }
      instance = result; handle.current = result;
      const { state: currentState, onReady: ready, onError: _error, ...paint } = latest.current;
      result.update(paint);
      if (currentState !== undefined) result.setState(currentState, { animate: false });
      ready?.(result);
    }).catch(error => { if (!abort.signal.aborted) latest.current.onError?.(error); });
    return () => { abort.abort(); instance?.destroy(); handle.current = null; };
  }, [icon, src, wasmUrl, size]);
  useEffect(() => {
    if (!handle.current) return;
    try { handle.current.update({ color, theme, style: variant, weight, motion, speed, reducedMotion }); if (state !== undefined) handle.current.setState(state); }
    catch (error) { latest.current.onError?.(error); }
  }, [state, color, theme, variant, weight, motion, speed, reducedMotion]);
  return createElement('canvas', { ...canvasProps, ref: canvas, width: size, height: size, style: { width: size, height: size, ...canvasProps.style }, role: label ? 'img' : undefined, 'aria-label': label, 'aria-hidden': label ? undefined : true });
}
