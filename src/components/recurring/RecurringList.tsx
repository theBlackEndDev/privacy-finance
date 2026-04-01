import { RecurringTransaction } from '../../types';
import { formatCurrency } from '../../utils/formatCurrency';
import { cx } from '../../utils/classNames';
import styles from './RecurringList.module.css';

const FREQUENCY_LABELS: Record<string, string> = {
  daily: 'Daily',
  weekly: 'Weekly',
  biweekly: 'Bi-weekly',
  monthly: 'Monthly',
  quarterly: 'Quarterly',
  yearly: 'Yearly',
};

interface RecurringListProps {
  items: RecurringTransaction[];
  onEdit: (item: RecurringTransaction) => void;
  onDelete: (id: string) => void;
  onToggleActive: (id: string) => void;
}

export function RecurringList({ items, onEdit, onDelete, onToggleActive }: RecurringListProps) {
  return (
    <div className={styles.container}>
      <table className={styles.table}>
        <thead>
          <tr>
            <th>Payee</th>
            <th>Amount</th>
            <th>Frequency</th>
            <th>Next Date</th>
            <th>Status</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {items.map((item) => (
            <tr key={item.id} data-testid={`recurring-row-${item.id}`}>
              <td>{item.payee || item.description || '(no payee)'}</td>
              <td>
                <span className={cx(styles.amount, styles[item.kind])}>
                  {item.kind === 'income' ? '+' : item.kind === 'expense' ? '-' : ''}
                  {formatCurrency(item.amount)}
                </span>
              </td>
              <td>{FREQUENCY_LABELS[item.frequency] ?? item.frequency}</td>
              <td>{item.nextDate}</td>
              <td>
                <span className={cx(styles.badge, item.isActive ? styles.badgeActive : styles.badgePaused)}>
                  {item.isActive ? 'Active' : 'Paused'}
                </span>
              </td>
              <td>
                <div className={styles.actions}>
                  <button
                    className={styles.actionBtn}
                    onClick={() => onToggleActive(item.id)}
                    title={item.isActive ? 'Pause' : 'Resume'}
                    data-testid={`recurring-toggle-${item.id}`}
                  >
                    {item.isActive ? '⏸' : '▶'}
                  </button>
                  <button
                    className={styles.actionBtn}
                    onClick={() => onEdit(item)}
                    title="Edit"
                  >
                    ✎
                  </button>
                  <button
                    className={styles.actionBtn}
                    onClick={() => onDelete(item.id)}
                    title="Delete"
                  >
                    ✕
                  </button>
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
