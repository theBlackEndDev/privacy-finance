/** Video configuration */
export const VIDEO = {
  width: 1920,
  height: 1080,
  fps: 30,
  durationInSeconds: 90,
  get durationInFrames() {
    return this.fps * this.durationInSeconds;
  },
} as const;

/** Convert seconds to frames */
export function sec(s: number): number {
  return Math.round(s * VIDEO.fps);
}

/** Act timing (in seconds) */
export const ACTS = {
  act1: { start: 0, end: 15 },
  act2: { start: 15, end: 75 },
  act3: { start: 75, end: 90 },
} as const;

/** Feature scenes within Act 2 (in seconds, relative to video start) */
export const FEATURES = [
  { id: 'onboarding',    start: 15, end: 22, overlay: 'Set up in 30 seconds' },
  { id: 'dashboard',     start: 22, end: 30, overlay: 'Everything at a glance' },
  { id: 'transactions',  start: 30, end: 38, overlay: 'Track every dollar' },
  { id: 'csv-import',    start: 38, end: 45, overlay: 'Import from any bank' },
  { id: 'ai-search',     start: 45, end: 52, overlay: 'Ask questions in plain English' },
  { id: 'budgets',       start: 52, end: 58, overlay: 'Set budgets. Stay on track.' },
  { id: 'reports',       start: 58, end: 64, overlay: 'See where your money goes' },
  { id: 'recurring',     start: 64, end: 70, overlay: 'Never miss a bill' },
  { id: 'rules',         start: 70, end: 75, overlay: 'Automate the boring stuff' },
] as const;

/** Privacy bullets for Act 3 */
export const PRIVACY_BULLETS = [
  'No account required',
  'No internet required',
  'AI search runs on-device',
  'Your data never leaves your machine',
] as const;

/** Colors pulled from app CSS variables */
export const COLORS = {
  bgPrimary: '#0f0f0f',
  bgSecondary: '#1a1a1a',
  textPrimary: '#e8e8e8',
  textSecondary: '#999999',
  textMuted: '#666666',
  accent: '#6366f1',
  income: '#22c55e',
  expense: '#ef4444',
  border: '#2a2a2a',
} as const;

/** Typography */
export const FONTS = {
  heading: 'Inter, system-ui, -apple-system, sans-serif',
  body: 'Inter, system-ui, -apple-system, sans-serif',
  mono: 'JetBrains Mono, Menlo, monospace',
} as const;

/** Crossfade duration in frames */
export const CROSSFADE_FRAMES = 8;
