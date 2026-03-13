import { Transaction } from '../../types';
import { formatCurrency } from '../../utils/formatCurrency';
import { formatDate } from '../../utils/formatDate';
import { cx } from '../../utils/classNames';
import styles from './TransactionRow.module.css';

interface TransactionRowProps {
  transaction: Transaction;
  onClick: (t: Transaction) => void;
}

const AMOUNT_PREFIX: Record<string, string> = {
  income: '+',
  expense: '-',
  transfer: '',
};

export function TransactionRow({ transaction: t, onClick }: TransactionRowProps) {
  return (
    <div className={styles.row} onClick={() => onClick(t)}>
      <span className={styles.date}>{formatDate(t.date)}</span>

      <div>
        <div className={styles.payee}>{t.payee || '—'}</div>
        {t.description && <div className={styles.description}>{t.description}</div>}
      </div>

      <div className={styles.category}>
        {t.categoryName ? (
          <>
            <span className={styles.categoryDot} style={{ background: t.categoryColor ?? '#6366f1' }} />
            <span className={styles.categoryName}>{t.categoryName}</span>
          </>
        ) : t.kind === 'transfer' ? (
          <span style={{ color: 'var(--color-transfer)' }}>Transfer</span>
        ) : (
          <span style={{ color: 'var(--text-muted)' }}>Uncategorized</span>
        )}
      </div>

      <span className={cx(styles.amount, styles[t.kind])}>
        {AMOUNT_PREFIX[t.kind]}{formatCurrency(t.amount)}
      </span>

      <span className={styles.cleared}>{t.isCleared ? '✓' : ''}</span>
    </div>
  );
}
