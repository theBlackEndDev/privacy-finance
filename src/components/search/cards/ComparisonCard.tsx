import { ComparisonCardData } from '../../../types';
import { formatCurrency } from '../../../utils/formatCurrency';
import styles from './Cards.module.css';

export function ComparisonCard({ data }: { data: ComparisonCardData }) {
  const isUp = data.changeAmount > 0;
  return (
    <div className={styles.card}>
      <div className={styles.cardLabel}>{data.label}</div>
      <div className={styles.comparisonRow}>
        <div className={styles.comparisonCol}>
          <div className={styles.comparisonLabel}>{data.currentLabel}</div>
          <div className={styles.comparisonAmount}>{formatCurrency(data.currentAmount)}</div>
        </div>
        <div className={styles.comparisonVs}>vs</div>
        <div className={styles.comparisonCol}>
          <div className={styles.comparisonLabel}>{data.previousLabel}</div>
          <div className={styles.comparisonAmount}>{formatCurrency(data.previousAmount)}</div>
        </div>
      </div>
      <div className={isUp ? styles.deltaUp : styles.deltaDown}>
        {isUp ? '+' : ''}{formatCurrency(data.changeAmount)} ({isUp ? '+' : ''}{data.changePct.toFixed(1)}%)
      </div>
    </div>
  );
}
