import { DueRecurringItem } from '../../types';
import { formatCurrency } from '../../utils/formatCurrency';
import { Modal } from '../ui/Modal';
import { Button } from '../ui/Button';
import styles from './RecurringList.module.css';
import { cx } from '../../utils/classNames';

interface RecurringDueModalProps {
  dueItems: DueRecurringItem[];
  onGenerateAll: () => void;
  onDismiss: () => void;
  loading?: boolean;
}

export function RecurringDueModal({ dueItems, onGenerateAll, onDismiss, loading }: RecurringDueModalProps) {
  if (dueItems.length === 0) return null;

  const totalTransactions = dueItems.reduce((sum, item) => sum + item.missedCount, 0);

  return (
    <Modal title="Recurring Transactions Due" onClose={onDismiss}>
      <p style={{ marginBottom: '1rem', color: 'var(--text-secondary)' }}>
        {totalTransactions} transaction{totalTransactions !== 1 ? 's' : ''} ready to generate from {dueItems.length} recurring item{dueItems.length !== 1 ? 's' : ''}.
      </p>

      <table className={styles.table}>
        <thead>
          <tr>
            <th>Payee</th>
            <th>Amount</th>
            <th>Missed</th>
          </tr>
        </thead>
        <tbody>
          {dueItems.map((item) => (
            <tr key={item.recurring.id}>
              <td>{item.recurring.payee || item.recurring.description || '(no payee)'}</td>
              <td>
                <span className={cx(styles.amount, styles[item.recurring.kind])}>
                  {formatCurrency(item.recurring.amount)}
                </span>
              </td>
              <td>{item.missedCount}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <div style={{ display: 'flex', gap: '0.75rem', justifyContent: 'flex-end', marginTop: '1.5rem' }}>
        <Button variant="ghost" onClick={onDismiss}>Dismiss</Button>
        <Button onClick={onGenerateAll} disabled={loading} data-testid="recurring-generate-btn">
          {loading ? 'Generating...' : `Generate All (${totalTransactions})`}
        </Button>
      </div>
    </Modal>
  );
}
