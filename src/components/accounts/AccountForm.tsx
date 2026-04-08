import { useState, FormEvent } from 'react';
import { Account, CreateAccountInput, UpdateAccountInput } from '../../types';
import { Button } from '../ui/Button';
import { Input } from '../ui/Input';
import { Modal } from '../ui/Modal';
import { Tooltip } from '../ui/Tooltip';
import { parseCurrency, formatCurrency } from '../../utils/formatCurrency';
import { ACCOUNT_TYPE_OPTIONS } from '../../utils/accountTypes';
import styles from './AccountForm.module.css';

interface AccountFormProps {
  account?: Account;
  onSave: (input: CreateAccountInput | UpdateAccountInput) => Promise<void>;
  onClose: () => void;
}

export function AccountForm({ account, onSave, onClose }: AccountFormProps) {
  const [name, setName] = useState(account?.name ?? '');
  const [type, setType] = useState(account?.type ?? 'checking');
  const [openingBalance, setOpeningBalance] = useState(
    account ? formatCurrency(account.openingBalance).replace('$', '') : '0.00'
  );
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (!name.trim()) { setError('Name is required'); return; }
    setSaving(true);
    try {
      await onSave({
        name: name.trim(),
        type: type as Account['type'],
        openingBalance: parseCurrency(openingBalance),
      });
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Modal title={account ? 'Edit Account' : 'Add Account'} onClose={onClose}>
      <form className={styles.form} onSubmit={handleSubmit}>
        <Input
          label="Account Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="e.g. Primary Checking"
          autoFocus
          error={error && !name.trim() ? error : undefined}
          data-testid="account-name-input"
        />

        <div>
          <label htmlFor="account-type" className={styles.selectLabel}>
            Account Type{' '}
            <Tooltip text="Checking for daily spending, savings for reserves, credit card for tracking debt, cash for physical money, investment for brokerage or retirement accounts." />
          </label>
          <select id="account-type" className={styles.select} value={type} onChange={(e) => setType(e.target.value as Account['type'])} data-testid="account-type-select">
            {ACCOUNT_TYPE_OPTIONS.map((t) => (
              <option key={t.value} value={t.value}>{t.label}</option>
            ))}
          </select>
        </div>

        <Input
          label="Opening Balance"
          value={openingBalance}
          onChange={(e) => setOpeningBalance(e.target.value)}
          placeholder="0.00"
          data-testid="account-balance-input"
        />

        {error && name.trim() && <p style={{ color: 'var(--color-expense)', fontSize: 13 }}>{error}</p>}

        <div className={styles.footer}>
          <Button variant="ghost" type="button" onClick={onClose}>Cancel</Button>
          <Button type="submit" disabled={saving} data-testid="create-account-btn">{saving ? 'Saving…' : 'Save'}</Button>
        </div>
      </form>
    </Modal>
  );
}
