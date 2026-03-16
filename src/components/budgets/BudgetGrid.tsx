import { BudgetCategoryRow as RowData, BudgetSummary } from '../../types';
import { formatCurrency } from '../../utils/formatCurrency';
import { cx } from '../../utils/classNames';
import { BudgetCategoryRow } from './BudgetCategoryRow';
import styles from './BudgetGrid.module.css';

interface Props {
  items: RowData[];
  summary: BudgetSummary | null;
  onUpdateBudgeted: (itemId: string, cents: number) => void;
  onToggleRollover: (itemId: string, rollover: boolean) => void;
}

export function BudgetGrid({ items, summary, onUpdateBudgeted, onToggleRollover }: Props) {
  const toBeBudgeted = summary?.toBeBudgeted ?? 0;
  const tbbClass = toBeBudgeted > 0 ? styles.positive : toBeBudgeted < 0 ? styles.negative : styles.zero;

  return (
    <div className={styles.container}>
      {/* To Be Budgeted banner */}
      <div className={styles.toBeBudgeted}>
        <span className={styles.toBeBudgetedLabel}>To Be Budgeted</span>
        <span className={cx(styles.toBeBudgetedAmount, tbbClass)}>
          {formatCurrency(toBeBudgeted)}
        </span>
      </div>

      {/* Header */}
      <div className={styles.header}>
        <span className={styles.headerCell}>Category</span>
        <span className={styles.headerRight}>Budgeted</span>
        <span className={styles.headerRight}>Spent</span>
        <span className={styles.headerRight}>Available</span>
        <span className={styles.headerCenter} title="Rollover">↻</span>
      </div>

      {/* Rows */}
      {items.length === 0 ? (
        <div className={styles.empty}>No budget items. Create a budget to get started.</div>
      ) : (
        items.map((row) => (
          <BudgetCategoryRow
            key={row.itemId}
            row={row}
            onUpdateBudgeted={onUpdateBudgeted}
            onToggleRollover={onToggleRollover}
          />
        ))
      )}

      {/* Summary row */}
      {summary && (
        <div className={styles.summaryRow}>
          <span className={styles.summaryLabel}>Totals</span>
          <span className={styles.summaryAmount}>{formatCurrency(summary.totalBudgeted)}</span>
          <span className={styles.summaryAmount}>{formatCurrency(summary.totalSpent)}</span>
          <span className={styles.summaryAmount}>{formatCurrency(summary.totalAvailable)}</span>
          <span />
        </div>
      )}
    </div>
  );
}
