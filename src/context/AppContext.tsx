import { createContext, useContext, useState, useCallback, ReactNode } from 'react';
import { View } from '../types';

interface AppContextValue {
  currentView: View;
  navigate: (view: View) => void;
  activeAccountId: string | null;
  setActiveAccountId: (id: string | null) => void;
}

const AppContext = createContext<AppContextValue | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const [currentView, setCurrentView] = useState<View>('dashboard');
  const [activeAccountId, setActiveAccountId] = useState<string | null>(null);

  const navigate = useCallback((view: View) => setCurrentView(view), []);

  return (
    <AppContext.Provider value={{ currentView, navigate, activeAccountId, setActiveAccountId }}>
      {children}
    </AppContext.Provider>
  );
}

export function useApp() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error('useApp must be used within AppProvider');
  return ctx;
}
