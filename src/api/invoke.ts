import { invoke as tauriInvoke } from '@tauri-apps/api/core';

// Returns true only when running inside a real Tauri webview.
export const isTauri = () =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

// Lazy-load demo data only in dev builds so Vite tree-shakes it in production.
const getDemoFallback: Promise<typeof import('./demoData').getDemoInvokeFallback> | null =
  import.meta.env.DEV
    ? import('./demoData').then(m => m.getDemoInvokeFallback)
    : null;

// Safe invoke: in plain browser preview, return realistic demo data for supported commands.
export function invoke<T>(cmd: string, args?: Record<string, unknown>, fallback?: T): Promise<T> {
  if (!isTauri() && getDemoFallback) {
    return getDemoFallback.then(fn => {
      const demo = fn(cmd, args);
      if (demo !== undefined) return demo as T;
      return fallback as T;
    });
  }
  if (!isTauri()) {
    return Promise.resolve(fallback as T);
  }
  return tauriInvoke<T>(cmd, args);
}
