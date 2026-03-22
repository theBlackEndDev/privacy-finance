import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { getDemoInvokeFallback } from './demoData';

// Returns true only when running inside a real Tauri webview.
export const isTauri = () =>
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

// Safe invoke: in plain browser preview, return realistic demo data for supported commands.
export function invoke<T>(cmd: string, args?: Record<string, unknown>, fallback?: T): Promise<T> {
  if (!isTauri()) {
    const demo = getDemoInvokeFallback(cmd, args);
    if (demo !== undefined) {
      return Promise.resolve(demo as T);
    }
    return Promise.resolve(fallback as T);
  }
  return tauriInvoke<T>(cmd, args);
}
