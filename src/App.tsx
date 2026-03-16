import { AppProvider, useApp } from './context/AppContext';
import { AccountProvider } from './context/AccountContext';
import { CategoryProvider } from './context/CategoryContext';
import { MainLayout } from './components/layout/MainLayout';
import { AccountsPage } from './views/AccountsPage';
import { CategoriesPage } from './views/CategoriesPage';
import { TransactionsPage } from './views/TransactionsPage';
import { DashboardPage } from './views/DashboardPage';
import { ImportPage } from './views/ImportPage';
import { RecurringPage } from './views/RecurringPage';
import { ToastProvider } from './components/ui/Toast';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';
import { RecurringDueModal } from './components/recurring/RecurringDueModal';
import { recurringApi } from './api/recurring';
import { DueRecurringItem } from './types';
import { useState, useEffect } from 'react';
import { useToast } from './components/ui/Toast';

function AppShell() {
  const { currentView, navigate } = useApp();
  useKeyboardShortcuts({ navigate });
  const toast = useToast();

  const [dueItems, setDueItems] = useState<DueRecurringItem[]>([]);
  const [showDueModal, setShowDueModal] = useState(false);
  const [generating, setGenerating] = useState(false);

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
      case 'accounts':     return <AccountsPage />;
      case 'categories':   return <CategoriesPage />;
      case 'dashboard':    return <DashboardPage />;
      case 'transactions': return <TransactionsPage />;
      case 'import':       return <ImportPage />;
      case 'recurring':    return <RecurringPage />;
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
    </MainLayout>
  );
}

export default function App() {
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
