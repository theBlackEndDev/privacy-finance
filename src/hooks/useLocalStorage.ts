import { useState } from 'react';

export function useLocalStorage<T>(key: string, defaultValue: T): [T, (value: T) => void] {
  const [state, setState] = useState<T>(() => {
    try {
      const stored = localStorage.getItem(key);
      return stored !== null ? (JSON.parse(stored) as T) : defaultValue;
    } catch {
      return defaultValue;
    }
  });

  function setValue(value: T) {
    setState(value);
    try { localStorage.setItem(key, JSON.stringify(value)); } catch { /* noop */ }
  }

  return [state, setValue];
}
