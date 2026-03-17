import { Transaction } from '../../types';
import { formatCurrency } from '../../utils/formatCurrency';
import { formatDate } from '../../utils/formatDate';
import { cx } from '../../utils/classNames';
import { TagBadge } from '../tags/TagBadge';
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
    <div className={styles.row} role="row" onClick={() => onClick(t)}>
      <span className={styles.date}>{formatDate(t.date)}</span>

      <div>
        <div className={styles.payee}>
          {t.payee || '—'}
          {t.isSplit && <span className={styles.splitBadge}>Split</span>}
        </div>
        {t.description && <div className={styles.description}>{t.description}</div>}
        {t.tags && t.tags.length > 0 && (
          <div className={styles.tags}>
            {t.tags.map((tag) => (
              <TagBadge key={tag.id} tag={tag} />
            ))}
          </div>
        )}
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
