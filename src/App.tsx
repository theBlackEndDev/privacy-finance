import { AppProvider, useApp } from './context/AppContext';
import { AccountProvider } from './context/AccountContext';
import { CategoryProvider } from './context/CategoryContext';
import { MainLayout } from './components/layout/MainLayout';
import { AccountsPage } from './views/AccountsPage';
import { CategoriesPage } from './views/CategoriesPage';
import { TransactionsPage } from './views/TransactionsPage';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';

// Placeholder views for phases not yet built
function PlaceholderView({ name }: { name: string }) {
  return (
    <div style={{ color: 'var(--text-secondary)', paddingTop: 48, textAlign: 'center' }}>
      <p style={{ fontSize: 18, fontWeight: 600 }}>{name}</p>
      <p style={{ fontSize: 14, marginTop: 8, color: 'var(--text-muted)' }}>Coming in a future phase</p>
    </div>
  );
}

function AppShell() {
  const { currentView, navigate } = useApp();
  useKeyboardShortcuts({ navigate });

  const view = (() => {
    switch (currentView) {
      case 'accounts':     return <AccountsPage />;
      case 'categories':   return <CategoriesPage />;
      case 'dashboard':    return <PlaceholderView name="Dashboard" />;
      case 'transactions': return <TransactionsPage />;
      case 'import':       return <PlaceholderView name="Import" />;
    }
  })();

  return <MainLayout>{view}</MainLayout>;
}

export default function App() {
  return (
    <AppProvider>
      <AccountProvider>
        <CategoryProvider>
          <AppShell />
        </CategoryProvider>
      </AccountProvider>
    </AppProvider>
  );
}
