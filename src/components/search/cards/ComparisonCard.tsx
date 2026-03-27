import { ComparisonCardData } from '../../../types';
import styles from './Cards.module.css';

export function ComparisonCard({ data }: { data: ComparisonCardData }) {
  const fmt = (cents: number) => '$' + (Math.abs(cents) / 100).toLocaleString('en-US', { minimumFractionDigits: 2 });
  const isUp = data.changeAmount > 0;
  return (
    <div className={styles.card}>
      <div className={styles.cardLabel}>{data.label}</div>
      <div className={styles.comparisonRow}>
        <div className={styles.comparisonCol}>
          <div className={styles.comparisonLabel}>{data.currentLabel}</div>
          <div className={styles.comparisonAmount}>{fmt(data.currentAmount)}</div>
        </div>
        <div className={styles.comparisonVs}>vs</div>
        <div className={styles.comparisonCol}>
          <div className={styles.comparisonLabel}>{data.previousLabel}</div>
          <div className={styles.comparisonAmount}>{fmt(data.previousAmount)}</div>
        </div>
      </div>
      <div className={isUp ? styles.deltaUp : styles.deltaDown}>
        {isUp ? '+' : ''}{fmt(data.changeAmount)} ({isUp ? '+' : ''}{data.changePct.toFixed(1)}%)
      </div>
    </div>
  );
}
