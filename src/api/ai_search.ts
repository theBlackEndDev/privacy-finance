import { invoke } from './invoke';
import { AISearchResult } from '../types';

export const aiSearchApi = {
  search: (query: string) =>
    invoke<AISearchResult>('ai_search', { query }, {
      matched: false,
      intent: null,
      cardType: null,
      cardData: null,
      textAnswer: null,
      suggestions: [],
      queryTimeMs: 0,
    }),
};
