import { invoke } from './invoke';
import { SearchResult } from '../types';

export const searchApi = {
  searchFull: (query: string, limit?: number, offset?: number) =>
    invoke<SearchResult[]>('search_transactions_full', { query, limit, offset }, []),
};
