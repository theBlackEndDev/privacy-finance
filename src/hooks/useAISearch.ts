import { useState, useCallback } from 'react';
import { aiSearchApi } from '../api/ai_search';
import { AISearchResult, IntentMatch } from '../types';

// Mirrors the backend decline response (D9), minus the "Install AI Search" hint:
// a proposal only exists when the model is installed.
const DECLINE_RESULT: AISearchResult = {
  matched: false,
  intent: null,
  cardType: null,
  cardData: null,
  textAnswer: "I can't answer that yet. Try one of these:",
  source: 'none',
  interpretation: null,
  proposal: null,
  suggestions: [
    'How much did I spend on [category]?',
    'Show me transactions over $[amount]',
    'Am I over budget on [category]?',
    'Compare this month vs last month',
    'What are my subscriptions?',
    'How much money do I have?',
  ],
  queryTimeMs: 0,
};

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

  // Accept a router proposal: run it on the backend and show the answer.
  const confirm = useCallback(async (intentMatch: IntentMatch) => {
    setLoading(true);
    setError(null);
    try {
      const res = await aiSearchApi.confirmAISearch(intentMatch);
      setResult(res);
    } catch (err) {
      setResult(null);
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  // Reject a router proposal: show the decline text and example suggestions.
  const decline = useCallback(() => {
    setError(null);
    setResult({ ...DECLINE_RESULT, suggestions: [...DECLINE_RESULT.suggestions] });
  }, []);

  const clear = useCallback(() => {
    setResult(null);
    setError(null);
  }, []);

  return { result, loading, error, search, confirm, decline, clear };
}
