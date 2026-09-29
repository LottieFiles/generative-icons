import type { CanvasHTMLAttributes, ReactElement } from 'react';
import type { IconDescriptor, IconHandle, PaintOptions, Style } from '@lottiefiles/generative-icons-runtime';
export interface AnimatedIconProps extends Omit<CanvasHTMLAttributes<HTMLCanvasElement>, 'color' | 'onError'>, Omit<PaintOptions, 'style'> { icon: IconDescriptor; src: string; state?: string; size?: number; variant?: Style; label?: string; wasmUrl?: string; onReady?: (handle: IconHandle) => void; onError?: (error: Error) => void }
export declare function AnimatedIcon(props: AnimatedIconProps): ReactElement;
