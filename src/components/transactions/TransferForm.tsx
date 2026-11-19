import { useState, FormEvent } from 'react';
import { Account, CreateTransferInput } from '../../types';
import { transactionsApi } from '../../api/transactions';
import { parseCurrency } from '../../utils/formatCurrency';
import { todayISO } from '../../utils/formatDate';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';
import { Modal } from '../ui/Modal';
import styles from './TransferForm.module.css';

interface TransferFormProps {
  accounts: Account[];
  onSaved: () => void;
  onClose: () => void;
}

export function TransferForm({ accounts, onSaved, onClose }: TransferFormProps) {
  const active = accounts.filter((a) => !a.isArchived);
  const [fromId, setFromId] = useState(active[0]?.id ?? '');
  const [toId, setToId] = useState(active[1]?.id ?? '');
  const [amount, setAmount] = useState('');
  const [date, setDate] = useState(todayISO());
  const [description, setDescription] = useState('');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (fromId === toId) { setError('From and To accounts must be different'); return; }
    if (!amount) { setError('Amount is required'); return; }
    const cents = parseCurrency(amount);
    if (cents <= 0) { setError('Amount must be greater than 0'); return; }
    setSaving(true);
    try {
      const input: CreateTransferInput = {
        fromAccountId: fromId,
        toAccountId: toId,
        amount: cents,
        date,
        description: description || undefined,
      };
      await transactionsApi.createTransfer(input);
      onSaved();
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal title="Transfer Between Accounts" onClose={onClose}>
      <form className={styles.form} onSubmit={handleSubmit}>
        <div className={styles.row}>
          <div>
            <label className={styles.selectLabel}>From</label>
            <select className={styles.select} value={fromId} data-testid="transfer-from-account" onChange={(e) => setFromId(e.target.value)}>
              {active.map((a) => <option key={a.id} value={a.id}>{a.name}</option>)}
            </select>
          </div>
          <div className={styles.arrow}>→</div>
          <div>
            <label className={styles.selectLabel}>To</label>
            <select className={styles.select} value={toId} data-testid="transfer-to-account" onChange={(e) => setToId(e.target.value)}>
              {active.map((a) => <option key={a.id} value={a.id}>{a.name}</option>)}
            </select>
          </div>
        </div>

        <Input label="Amount" value={amount} onChange={(e) => setAmount(e.target.value)} placeholder="0.00" autoFocus data-testid="transfer-amount" />
        <Input label="Date" type="date" value={date} onChange={(e) => setDate(e.target.value)} />
        <Input label="Description (optional)" value={description} onChange={(e) => setDescription(e.target.value)} placeholder="e.g. Monthly savings transfer" />

        {error && <p style={{ color: 'var(--color-expense)', fontSize: 13 }}>{error}</p>}

        <div className={styles.footer}>
          <Button variant="ghost" type="button" onClick={onClose}>Cancel</Button>
          <Button type="submit" disabled={saving} data-testid="transfer-submit-btn">{saving ? 'Transferring…' : 'Transfer'}</Button>
        </div>
      </form>
    </Modal>
  );
}
