import { createContext, useContext, useState, useCallback, useEffect, ReactNode } from 'react';
import { TransactionFilters, TransactionPage } from '../types';
import { transactionsApi } from '../api/transactions';

interface TransactionContextValue {
  page: TransactionPage | null;
  filters: TransactionFilters;
  loading: boolean;
  setFilters: (filters: TransactionFilters) => void;
  refresh: () => Promise<void>;
}

const TransactionContext = createContext<TransactionContextValue | null>(null);

const DEFAULT_FILTERS: TransactionFilters = { page: 0, pageSize: 50 };

export function TransactionProvider({ children }: { children: ReactNode }) {
  const [page, setPage] = useState<TransactionPage | null>(null);
  const [filters, setFiltersState] = useState<TransactionFilters>(DEFAULT_FILTERS);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const data = await transactionsApi.getAll(filters);
      setPage(data);
    } finally {
      setLoading(false);
    }
  }, [filters]);

  useEffect(() => { refresh(); }, [refresh]);

  const setFilters = useCallback((f: TransactionFilters) => {
    setFiltersState({ ...DEFAULT_FILTERS, ...f });
  }, []);

  return (
    <TransactionContext.Provider value={{ page, filters, loading, setFilters, refresh }}>
      {children}
    </TransactionContext.Provider>
  );
}

export function useTransactions() {
  const ctx = useContext(TransactionContext);
  if (!ctx) throw new Error('useTransactions must be used within TransactionProvider');
  return ctx;
}
