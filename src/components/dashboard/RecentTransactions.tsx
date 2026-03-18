import { Transaction } from '../../types';
import { formatCurrency } from '../../utils/formatCurrency';
import { formatDate } from '../../utils/formatDate';
import { cx } from '../../utils/classNames';
import styles from './RecentTransactions.module.css';

const AMOUNT_PREFIX: Record<Transaction['kind'], string> = {
  income: '+',
  expense: '-',
  transfer: '',
};

interface Props {
  transactions: Transaction[];
}

export function RecentTransactions({ transactions }: Props) {
  if (transactions.length === 0) {
    return <div className={styles.empty}>No transactions this month</div>;
  }

  return (
    <div>
      {transactions.map(t => (
        <div key={t.id} className={styles.row}>
          <div className={styles.date}>{formatDate(t.date)}</div>
          <div>
            <div className={styles.payee}>{t.payee || '\u2014'}</div>
            {t.categoryName && <div className={styles.category}>{t.categoryName}</div>}
          </div>
          <div className={cx(styles.amount, styles[t.kind])}>
            {AMOUNT_PREFIX[t.kind]}{formatCurrency(Math.abs(t.amount))}
          </div>
        </div>
      ))}
    </div>
  );
}
