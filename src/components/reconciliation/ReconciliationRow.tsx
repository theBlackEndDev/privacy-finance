import { ReconciliationItemDetail } from '../../types';
import { formatCurrency } from '../../utils/formatCurrency';
import { formatDate } from '../../utils/formatDate';
import { cx } from '../../utils/classNames';
import styles from './ReconciliationRow.module.css';

interface Props {
  item: ReconciliationItemDetail;
  onToggle: (transactionId: string) => void;
}

export function ReconciliationRow({ item, onToggle }: Props) {
  const isIncome = item.kind === 'income';

  return (
    <div className={styles.row}>
      <input
        type="checkbox"
        className={styles.checkbox}
        checked={item.isReconciled}
        onChange={() => onToggle(item.transactionId)}
      />
      <span className={styles.date}>{formatDate(item.date)}</span>
      <span className={styles.payee}>{item.payee || '(no payee)'}</span>
      <span className={styles.kind}>{item.kind}</span>
      <span className={cx(styles.amount, isIncome ? styles.income : styles.expense)}>
        {isIncome ? '' : '-'}{formatCurrency(item.amount)}
      </span>
    </div>
  );
}
