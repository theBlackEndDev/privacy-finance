import { invoke } from './invoke';
import { AISearchResult, IntentMatch } from '../types';

const EMPTY_RESULT: AISearchResult = {
  matched: false,
  intent: null,
  cardType: null,
  cardData: null,
  textAnswer: null,
  source: 'none',
  interpretation: null,
  proposal: null,
  suggestions: [],
  queryTimeMs: 0,
};

export const aiSearchApi = {
  search: (query: string) =>
    invoke<AISearchResult>('ai_search', { query }, EMPTY_RESULT),
  confirmAISearch: (intentMatch: IntentMatch) =>
    invoke<AISearchResult>('ai_search_confirm', { intentMatch }, EMPTY_RESULT),
};
