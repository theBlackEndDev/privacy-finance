import { AccountSummaryCardData } from '../../../types';
import styles from './Cards.module.css';

export function AccountSummaryCard({ data }: { data: AccountSummaryCardData }) {
  const fmt = (cents: number) => '$' + (cents / 100).toLocaleString('en-US', { minimumFractionDigits: 2 });
  return (
    <div className={styles.card}>
      <div className={styles.cardLabel}>Account Balances</div>
      <div className={styles.accountList}>
        {data.accounts.map((acct, i) => (
          <div key={i} className={styles.accountRow}>
            <span className={styles.accountName}>{acct.name}</span>
            <span className={styles.accountType}>{acct.accountType}</span>
            <span className={acct.balance >= 0 ? styles.accountBalance : styles.accountBalanceNeg}>
              {fmt(acct.balance)}
            </span>
          </div>
        ))}
      </div>
      <div className={styles.netWorth}>
        <span>Net Worth</span>
        <span className={styles.bigNumber}>{fmt(data.netWorth)}</span>
      </div>
    </div>
  );
}
