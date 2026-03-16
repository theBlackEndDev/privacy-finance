import { useState, useEffect, useCallback } from 'react';
import { RecurringTransaction, CreateRecurringInput } from '../types';
import { recurringApi } from '../api/recurring';
import { RecurringList } from '../components/recurring/RecurringList';
import { RecurringForm } from '../components/recurring/RecurringForm';
import { Modal } from '../components/ui/Modal';
import { Button } from '../components/ui/Button';
import { EmptyState } from '../components/ui/EmptyState';
import { Spinner } from '../components/ui/Spinner';
import { useToast } from '../components/ui/Toast';

export function RecurringPage() {
  const [items, setItems] = useState<RecurringTransaction[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [editing, setEditing] = useState<RecurringTransaction | undefined>();
  const toast = useToast();

  const fetchItems = useCallback(async () => {
    setLoading(true);
    try {
      const data = await recurringApi.getAll();
      setItems(data);
    } catch (err) {
      toast(`Failed to load recurring: ${err}`, 'error');
    } finally {
      setLoading(false);
    }
  }, [toast]);

  useEffect(() => { fetchItems(); }, [fetchItems]);

  async function handleCreate(input: CreateRecurringInput) {
    try {
      await recurringApi.create(input);
      toast('Recurring transaction created', 'success');
      setShowForm(false);
      fetchItems();
    } catch (err) {
      toast(`Failed to create: ${err}`, 'error');
    }
  }

  async function handleUpdate(input: CreateRecurringInput) {
    if (!editing) return;
    try {
      await recurringApi.update(editing.id, input);
      toast('Recurring transaction updated', 'success');
      setEditing(undefined);
      setShowForm(false);
      fetchItems();
    } catch (err) {
      toast(`Failed to update: ${err}`, 'error');
    }
  }

  async function handleDelete(id: string) {
    try {
      await recurringApi.delete(id);
      toast('Recurring transaction deleted', 'success');
      fetchItems();
    } catch (err) {
      toast(`Failed to delete: ${err}`, 'error');
    }
  }

  async function handleToggleActive(id: string) {
    try {
      await recurringApi.toggleActive(id);
      fetchItems();
    } catch (err) {
      toast(`Failed to toggle: ${err}`, 'error');
    }
  }

  function handleEdit(item: RecurringTransaction) {
    setEditing(item);
    setShowForm(true);
  }

  function handleCancel() {
    setEditing(undefined);
    setShowForm(false);
  }

  if (loading) return <Spinner />;

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1.5rem' }}>
        <h1>Recurring Transactions</h1>
        <Button onClick={() => { setEditing(undefined); setShowForm(true); }}>+ New Recurring</Button>
      </div>

      {items.length === 0 ? (
        <EmptyState
          title="No recurring transactions"
          description="Create a recurring transaction for expenses or income that repeat on a schedule."
        />
      ) : (
        <RecurringList
          items={items}
          onEdit={handleEdit}
          onDelete={handleDelete}
          onToggleActive={handleToggleActive}
        />
      )}

      {showForm && (
        <Modal title={editing ? 'Edit Recurring' : 'New Recurring'} onClose={handleCancel}>
          <RecurringForm
            initial={editing}
            onSubmit={editing ? handleUpdate : handleCreate}
            onCancel={handleCancel}
          />
        </Modal>
      )}
    </div>
  );
}
