import { useState, useRef, useEffect } from 'react';
import { Transaction } from '../types';
import { useAccounts } from '../context/AccountContext';
import { useCategories } from '../context/CategoryContext';
import { useTransactions, TransactionProvider } from '../context/TransactionContext';
import { useApp } from '../context/AppContext';
import { TransactionForm } from '../components/transactions/TransactionForm';
import { TransferForm } from '../components/transactions/TransferForm';
import { TransactionFiltersBar } from '../components/transactions/TransactionFilters';
import { TransactionList } from '../components/transactions/TransactionList';
import { TransactionEditModal } from '../components/transactions/TransactionEditModal';
import { Button } from '../components/ui/Button';
import { Spinner } from '../components/ui/Spinner';

function TransactionsInner() {
  const { accounts } = useAccounts();
  const { categories } = useCategories();
  const { page, filters, loading, setFilters, refresh } = useTransactions();
  const { activeAccountId } = useApp();

  // Sync sidebar account selection into filters
  useEffect(() => {
    setFilters({ page: 0, pageSize: 50, accountId: activeAccountId ?? undefined });
  }, [activeAccountId]); // eslint-disable-line react-hooks/exhaustive-deps
  const [editing, setEditing] = useState<Transaction | null>(null);
  const [showTransfer, setShowTransfer] = useState(false);
  const quickEntryRef = useRef<HTMLInputElement | null>(null);

  function handlePageChange(p: number) {
    setFilters({ ...filters, page: p });
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      {/* Quick-entry bar */}
      <TransactionForm
        accounts={accounts}
        categories={categories}
        defaultAccountId={activeAccountId}
        onSaved={refresh}
        focusRef={quickEntryRef}
      />

      {/* Filter bar + transfer toggle */}
      <div style={{ display: 'flex', alignItems: 'stretch', gap: 0 }}>
        <div style={{ flex: 1 }}>
          <TransactionFiltersBar
            filters={filters}
            categories={categories}
            onChange={setFilters}
          />
        </div>
        <div style={{ display: 'flex', alignItems: 'center', padding: '0 12px', borderBottom: '1px solid var(--border-color)', background: 'var(--bg-sidebar)' }}>
          <Button variant="ghost" size="sm" onClick={() => setShowTransfer(true)}>⇅ Transfer</Button>
        </div>
      </div>

      {/* Transaction list */}
      {loading ? (
        <div style={{ display: 'flex', justifyContent: 'center', paddingTop: 32 }}><Spinner /></div>
      ) : (
        <TransactionList
          page={page}
          onPageChange={handlePageChange}
          onRowClick={setEditing}
        />
      )}

      {/* Edit modal */}
      {editing && (
        <TransactionEditModal
          transaction={editing}
          categories={categories}
          onSaved={refresh}
          onClose={() => setEditing(null)}
        />
      )}

      {/* Transfer modal */}
      {showTransfer && (
        <TransferForm
          accounts={accounts}
          onSaved={refresh}
          onClose={() => setShowTransfer(false)}
        />
      )}
    </div>
  );
}

export function TransactionsPage() {
  const { activeAccountId } = useApp();
  return (
    <TransactionProvider initialAccountId={activeAccountId}>
      <TransactionsInner />
    </TransactionProvider>
  );
}
