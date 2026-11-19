import { AppProvider, useApp } from './context/AppContext';
import { AccountProvider } from './context/AccountContext';
import { CategoryProvider } from './context/CategoryContext';
import { MainLayout } from './components/layout/MainLayout';
import { AccountsPage } from './views/AccountsPage';
import { AccountDetailPage } from './views/AccountDetailPage';
import { CategoriesPage } from './views/CategoriesPage';
import { TransactionsPage } from './views/TransactionsPage';
import { DashboardPage } from './views/DashboardPage';
import { ImportPage } from './views/ImportPage';
import { RecurringPage } from './views/RecurringPage';
import { BudgetsPage } from './views/BudgetsPage';
import { RulesPage } from './views/RulesPage';
import { ReportsPage } from './views/ReportsPage';
import { ReconciliationPage } from './views/ReconciliationPage';

import { ToastProvider } from './components/ui/Toast';
import { ErrorBoundary } from './components/ui/ErrorBoundary';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';
import { RecurringDueModal } from './components/recurring/RecurringDueModal';
import { KeyboardShortcutHelp } from './components/ui/KeyboardShortcutHelp';
import { OnboardingWizard } from './components/onboarding/OnboardingWizard';
import { UnlockScreen } from './components/encryption/UnlockScreen';
import { useAccounts } from './context/AccountContext';
import { useCategories } from './context/CategoryContext';
import { encryptionApi } from './api/encryption';
import { settingsApi } from './api/settings';
import { recurringApi } from './api/recurring';
import { DueRecurringItem } from './types';
import { useState, useEffect, useCallback } from 'react';
import { useToast } from './components/ui/Toast';
import { setCurrencySymbol } from './utils/formatCurrency';
import { setDateFormat, DateFormat } from './utils/formatDate';

const DATE_FORMATS: DateFormat[] = ['MMM D', 'MMM D, YYYY', 'MM/DD/YYYY', 'DD/MM/YYYY', 'YYYY-MM-DD'];

function AppShell() {
  const { currentView, navigate } = useApp();
  const { accounts, loading: accountsLoading, refresh: refreshAccounts } = useAccounts();
  const { categories } = useCategories();
  const toast = useToast();

  const [dueItems, setDueItems] = useState<DueRecurringItem[]>([]);
  const [showDueModal, setShowDueModal] = useState(false);
  const [generating, setGenerating] = useState(false);
  const [showShortcutHelp, setShowShortcutHelp] = useState(false);
  const [onboarding, setOnboarding] = useState<'loading' | 'visible' | 'hidden'>('loading');

  useEffect(() => {
    if (accountsLoading) return;
    if (accounts.length > 0) {
      setOnboarding('hidden');
      return;
    }
    settingsApi.get('onboarding_complete').then(val => {
      setOnboarding(val !== 'true' ? 'visible' : 'hidden');
    }).catch(() => setOnboarding('hidden'));
  }, [accountsLoading, accounts.length]);

  async function handleOnboardingComplete() {
    setOnboarding('hidden');
    await refreshAccounts();
  }

  const toggleShortcutHelp = useCallback(() => setShowShortcutHelp(prev => !prev), []);
  useKeyboardShortcuts({ navigate, onToggleShortcutHelp: toggleShortcutHelp });

  useEffect(() => {
    recurringApi.checkDue().then((items) => {
      if (items.length > 0) {
        setDueItems(items);
        setShowDueModal(true);
      }
    });
  }, []);

  async function handleGenerateAll() {
    setGenerating(true);
    try {
      await recurringApi.generateAll();
      await refreshAccounts();
      setShowDueModal(false);
      setDueItems([]);
    } catch (err) {
      toast(`Failed to generate transactions: ${err}`, 'error');
    } finally {
      setGenerating(false);
    }
  }

  const view = (() => {
    switch (currentView) {
      case 'accounts':        return <ErrorBoundary><AccountsPage /></ErrorBoundary>;
      case 'account-detail':  return <ErrorBoundary><AccountDetailPage /></ErrorBoundary>;
      case 'categories':      return <ErrorBoundary><CategoriesPage /></ErrorBoundary>;
      case 'dashboard':    return <ErrorBoundary><DashboardPage /></ErrorBoundary>;
      case 'transactions': return <ErrorBoundary><TransactionsPage /></ErrorBoundary>;
      case 'import':       return <ErrorBoundary><ImportPage /></ErrorBoundary>;
      case 'recurring':    return <ErrorBoundary><RecurringPage /></ErrorBoundary>;
      case 'budgets':      return <ErrorBoundary><BudgetsPage /></ErrorBoundary>;
      case 'rules':        return <ErrorBoundary><RulesPage /></ErrorBoundary>;
      case 'reports':      return <ErrorBoundary><ReportsPage /></ErrorBoundary>;
      case 'reconciliation': return <ErrorBoundary><ReconciliationPage /></ErrorBoundary>;
      default: return <ErrorBoundary><DashboardPage /></ErrorBoundary>;
    }
  })();

  return (
    <MainLayout>
      {view}
      {showDueModal && (
        <RecurringDueModal
          dueItems={dueItems}
          onGenerateAll={handleGenerateAll}
          onDismiss={() => setShowDueModal(false)}
          loading={generating}
        />
      )}
      {showShortcutHelp && (
        <KeyboardShortcutHelp onClose={() => setShowShortcutHelp(false)} />
      )}
      {onboarding === 'visible' && (
        <OnboardingWizard
          categories={categories}
          onComplete={handleOnboardingComplete}
        />
      )}
    </MainLayout>
  );
}

export default function App() {
  const [locked, setLocked] = useState<boolean | null>(null);
  const [settingsLoaded, setSettingsLoaded] = useState(false);

  useEffect(() => {
    encryptionApi.checkStatus().then(status => {
      setLocked(status.isEncrypted);
    }).catch(() => setLocked(false));
  }, []);

  useEffect(() => {
    if (locked !== false) return;
    let cancelled = false;
    Promise.all([
      settingsApi.get('currency_symbol').catch(() => null),
      settingsApi.get('date_format').catch(() => null),
    ]).then(([symbol, format]) => {
      if (cancelled) return;
      if (symbol) setCurrencySymbol(symbol);
      if (format && (DATE_FORMATS as string[]).includes(format)) setDateFormat(format as DateFormat);
      setSettingsLoaded(true);
    });
    return () => { cancelled = true; };
  }, [locked]);

  if (locked === null) {
    return null; // Loading encryption status
  }

  if (locked) {
    return (
      <ToastProvider>
        <UnlockScreen onUnlocked={() => setLocked(false)} />
      </ToastProvider>
    );
  }

  if (!settingsLoaded) {
    return null; // Loading settings that affect formatting
  }

  return (
    <ToastProvider>
      <AppProvider>
        <AccountProvider>
          <CategoryProvider>
            <AppShell />
          </CategoryProvider>
        </AccountProvider>
      </AppProvider>
    </ToastProvider>
  );
}
