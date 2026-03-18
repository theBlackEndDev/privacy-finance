import { useState, useEffect, useCallback, useRef } from 'react';
import { BudgetCategoryRow, Transaction } from '../../types';
import { budgetApi } from '../../api/budgets';
import { formatCurrency, parseCurrency } from '../../utils/formatCurrency';
import { formatShortDate } from '../../utils/formatDate';
import { BudgetProgressBar } from './BudgetProgressBar';
import styles from './BudgetDetailPanel.module.css';

interface Props {
  row: BudgetCategoryRow;
  year: number;
  month: number;
  onClose: () => void;
  onUpdateTarget: (itemId: string, targetAmount: number | null) => void;
}

export function BudgetDetailPanel({ row, year, month, onClose, onUpdateTarget }: Props) {
  const [activity, setActivity] = useState<Transaction[]>([]);
  const [loadingActivity, setLoadingActivity] = useState(true);
  const [targetValue, setTargetValue] = useState(() =>
    row.targetAmount != null ? (row.targetAmount / 100).toFixed(2) : ''
  );
  const targetRef = useRef<HTMLInputElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setLoadingActivity(true);
    budgetApi.getCategoryActivity(row.categoryId, year, month)
      .then(setActivity)
      .catch(() => setActivity([]))
      .finally(() => setLoadingActivity(false));
  }, [row.categoryId, year, month]);

  useEffect(() => {
    if (document.activeElement !== targetRef.current) {
      setTargetValue(row.targetAmount != null ? (row.targetAmount / 100).toFixed(2) : '');
    }
  }, [row.targetAmount]);

  const handleTargetBlur = useCallback(() => {
    const trimmed = targetValue.trim();
    if (trimmed === '') {
      if (row.targetAmount != null) {
        onUpdateTarget(row.itemId, null);
      }
      return;
    }
    const cents = parseCurrency(trimmed);
    if (cents !== row.targetAmount) {
      onUpdateTarget(row.itemId, cents);
    }
  }, [targetValue, row.targetAmount, row.itemId, onUpdateTarget]);

  const handleOverlayClick = useCallback((e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }, [onClose]);

  return (
    <>
      <div className={styles.overlay} onClick={handleOverlayClick} />
      <div className={styles.panel} ref={panelRef} role="dialog" aria-label={`${row.categoryName} details`}>
        <div className={styles.header}>
          <span className={styles.dot} style={{ background: row.categoryColor }} />
          <div className={styles.headerInfo}>
            <h3 className={styles.categoryName}>{row.categoryName}</h3>
            <div className={styles.summaryRow}>
              <span>Budgeted: {formatCurrency(row.budgeted)}</span>
              <span>Spent: {formatCurrency(row.spent)}</span>
              <span>Available: {formatCurrency(row.available)}</span>
            </div>
          </div>
          <button className={styles.closeBtn} onClick={onClose} aria-label="Close panel">&times;</button>
        </div>

        <div className={styles.body}>
          <div className={styles.section}>
            <h4 className={styles.sectionTitle}>Monthly Target</h4>
            <div className={styles.targetRow}>
              <span className={styles.targetLabel}>Target:</span>
              <input
                ref={targetRef}
                className={styles.targetInput}
                type="text"
                placeholder="0.00"
                value={targetValue}
                onChange={(e) => setTargetValue(e.target.value)}
                onBlur={handleTargetBlur}
                onKeyDown={(e) => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }}
                aria-label={`Monthly target for ${row.categoryName}`}
              />
            </div>
            {row.targetAmount != null && row.targetAmount > 0 && (
              <div className={styles.targetProgress}>
                <BudgetProgressBar spent={row.spent} budgeted={row.targetAmount} />
              </div>
            )}
          </div>

          <div className={styles.section}>
            <h4 className={styles.sectionTitle}>
              Activity ({activity.length})
            </h4>
            {loadingActivity ? (
              <div className={styles.emptyActivity}>Loading...</div>
            ) : activity.length === 0 ? (
              <div className={styles.emptyActivity}>No transactions this month</div>
            ) : (
              <ul className={styles.activityList}>
                {activity.map((txn) => (
                  <li key={txn.id} className={styles.activityItem}>
                    <span className={styles.activityDate}>{formatShortDate(txn.date)}</span>
                    <span className={styles.activityPayee}>{txn.payee || txn.description || '—'}</span>
                    <span className={styles.activityAmount}>{formatCurrency(txn.amount)}</span>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>
      </div>
    </>
  );
}
