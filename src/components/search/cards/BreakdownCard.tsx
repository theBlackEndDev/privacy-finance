import { BreakdownCardData } from '../../../types';
import { formatCurrency } from '../../../utils/formatCurrency';
import styles from './Cards.module.css';

export function BreakdownCard({ data }: { data: BreakdownCardData }) {
  return (
    <div className={styles.card}>
      <div className={styles.cardLabel}>{data.label}</div>
      <div className={styles.breakdownList}>
        {data.items.map((item, i) => (
          <div key={i} className={styles.breakdownItem}>
            <div className={styles.breakdownHeader}>
              {item.color && <span className={styles.dot} style={{ background: item.color }} />}
              <span className={styles.breakdownName}>{item.name}</span>
              <span className={styles.breakdownAmount}>
                {formatCurrency(item.amount)}
              </span>
              <span className={styles.breakdownPct}>{item.percentage.toFixed(1)}%</span>
            </div>
            <div className={styles.progressTrack}>
              <div className={styles.progressBar} style={{ width: `${item.percentage}%`, background: item.color || 'var(--color-accent)' }} />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
