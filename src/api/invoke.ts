import { invoke as tauriInvoke } from '@tauri-apps/api/core';

// Returns true only when running inside a real Tauri webview.
export const isTauri = () =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

// Safe invoke: falls back to `fallback` when running in plain browser (e.g. `vite dev`).
export function invoke<T>(cmd: string, args?: Record<string, unknown>, fallback?: T): Promise<T> {
  if (!isTauri()) {
    return Promise.resolve(fallback as T);
  }
  return tauriInvoke<T>(cmd, args);
}
