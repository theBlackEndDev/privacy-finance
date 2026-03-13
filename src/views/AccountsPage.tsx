import { useState, useEffect, useCallback } from 'react';
import { Account, CreateAccountInput, UpdateAccountInput } from '../types';
import { accountsApi } from '../api/accounts';
import { AccountList } from '../components/accounts/AccountList';
import { AccountForm } from '../components/accounts/AccountForm';
import { Spinner } from '../components/ui/Spinner';

export function AccountsPage() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [editing, setEditing] = useState<Account | undefined>();

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
    } else {
      await accountsApi.create(input as CreateAccountInput);
    }
    await load();
  }

  async function handleDelete(account: Account) {
    if (!confirm(`Delete "${account.name}"? This cannot be undone if there are no transactions.`)) return;
    try {
      await accountsApi.delete(account.id);
      await load();
    } catch (err) {
      alert(String(err));
    }
  }

  function openAdd() { setEditing(undefined); setShowForm(true); }
  function openEdit(account: Account) { setEditing(account); setShowForm(true); }
  function closeForm() { setShowForm(false); setEditing(undefined); }

  if (loading) return <div style={{ display: 'flex', justifyContent: 'center', paddingTop: 48 }}><Spinner /></div>;

  return (
    <>
      <AccountList accounts={accounts} onAdd={openAdd} onEdit={openEdit} onDelete={handleDelete} />
      {showForm && <AccountForm account={editing} onSave={handleSave} onClose={closeForm} />}
    </>
  );
}
