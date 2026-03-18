import { createContext, useContext, useState, useCallback, useEffect, ReactNode } from 'react';
import { View } from '../types';
import { settingsApi } from '../api/settings';

type Theme = 'dark' | 'light';

interface AppContextValue {
  currentView: View;
  navigate: (view: View) => void;
  activeAccountId: string | null;
  setActiveAccountId: (id: string | null) => void;
  pendingEditTransactionId: string | null;
  setPendingEditTransactionId: (id: string | null) => void;
  theme: Theme;
  setTheme: (theme: Theme) => void;
}

const AppContext = createContext<AppContextValue | null>(null);

function applyTheme(theme: Theme) {
  document.documentElement.setAttribute('data-theme', theme);
}

export function AppProvider({ children }: { children: ReactNode }) {
  const [currentView, setCurrentView] = useState<View>('dashboard');
  const [activeAccountId, setActiveAccountId] = useState<string | null>(null);
  const [pendingEditTransactionId, setPendingEditTransactionId] = useState<string | null>(null);
  const [theme, setThemeState] = useState<Theme>('dark');

  useEffect(() => {
    settingsApi.get('theme').then(v => {
      const t = v === 'light' ? 'light' : 'dark';
      setThemeState(t);
      applyTheme(t);
    });
  }, []);

  const setTheme = useCallback((t: Theme) => {
    setThemeState(t);
    applyTheme(t);
    settingsApi.set('theme', t);
  }, []);

  const navigate = useCallback((view: View) => setCurrentView(view), []);

  return (
    <AppContext.Provider value={{ currentView, navigate, activeAccountId, setActiveAccountId, pendingEditTransactionId, setPendingEditTransactionId, theme, setTheme }}>
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error('useApp must be used within AppProvider');
  return ctx;
}
