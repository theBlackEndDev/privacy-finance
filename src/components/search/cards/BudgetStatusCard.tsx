import { BudgetStatusCardData } from '../../../types';
import styles from './Cards.module.css';

export function BudgetStatusCard({ data }: { data: BudgetStatusCardData }) {
  return (
    <div className={styles.card}>
      <div className={styles.cardLabel}>Budget Status — {data.periodLabel}</div>
      {data.items.length === 0 ? (
        <div className={styles.meta}>No budget data for this period</div>
      ) : (
        <div className={styles.budgetList}>
          {data.items.map((item, i) => {
            const pct = item.budgeted > 0 ? (item.spent / item.budgeted) * 100 : 0;
            const over = item.spent > item.budgeted;
            return (
              <div key={i} className={styles.budgetItem}>
                <div className={styles.budgetHeader}>
                  <span style={{ color: item.color }}>{item.categoryName}</span>
                  <span className={over ? styles.overBudget : styles.meta}>
                    ${(item.spent / 100).toFixed(0)} / ${(item.budgeted / 100).toFixed(0)}
                  </span>
                </div>
                <div className={styles.progressTrack}>
                  <div
                    className={over ? styles.progressBarOver : styles.progressBar}
                    style={{ width: `${Math.min(pct, 100)}%` }}
                  />
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
