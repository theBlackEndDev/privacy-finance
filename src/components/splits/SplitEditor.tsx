import { useState, useEffect } from 'react';
import { Category, CreateSplitInput, TransactionSplit } from '../../types';
import { splitsApi } from '../../api/splits';
import { formatCurrency, parseCurrency } from '../../utils/formatCurrency';
import styles from './SplitEditor.module.css';

interface SplitEditorProps {
  transactionId: string;
  parentAmount: number;
  categories: Category[];
  onSaved: () => void;
}

interface SplitRow {
  categoryId: string;
  amount: string;
  description: string;
}

function toSplitRows(splits: TransactionSplit[]): SplitRow[] {
  return splits.map((s) => ({
    categoryId: s.categoryId,
    amount: formatCurrency(s.amount).replace('$', ''),
    description: s.description,
  }));
}

export function SplitEditor({ transactionId, parentAmount, categories, onSaved }: SplitEditorProps) {
  const [rows, setRows] = useState<SplitRow[]>([]);
  const [existing, setExisting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  useEffect(() => {
    splitsApi.getAll(transactionId).then((splits) => {
      if (splits.length > 0) {
        setRows(toSplitRows(splits));
        setExisting(true);
      } else {
        setRows([
          { categoryId: '', amount: formatCurrency(parentAmount).replace('$', ''), description: '' },
        ]);
      }
    }).catch(() => {});
  }, [transactionId, parentAmount]);

  const expenseCategories = categories.filter((c) => c.type === 'expense' && !c.isArchived);

  function updateRow(index: number, field: keyof SplitRow, value: string) {
    setRows((prev) => prev.map((r, i) => i === index ? { ...r, [field]: value } : r));
  }

  function addRow() {
    setRows((prev) => [...prev, { categoryId: '', amount: '0.00', description: '' }]);
  }

  function removeRow(index: number) {
    if (rows.length <= 1) return;
    setRows((prev) => prev.filter((_, i) => i !== index));
  }

  const splitSum = rows.reduce((sum, r) => sum + parseCurrency(r.amount), 0);
  const isValid = splitSum === parentAmount && rows.every((r) => r.categoryId !== '');

  async function handleSave() {
    if (!isValid) return;
    setSaving(true);
    setError('');
    try {
      const inputs: CreateSplitInput[] = rows.map((r) => ({
        categoryId: r.categoryId,
        amount: parseCurrency(r.amount),
        description: r.description,
      }));
      if (existing) {
        await splitsApi.update(transactionId, inputs);
      } else {
        await splitsApi.create(transactionId, inputs);
      }
      onSaved();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }

  async function handleRemoveSplits() {
    try {
      await splitsApi.delete(transactionId);
      onSaved();
    } catch (err) {
      setError(String(err));
    }
  }

  return (
    <div className={styles.editor}>
      <div className={styles.header}>
        <span>Split Transaction</span>
        {existing && (
          <button className={styles.removeBtn} onClick={handleRemoveSplits} type="button" title="Remove all splits">
            Remove Splits
          </button>
        )}
      </div>

      {rows.map((row, i) => (
        <div key={i} className={styles.splitRow}>
          <select
            className={styles.select}
            value={row.categoryId}
            onChange={(e) => updateRow(i, 'categoryId', e.target.value)}
          >
            <option value="">Category...</option>
            {expenseCategories.map((c) => (
              <option key={c.id} value={c.id}>{c.name}</option>
            ))}
          </select>

          <input
            className={styles.amountInput}
            value={row.amount}
            onChange={(e) => updateRow(i, 'amount', e.target.value)}
            placeholder="0.00"
          />

          <input
            className={styles.descInput}
            value={row.description}
            onChange={(e) => updateRow(i, 'description', e.target.value)}
            placeholder="Description..."
          />

          <button
            className={styles.removeBtn}
            onClick={() => removeRow(i)}
            type="button"
            disabled={rows.length <= 1}
          >
            &times;
          </button>
        </div>
      ))}

      <div className={styles.footer}>
        <div>
          <button className={styles.addBtn} onClick={addRow} type="button">+ Add row</button>
          <span className={`${styles.total} ${isValid ? styles.valid : styles.invalid}`}>
            {' '}{formatCurrency(splitSum)} / {formatCurrency(parentAmount)}
          </span>
        </div>
        <button
          className={styles.addBtn}
          onClick={handleSave}
          disabled={!isValid || saving}
          type="button"
          style={{ fontWeight: 600 }}
        >
          {saving ? 'Saving...' : existing ? 'Update Splits' : 'Save Splits'}
        </button>
      </div>

      {error && <p style={{ color: 'var(--color-expense)', fontSize: 12 }}>{error}</p>}
    </div>
  );
}
