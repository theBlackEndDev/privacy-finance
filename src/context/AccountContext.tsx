import { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';
import { Account } from '../types';
import { accountsApi } from '../api/accounts';

interface AccountContextValue {
  accounts: Account[];
  loading: boolean;
  refresh: () => Promise<void>;
}

const AccountContext = createContext<AccountContextValue | null>(null);

export function AccountProvider({ children }: { children: ReactNode }) {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const data = await accountsApi.getAll();
      setAccounts(data);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  return (
    <AccountContext.Provider value={{ accounts, loading, refresh }}>
      {children}
    </AccountContext.Provider>
  );
}

export function useAccounts() {
  const ctx = useContext(AccountContext);
  if (!ctx) throw new Error('useAccounts must be used within AccountProvider');
  return ctx;
}
