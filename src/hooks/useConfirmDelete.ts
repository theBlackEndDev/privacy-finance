import { useState, useCallback } from 'react';

interface UseConfirmDeleteOptions<T> {
  onConfirm: (item: T) => Promise<void>;
}

export function useConfirmDelete<T>({ onConfirm }: UseConfirmDeleteOptions<T>) {
  const [pending, setPending] = useState<T | null>(null);

  const request = useCallback((item: T) => { setPending(item); }, []);
  const cancel = useCallback(() => { setPending(null); }, []);

  const confirm = useCallback(async () => {
    if (!pending) return;
    try {
      await onConfirm(pending);
    } finally {
      setPending(null);
    }
  }, [pending, onConfirm]);

  return { pending, request, cancel, confirm };
}
