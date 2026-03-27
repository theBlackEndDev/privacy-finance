import { useState, useCallback } from 'react';
import { aiSearchApi } from '../api/ai_search';
import { AISearchResult } from '../types';

export function useAISearch() {
  const [result, setResult] = useState<AISearchResult | null>(null);
  const [loading, setLoading] = useState(false);

  const search = useCallback(async (query: string) => {
    setLoading(true);
    try {
      const res = await aiSearchApi.search(query);
      setResult(res);
    } catch {
      setResult(null);
    } finally {
      setLoading(false);
    }
  }, []);

  const clear = useCallback(() => {
    setResult(null);
  }, []);

  return { result, loading, search, clear };
}
