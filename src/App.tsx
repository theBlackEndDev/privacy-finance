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
  }, [accountsLoading]); // eslint-disable-line react-hooks/exhaustive-deps

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
      case 'accounts':        return <AccountsPage />;
      case 'account-detail':  return <AccountDetailPage />;
      case 'categories':      return <CategoriesPage />;
      case 'dashboard':    return <DashboardPage />;
      case 'transactions': return <TransactionsPage />;
      case 'import':       return <ImportPage />;
      case 'recurring':    return <RecurringPage />;
      case 'budgets':      return <BudgetsPage />;
      case 'rules':        return <RulesPage />;
      case 'reports':      return <ReportsPage />;
      case 'reconciliation': return <ReconciliationPage />;

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

  useEffect(() => {
    encryptionApi.checkStatus().then(status => {
      setLocked(status.isEncrypted);
    }).catch(() => setLocked(false));
  }, []);

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
