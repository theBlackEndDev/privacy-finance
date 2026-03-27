import { TransactionListCardData } from '../../../types';
import styles from './Cards.module.css';

export function TransactionListCard({ data }: { data: TransactionListCardData }) {
  const fmt = (cents: number) => {
    const sign = cents < 0 ? '-' : cents > 0 ? '+' : '';
    return sign + '$' + (Math.abs(cents) / 100).toLocaleString('en-US', { minimumFractionDigits: 2 });
  };
  return (
    <div className={styles.card}>
      <div className={styles.cardLabel}>{data.label}</div>
      <div className={styles.txList}>
        {data.transactions.slice(0, 10).map((tx) => (
          <div key={tx.id} className={styles.txRow}>
            <span className={styles.txDate}>{tx.date}</span>
            <span className={styles.txPayee}>{tx.payee || 'Unknown'}</span>
            {tx.categoryName && <span className={styles.txCategory}>{tx.categoryName}</span>}
            <span className={tx.kind === 'income' ? styles.txAmountIncome : styles.txAmount}>
              {fmt(tx.amount)}
            </span>
          </div>
        ))}
      </div>
      <div className={styles.txFooter}>
        {data.count} transactions — Total: ${(Math.abs(data.total) / 100).toLocaleString('en-US', { minimumFractionDigits: 2 })}
      </div>
    </div>
  );
}
