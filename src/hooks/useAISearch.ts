import { useState, useCallback } from 'react';
import { aiSearchApi } from '../api/ai_search';
import { AISearchResult } from '../types';

export function useAISearch() {
  const [result, setResult] = useState<AISearchResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const search = useCallback(async (query: string) => {
    setLoading(true);
    setError(null);
    try {
      const res = await aiSearchApi.search(query);
      setResult(res);
    } catch (err) {
      setResult(null);
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  const clear = useCallback(() => {
    setResult(null);
    setError(null);
  }, []);

  return { result, loading, error, search, clear };
}
