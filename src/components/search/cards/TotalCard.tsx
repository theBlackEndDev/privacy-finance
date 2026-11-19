import { TotalCardData } from '../../../types';
import { formatCurrency } from '../../../utils/formatCurrency';
import styles from './Cards.module.css';

export function TotalCard({ data }: { data: TotalCardData }) {
  return (
    <div className={styles.card}>
      <div className={styles.cardLabel}>{data.label} — {data.periodLabel}</div>
      <div className={styles.bigNumber}>{formatCurrency(Math.abs(data.amount))}</div>
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
              <span className={styles.topAmount}>{formatCurrency(item.amount)}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
