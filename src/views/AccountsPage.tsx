import { useState, useEffect, useCallback } from 'react';
import { Account, CreateAccountInput, UpdateAccountInput } from '../types';
import { accountsApi } from '../api/accounts';
import { useApp } from '../context/AppContext';
import { useConfirmDelete } from '../hooks/useConfirmDelete';
import { AccountList } from '../components/accounts/AccountList';
import { AccountForm } from '../components/accounts/AccountForm';
import { ConfirmDialog } from '../components/ui/ConfirmDialog';
import { Spinner } from '../components/ui/Spinner';
import { useToast } from '../components/ui/Toast';

export function AccountsPage() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [editing, setEditing] = useState<Account | undefined>();
  const { navigate, setActiveAccountId } = useApp();
  const toast = useToast();

  const load = useCallback(async () => {
    try {
      const data = await accountsApi.getAll();
      setAccounts(data);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  async function handleSave(input: CreateAccountInput | UpdateAccountInput) {
    if (editing) {
      await accountsApi.update(editing.id, input as UpdateAccountInput);
      toast('Account updated');
    } else {
      await accountsApi.create(input as CreateAccountInput);
      toast('Account created');
    }
    await load();
  }

  const deleteFlow = useConfirmDelete<Account>({
    onConfirm: async (account) => {
      await accountsApi.delete(account.id);
      await load();
      toast('Account deleted');
    },
  });

  const handleClickAccount = useCallback((account: Account) => {
    setActiveAccountId(account.id);
    navigate('transactions');
  }, [setActiveAccountId, navigate]);

  function openAdd() { setEditing(undefined); setShowForm(true); }
  function openEdit(account: Account) { setEditing(account); setShowForm(true); }
  function closeForm() { setShowForm(false); setEditing(undefined); }

  if (loading) return <div style={{ display: 'flex', justifyContent: 'center', paddingTop: 48 }}><Spinner /></div>;

  return (
    <>
      <AccountList
        accounts={accounts}
        onAdd={openAdd}
        onEdit={openEdit}
        onDelete={deleteFlow.request}
        onClickAccount={handleClickAccount}
      />
      {showForm && <AccountForm account={editing} onSave={handleSave} onClose={closeForm} />}
      {deleteFlow.pending && (
        <ConfirmDialog
          title="Delete Account"
          message={`Delete "${deleteFlow.pending.name}"? This cannot be undone if there are no transactions.`}
          confirmLabel="Delete"
          onConfirm={deleteFlow.confirm}
          onCancel={deleteFlow.cancel}
        />
      )}
    </>
  );
}
