export type Motion = 'light' | 'strong';
export type Style = 'outline' | 'solid' | 'duotone';
export interface Action { name: string; motionId: Motion; fps: number; frames: number; active: number; rest: number; startState: string; endState: string; continuous?: boolean; reverse?: Action }
export interface IconDescriptor { schemaVersion: 1; id: string; name: string; label: string; intent: string; stateful: boolean; previewReset: boolean; startState: string; endState: string; motions: Action[]; asset: { path: string; sha256: string; bytes: number }; category: string; family: string; styles: Style[]; weights: number[]; width: number; height: number; license: string }
export interface PaintOptions { color?: string; theme?: 'light' | 'dark'; style?: Style; /** Outline stroke weight from 1 to 3. 1.5, 2 and 2.5 are the authored Light, Regular and Bold layers; other values use the Regular geometry with the stroke-width token. */ weight?: number; motion?: Motion; speed?: number; reducedMotion?: boolean }
export interface IconHandle { readonly state: string; readonly configuration: PaintOptions; setState(state: string, options?: { animate?: boolean }): Promise<boolean>; play(direction?: 'forward' | 'reverse'): Promise<boolean>; preview(): Promise<boolean>; stop(): void; update(options: PaintOptions): void; destroy(): void }
export declare const schemaVersion: 1;
export declare function validateIcon(icon: IconDescriptor): IconDescriptor;
export declare function createIcon(canvas: HTMLCanvasElement, options: PaintOptions & { icon: IconDescriptor; src?: string; data?: ArrayBuffer; state?: string; size?: number; signal?: AbortSignal; wasmUrl?: string }): Promise<IconHandle>;
