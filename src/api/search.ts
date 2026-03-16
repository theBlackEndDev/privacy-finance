import { invoke } from './invoke';
import { SearchResult, SearchSuggestion } from '../types';

export const searchApi = {
  search: (query: string, limit?: number, offset?: number) =>
    invoke<string[]>('search_transactions', { query, limit, offset }, []),
  searchFull: (query: string, limit?: number, offset?: number) =>
    invoke<SearchResult[]>('search_transactions_full', { query, limit, offset }, []),
  suggest: (query: string) =>
    invoke<SearchSuggestion[]>('search_suggest', { query }, []),
  rebuildFts: () => invoke<void>('rebuild_fts', {}),
};
