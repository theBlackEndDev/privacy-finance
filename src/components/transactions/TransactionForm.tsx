import { useState, useEffect, useRef, FormEvent } from 'react';
import { Account, Category, CreateTransactionInput } from '../../types';
import { transactionsApi } from '../../api/transactions';
import { parseCurrency } from '../../utils/formatCurrency';
import { todayISO } from '../../utils/formatDate';
import { Button } from '../ui/Button';
import styles from './TransactionForm.module.css';

interface TransactionFormProps {
  accounts: Account[];
  categories: Category[];
  defaultAccountId?: string | null;
  onSaved: () => void;
  focusRef?: React.RefObject<HTMLInputElement | null>;
}

type Kind = 'income' | 'expense' | 'transfer';

export function TransactionForm({ accounts, categories, defaultAccountId, onSaved, focusRef }: TransactionFormProps) {
  const [accountId, setAccountId] = useState(defaultAccountId ?? accounts[0]?.id ?? '');
  const [date, setDate] = useState(todayISO());
  const [payee, setPayee] = useState('');
  const [categoryId, setCategoryId] = useState('');
  const [amount, setAmount] = useState('');
  const [kind, setKind] = useState<Kind>('expense');
  const [saving, setSaving] = useState(false);
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const dateRef = useRef<HTMLInputElement>(null);

  // Expose date input for Ctrl+N focus
  useEffect(() => {
    if (focusRef && dateRef.current) {
      (focusRef as React.MutableRefObject<HTMLInputElement | null>).current = dateRef.current;
    }
  }, [focusRef]);

  // Sync accountId when default changes
  useEffect(() => {
    if (defaultAccountId) setAccountId(defaultAccountId);
  }, [defaultAccountId]);

  async function fetchSuggestions(value: string) {
    if (value.length < 2) { setSuggestions([]); return; }
    try {
      const results = await transactionsApi.getPayees(value);
      setSuggestions(results);
    } catch { setSuggestions([]); }
  }

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (!accountId || !amount) return;
    setSaving(true);
    try {
      const input: CreateTransactionInput = {
        accountId,
        date,
        amount: parseCurrency(amount),
        kind,
        payee,
        categoryId: categoryId || null,
      };
      await transactionsApi.create(input);
      setPayee(''); setAmount(''); setCategoryId(''); setDate(todayISO());
      dateRef.current?.focus();
      onSaved();
    } finally {
      setSaving(false);
    }
  }

  const filteredCategories = categories.filter(
    (c) => kind === 'income' ? c.type === 'income' : c.type === 'expense'
  );

  return (
    <form className={styles.bar} onSubmit={handleSubmit}>
      {/* Account */}
      <div className={styles.field}>
        <span className={styles.label}>Account</span>
        <select className={styles.select} value={accountId} onChange={(e) => setAccountId(e.target.value)}>
          {accounts.filter(a => !a.isArchived).map((a) => (
            <option key={a.id} value={a.id}>{a.name}</option>
          ))}
        </select>
      </div>

      {/* Date */}
      <div className={styles.field}>
        <span className={styles.label}>Date</span>
        <input ref={dateRef} type="date" className={styles.input} value={date}
          onChange={(e) => setDate(e.target.value)} />
      </div>

      {/* Payee with autocomplete */}
      <div className={styles.field}>
        <span className={styles.label}>Payee</span>
        <div className={styles.autocomplete}>
          <input
            className={styles.input}
            placeholder="Payee…"
            value={payee}
            style={{ width: 160 }}
            onChange={(e) => { setPayee(e.target.value); fetchSuggestions(e.target.value); setShowSuggestions(true); }}
            onBlur={() => setTimeout(() => setShowSuggestions(false), 150)}
          />
          {showSuggestions && suggestions.length > 0 && (
            <div className={styles.suggestions}>
              {suggestions.map((s) => (
                <div key={s} className={styles.suggestion} onMouseDown={() => { setPayee(s); setShowSuggestions(false); }}>{s}</div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Kind toggle */}
      <div className={styles.field}>
        <span className={styles.label}>Kind</span>
        <div className={styles.kindToggle}>
          {(['income', 'expense', 'transfer'] as Kind[]).map((k) => (
            <button
              key={k}
              type="button"
              className={`${styles.kindBtn} ${styles[`kindBtn${k.charAt(0).toUpperCase() + k.slice(1)}`]} ${kind === k ? styles.active : ''}`}
              onClick={() => setKind(k)}
            >
              {k === 'income' ? '+' : k === 'expense' ? '−' : '⇅'}
            </button>
          ))}
        </div>
      </div>

      {/* Category (hidden for transfer) */}
      {kind !== 'transfer' && (
        <div className={styles.field}>
          <span className={styles.label}>Category</span>
          <select className={styles.select} value={categoryId} onChange={(e) => setCategoryId(e.target.value)}>
            <option value="">None</option>
            {filteredCategories.map((c) => (
              <option key={c.id} value={c.id}>{c.name}</option>
            ))}
          </select>
        </div>
      )}

      {/* Amount */}
      <div className={styles.field}>
        <span className={styles.label}>Amount</span>
        <input
          className={styles.input}
          placeholder="0.00"
          value={amount}
          style={{ width: 90 }}
          onChange={(e) => setAmount(e.target.value)}
        />
      </div>

      <Button type="submit" disabled={saving} size="sm" style={{ marginBottom: 1 }}>
        {saving ? '…' : 'Add'}
      </Button>
    </form>
  );
}
