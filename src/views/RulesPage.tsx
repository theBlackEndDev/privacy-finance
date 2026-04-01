import { useCallback, useEffect, useState } from 'react';
import { CategorizationRule, CreateRuleInput } from '../types';
import { rulesApi } from '../api/rules';
import { useCategories } from '../context/CategoryContext';
import { useToast } from '../components/ui/Toast';
import { Button } from '../components/ui/Button';
import { Spinner } from '../components/ui/Spinner';
import { EmptyState } from '../components/ui/EmptyState';
import { Modal } from '../components/ui/Modal';
import { RulesList } from '../components/rules/RulesList';
import { RuleForm } from '../components/rules/RuleForm';
import { RuleTestModal } from '../components/rules/RuleTestModal';

export function RulesPage() {
  const [rules, setRules] = useState<CategorizationRule[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [editing, setEditing] = useState<CategorizationRule | undefined>();
  const [testing, setTesting] = useState<CategorizationRule | undefined>();
  const { categories } = useCategories();
  const toast = useToast();

  const fetchRules = useCallback(async () => {
    setLoading(true);
    try {
      const data = await rulesApi.getAll();
      setRules(data);
    } catch (err) {
      toast(`Failed to load rules: ${err}`, 'error');
    } finally {
      setLoading(false);
    }
  }, [toast]);

  useEffect(() => {
    fetchRules();
  }, [fetchRules]);

  async function handleCreate(input: CreateRuleInput) {
    try {
      await rulesApi.create(input);
      toast('Rule created', 'success');
      setShowForm(false);
      fetchRules();
    } catch (err) {
      toast(`Failed to create rule: ${err}`, 'error');
    }
  }

  async function handleUpdate(input: CreateRuleInput) {
    if (!editing) return;
    try {
      await rulesApi.update(editing.id, input);
      toast('Rule updated', 'success');
      setEditing(undefined);
      setShowForm(false);
      fetchRules();
    } catch (err) {
      toast(`Failed to update rule: ${err}`, 'error');
    }
  }

  async function handleDelete(id: string) {
    try {
      await rulesApi.delete(id);
      toast('Rule deleted', 'success');
      fetchRules();
    } catch (err) {
      toast(`Failed to delete rule: ${err}`, 'error');
    }
  }

  async function handleToggleActive(rule: CategorizationRule) {
    try {
      await rulesApi.update(rule.id, { isActive: !rule.isActive });
      fetchRules();
    } catch (err) {
      toast(`Failed to toggle rule: ${err}`, 'error');
    }
  }

  async function handleMove(id: string, direction: 'up' | 'down') {
    const idx = rules.findIndex((r) => r.id === id);
    if (idx < 0) return;
    const swapIdx = direction === 'up' ? idx - 1 : idx + 1;
    if (swapIdx < 0 || swapIdx >= rules.length) return;

    const newOrder = [...rules];
    [newOrder[idx], newOrder[swapIdx]] = [newOrder[swapIdx], newOrder[idx]];
    const ids = newOrder.map((r) => r.id);

    try {
      await rulesApi.reorder(ids);
      setRules(newOrder);
    } catch (err) {
      toast(`Failed to reorder: ${err}`, 'error');
    }
  }

  async function handleApplyAll() {
    try {
      const count = await rulesApi.applyToUncategorized();
      if (count > 0) {
        toast(`Categorized ${count} transaction${count !== 1 ? 's' : ''}`, 'success');
        fetchRules();
      } else {
        toast('No uncategorized transactions matched any rules', 'info');
      }
    } catch (err) {
      toast(`Failed to apply rules: ${err}`, 'error');
    }
  }

  function handleEdit(rule: CategorizationRule) {
    setEditing(rule);
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
        <h1>Auto-Categorization Rules</h1>
        <div style={{ display: 'flex', gap: '0.75rem' }}>
          <Button onClick={handleApplyAll} data-testid="rule-apply-all-btn">Apply Rules</Button>
          <Button onClick={() => { setEditing(undefined); setShowForm(true); }} data-testid="rule-create-btn">+ New Rule</Button>
        </div>
      </div>

      {rules.length === 0 ? (
        <EmptyState
          title="No categorization rules"
          description="Create rules to automatically assign categories to transactions based on payee, description, or amount patterns."
        />
      ) : (
        <RulesList
          rules={rules}
          onEdit={handleEdit}
          onDelete={handleDelete}
          onToggleActive={handleToggleActive}
          onMove={handleMove}
          onTest={setTesting}
        />
      )}

      {showForm && (
        <Modal title={editing ? 'Edit Rule' : 'New Rule'} onClose={handleCancel}>
          <RuleForm
            initial={editing}
            categories={categories}
            onSubmit={editing ? handleUpdate : handleCreate}
            onCancel={handleCancel}
          />
        </Modal>
      )}

      {testing && (
        <RuleTestModal rule={testing} onClose={() => setTesting(undefined)} />
      )}
    </div>
  );
}
