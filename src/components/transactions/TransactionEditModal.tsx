import { useState, FormEvent } from 'react';
import { Transaction, Category, UpdateTransactionInput, TagBrief } from '../../types';
import { transactionsApi } from '../../api/transactions';
import { Modal } from '../ui/Modal';
import { Input } from '../ui/Input';
import { Button } from '../ui/Button';
import { ConfirmDialog } from '../ui/ConfirmDialog';
import { useToast } from '../ui/Toast';
import { parseCurrency } from '../../utils/formatCurrency';
import { todayISO } from '../../utils/formatDate';
import { SplitEditor } from '../splits/SplitEditor';
import { TagPicker } from '../tags/TagPicker';
import styles from './TransactionEditModal.module.css';

interface TransactionEditModalProps {
  transaction: Transaction;
  categories: Category[];
  onSaved: () => void;
  onClose: () => void;
}

export function TransactionEditModal({ transaction: t, categories, onSaved, onClose }: TransactionEditModalProps) {
  const today = todayISO();
  const [date, setDate] = useState(t.date);
  const [payee, setPayee] = useState(t.payee);
  const [amount, setAmount] = useState((Math.abs(t.amount) / 100).toFixed(2));
  const [kind, setKind] = useState(t.kind);
  const [categoryId, setCategoryId] = useState(t.categoryId ?? '');
  const [description, setDescription] = useState(t.description);
  const [isCleared, setIsCleared] = useState(t.isCleared);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [showSplit, setShowSplit] = useState(t.isSplit);
  const [tags, setTags] = useState<TagBrief[]>(t.tags ?? []);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const isFutureDate = date > today;
  const toast = useToast();

  const filteredCategories = categories.filter(
    (c) => kind === 'income' ? c.type === 'income' : c.type === 'expense'
  );

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    const cents = parseCurrency(amount);
    if (cents <= 0) { setError('Amount must be greater than 0'); return; }
    // Outgoing transfer legs are stored negative; keep the sign the user cannot see.
    const signedCents = kind === 'transfer' && t.kind === 'transfer' && t.amount < 0 ? -cents : cents;
    setSaving(true);
    try {
      const input: UpdateTransactionInput = {
        date, payee, amount: signedCents, kind,
        categoryId: categoryId || null, description,
        isCleared: isFutureDate ? false : isCleared,
      };
      await transactionsApi.update(t.id, input);
      toast('Transaction updated');
      onSaved();
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete() {
    try {
      await transactionsApi.delete(t.id);
      toast('Transaction deleted');
      onSaved();
      onClose();
    } catch (err) {
      setError(String(err));
      setShowDeleteConfirm(false);
    }
  }

  return (
    <>
    <Modal title="Edit Transaction" onClose={onClose}>
      <form className={styles.form} onSubmit={handleSubmit} data-testid="tx-edit-modal">
        <div className={styles.row}>
          <Input label="Date" type="date" value={date} onChange={(e) => setDate(e.target.value)} />
          <Input label="Amount" value={amount} onChange={(e) => setAmount(e.target.value)} placeholder="0.00" data-testid="tx-edit-amount" />
        </div>

        <Input label="Payee" value={payee} onChange={(e) => setPayee(e.target.value)} placeholder="Payee" autoFocus data-testid="tx-edit-payee" />

        <div className={styles.row}>
          <div>
            <label className={styles.selectLabel}>Kind</label>
            <select className={styles.select} value={kind}
              onChange={(e) => setKind(e.target.value as Transaction['kind'])}>
              <option value="expense">Expense</option>
              <option value="income">Income</option>
              <option value="transfer">Transfer</option>
            </select>
          </div>

          {kind !== 'transfer' && (
            <div>
              <label className={styles.selectLabel}>Category</label>
              <select className={styles.select} value={categoryId} onChange={(e) => setCategoryId(e.target.value)}>
                <option value="">None</option>
                {filteredCategories.map((c) => (
                  <option key={c.id} value={c.id}>{c.name}</option>
                ))}
              </select>
            </div>
          )}
        </div>

        <Input label="Description" value={description} onChange={(e) => setDescription(e.target.value)} placeholder="Optional note" />

        <label style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 14, color: isFutureDate ? 'var(--text-muted)' : 'var(--text-secondary)', cursor: isFutureDate ? 'not-allowed' : 'pointer' }}>
          <input
            type="checkbox"
            checked={isFutureDate ? false : isCleared}
            onChange={(e) => setIsCleared(e.target.checked)}
            disabled={isFutureDate}
          />
          {isFutureDate ? 'Pending (future date)' : 'Cleared'}
        </label>

        {kind === 'expense' && (
          <div>
            {!showSplit ? (
              <Button variant="ghost" type="button" size="sm" onClick={() => setShowSplit(true)}>
                Split Transaction
              </Button>
            ) : (
              <SplitEditor
                transactionId={t.id}
                parentAmount={parseCurrency(amount)}
                categories={categories}
                onSaved={onSaved}
              />
            )}
          </div>
        )}

        <TagPicker
          selectedTags={tags}
          onTagsChange={setTags}
          transactionId={t.id}
        />

        {error && <p style={{ color: 'var(--color-expense)', fontSize: 13 }}>{error}</p>}

        <div className={styles.footer}>
          <Button variant="danger" type="button" size="sm" data-testid="tx-edit-delete" onClick={() => setShowDeleteConfirm(true)}>Delete</Button>
          <div className={styles.footerRight}>
            <Button variant="ghost" type="button" onClick={onClose}>Cancel</Button>
            <Button type="submit" disabled={saving} data-testid="tx-edit-save">{saving ? 'Saving…' : 'Save'}</Button>
          </div>
        </div>
      </form>
    </Modal>
    {showDeleteConfirm && (
      <ConfirmDialog
        title="Delete Transaction"
        message="Delete this transaction? This cannot be undone."
        confirmLabel="Delete"
        onConfirm={handleDelete}
        onCancel={() => setShowDeleteConfirm(false)}
      />
    )}
    </>
  );
}
