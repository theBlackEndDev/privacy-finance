import { AppProvider, useApp } from './context/AppContext';
import { AccountProvider } from './context/AccountContext';
import { CategoryProvider } from './context/CategoryContext';
import { MainLayout } from './components/layout/MainLayout';
import { AccountsPage } from './views/AccountsPage';
import { CategoriesPage } from './views/CategoriesPage';
import { TransactionsPage } from './views/TransactionsPage';
import { DashboardPage } from './views/DashboardPage';
import { ImportPage } from './views/ImportPage';
import { ToastProvider } from './components/ui/Toast';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';

function AppShell() {
  const { currentView, navigate } = useApp();
  useKeyboardShortcuts({ navigate });

  const view = (() => {
    switch (currentView) {
      case 'accounts':     return <AccountsPage />;
      case 'categories':   return <CategoriesPage />;
      case 'dashboard':    return <DashboardPage />;
      case 'transactions': return <TransactionsPage />;
      case 'import':       return <ImportPage />;
    }
  })();

  return <MainLayout>{view}</MainLayout>;
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
