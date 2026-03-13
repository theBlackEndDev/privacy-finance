import { useAccounts } from '../context/AccountContext';
import { useApp } from '../context/AppContext';
import { ImportWizard } from '../components/import/ImportWizard';
import { Spinner } from '../components/ui/Spinner';

export function ImportPage() {
  const { accounts, loading } = useAccounts();
  const { navigate } = useApp();

  if (loading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', paddingTop: 64 }}>
        <Spinner />
      </div>
    );
  }

  if (accounts.length === 0) {
    return (
      <div style={{ color: 'var(--text-muted)', textAlign: 'center', paddingTop: 48 }}>
        <p style={{ fontSize: 16, fontWeight: 600 }}>No accounts yet</p>
        <p style={{ fontSize: 13, marginTop: 8 }}>Create an account first before importing transactions.</p>
      </div>
    );
  }

  return (
    <ImportWizard
      accounts={accounts}
      onNavigateToTransactions={() => navigate('transactions')}
    />
  );
}
