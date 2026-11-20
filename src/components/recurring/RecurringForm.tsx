import { useState } from 'react';
import { CreateRecurringInput, RecurringTransaction, Frequency } from '../../types';
import { useAccounts } from '../../context/AccountContext';
import { useCategories } from '../../context/CategoryContext';
import { Button } from '../ui/Button';
import { todayISO } from '../../utils/formatDate';
import styles from './RecurringForm.module.css';

const FREQUENCIES: { value: Frequency; label: string }[] = [
  { value: 'daily', label: 'Daily' },
  { value: 'weekly', label: 'Weekly' },
  { value: 'biweekly', label: 'Bi-weekly' },
  { value: 'monthly', label: 'Monthly' },
  { value: 'quarterly', label: 'Quarterly' },
  { value: 'yearly', label: 'Yearly' },
];

interface RecurringFormProps {
  initial?: RecurringTransaction;
  onSubmit: (input: CreateRecurringInput) => void;
  onCancel: () => void;
}

export function RecurringForm({ initial, onSubmit, onCancel }: RecurringFormProps) {
  const { accounts } = useAccounts();
  const { categories } = useCategories();

  const [accountId, setAccountId] = useState(initial?.accountId ?? accounts[0]?.id ?? '');
  const [payee, setPayee] = useState(initial?.payee ?? '');
  const [description, setDescription] = useState(initial?.description ?? '');
  const [amount, setAmount] = useState(initial ? (initial.amount / 100).toFixed(2) : '');
  const [kind, setKind] = useState<'income' | 'expense' | 'transfer'>(initial?.kind ?? 'expense');
  const [categoryId, setCategoryId] = useState(initial?.categoryId ?? '');
  const [transferToAccountId, setTransferToAccountId] = useState(initial?.transferToAccountId ?? '');
  const [frequency, setFrequency] = useState<Frequency>(initial?.frequency ?? 'monthly');
  const [startDate, setStartDate] = useState(initial?.startDate ?? todayISO());
  const [endDate, setEndDate] = useState(initial?.endDate ?? '');
  const [autoPost, setAutoPost] = useState(initial?.autoPost ?? false);

  const activeAccounts = accounts.filter((a) => !a.isArchived);
  const filteredCategories = categories.filter((c) => c.type === (kind === 'income' ? 'income' : 'expense'));

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const cents = Math.round(parseFloat(amount) * 100);
    if (isNaN(cents) || cents <= 0) return;

    onSubmit({
      accountId,
      payee: payee || undefined,
      description: description || undefined,
      amount: cents,
      kind,
      categoryId: kind !== 'transfer' && categoryId ? categoryId : undefined,
      transferToAccountId: kind === 'transfer' && transferToAccountId ? transferToAccountId : undefined,
      frequency,
      startDate,
      endDate: endDate || undefined,
      autoPost,
    });
  }

  return (
    <form className={styles.form} onSubmit={handleSubmit}>
      <div className={styles.row}>
        <div className={styles.field}>
          <label>Account</label>
          <select value={accountId} onChange={(e) => setAccountId(e.target.value)}>
            {activeAccounts.map((a) => (
              <option key={a.id} value={a.id}>{a.name}</option>
            ))}
          </select>
        </div>
        <div className={styles.field}>
          <label>Type</label>
          <select value={kind} onChange={(e) => setKind(e.target.value as typeof kind)}>
            <option value="expense">Expense</option>
            <option value="income">Income</option>
            <option value="transfer">Transfer</option>
          </select>
        </div>
      </div>

      <div className={styles.row}>
        <div className={styles.field}>
          <label>Payee</label>
          <input type="text" value={payee} onChange={(e) => setPayee(e.target.value)} placeholder="e.g. Netflix" data-testid="recurring-payee-input" />
        </div>
        <div className={styles.field}>
          <label>Amount</label>
          <input type="number" step="0.01" min="0.01" value={amount} onChange={(e) => setAmount(e.target.value)} required data-testid="recurring-amount-input" />
        </div>
      </div>

      <div className={styles.field}>
        <label>Description</label>
        <input type="text" value={description} onChange={(e) => setDescription(e.target.value)} placeholder="Optional" />
      </div>

      {kind === 'transfer' ? (
        <div className={styles.field}>
          <label>Transfer To Account</label>
          <select value={transferToAccountId} onChange={(e) => setTransferToAccountId(e.target.value)}>
            <option value="">Select account...</option>
            {activeAccounts.filter((a) => a.id !== accountId).map((a) => (
              <option key={a.id} value={a.id}>{a.name}</option>
            ))}
          </select>
        </div>
      ) : (
        <div className={styles.field}>
          <label>Category</label>
          <select value={categoryId} onChange={(e) => setCategoryId(e.target.value)}>
            <option value="">No category</option>
            {filteredCategories.map((c) => (
              <option key={c.id} value={c.id}>{c.name}</option>
            ))}
          </select>
        </div>
      )}

      <div className={styles.row}>
        <div className={styles.field}>
          <label>Frequency</label>
          <select value={frequency} onChange={(e) => setFrequency(e.target.value as Frequency)} data-testid="recurring-freq-select">
            {FREQUENCIES.map((f) => (
              <option key={f.value} value={f.value}>{f.label}</option>
            ))}
          </select>
        </div>
        <div className={styles.field}>
          <label>Start Date</label>
          <input type="date" value={startDate} onChange={(e) => setStartDate(e.target.value)} required />
        </div>
      </div>

      <div className={styles.row}>
        <div className={styles.field}>
          <label>End Date (optional)</label>
          <input type="date" value={endDate} min={startDate} onChange={(e) => setEndDate(e.target.value)} />
        </div>
        <div className={styles.field}>
          <label>&nbsp;</label>
          <div className={styles.toggle}>
            <input type="checkbox" checked={autoPost} onChange={(e) => setAutoPost(e.target.checked)} id="autoPost" />
            <label htmlFor="autoPost">Auto-post (no confirmation)</label>
          </div>
        </div>
      </div>

      <div className={styles.actions}>
        <Button type="button" variant="ghost" onClick={onCancel}>Cancel</Button>
        <Button type="submit" data-testid="recurring-submit-btn">{initial ? 'Update' : 'Create'}</Button>
      </div>
    </form>
  );
}
