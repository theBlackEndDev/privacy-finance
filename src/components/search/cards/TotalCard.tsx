import { TotalCardData } from '../../../types';
import styles from './Cards.module.css';

export function TotalCard({ data }: { data: TotalCardData }) {
  const dollars = Math.abs(data.amount) / 100;
  return (
    <div className={styles.card}>
      <div className={styles.cardLabel}>{data.label} — {data.periodLabel}</div>
      <div className={styles.bigNumber}>${dollars.toLocaleString('en-US', { minimumFractionDigits: 2 })}</div>
      <div className={styles.meta}>{data.count} transactions</div>
      {data.changePct != null && (
        <div className={data.changePct > 0 ? styles.deltaUp : styles.deltaDown}>
          {data.changePct > 0 ? '+' : ''}{data.changePct.toFixed(1)}% vs previous period
        </div>
      )}
      {data.topItems.length > 0 && (
        <div className={styles.topItems}>
          <div className={styles.topLabel}>Top:</div>
          {data.topItems.slice(0, 3).map((item, i) => (
            <div key={i} className={styles.topItem}>
              <span>{item.name}</span>
              <span className={styles.topAmount}>${(item.amount / 100).toLocaleString('en-US', { minimumFractionDigits: 2 })}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
