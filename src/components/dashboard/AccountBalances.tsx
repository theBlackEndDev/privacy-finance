import { AccountBalance } from '../../types';
import { formatCurrency } from '../../utils/formatCurrency';
import { ACCOUNT_TYPE_LABELS } from '../../utils/accountTypes';
import { cx } from '../../utils/classNames';
import styles from './AccountBalances.module.css';

interface Props {
  balances: AccountBalance[];
}

export function AccountBalances({ balances }: Props) {
  const total = balances.reduce((sum, b) => sum + b.balance, 0);

  return (
    <div>
      {balances.map(b => (
        <div key={b.accountId} className={styles.row}>
          <div>
            <div className={styles.name}>{b.accountName}</div>
            <div className={styles.type}>
              {ACCOUNT_TYPE_LABELS[b.accountType as keyof typeof ACCOUNT_TYPE_LABELS] ?? b.accountType}
            </div>
          </div>
          <div className={cx(styles.balance, b.balance < 0 && styles.negative)}>
            {formatCurrency(b.balance)}
          </div>
        </div>
      ))}
      {balances.length > 1 && (
        <div className={styles.totalRow}>
          <div className={styles.totalLabel}>Net Worth</div>
          <div className={cx(styles.totalValue, total < 0 ? styles.negative : styles.positive)}>
            {formatCurrency(total)}
          </div>
        </div>
      )}
    </div>
  );
}
