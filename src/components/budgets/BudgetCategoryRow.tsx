import { useState, useCallback, useEffect, useRef } from 'react';
import { BudgetCategoryRow as RowData } from '../../types';
import { formatCurrency, parseCurrency } from '../../utils/formatCurrency';
import { cx } from '../../utils/classNames';
import { BudgetProgressBar } from './BudgetProgressBar';
import styles from './BudgetCategoryRow.module.css';

interface Props {
  row: RowData;
  onUpdateBudgeted: (itemId: string, cents: number) => void;
  onToggleRollover: (itemId: string, rollover: boolean) => void;
}

export function BudgetCategoryRow({ row, onUpdateBudgeted, onToggleRollover }: Props) {
  const [editValue, setEditValue] = useState(() =>
    (row.budgeted / 100).toFixed(2)
  );
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (document.activeElement !== inputRef.current) {
      setEditValue((row.budgeted / 100).toFixed(2));
    }
  }, [row.budgeted]);

  const handleBlur = useCallback(() => {
    const cents = parseCurrency(editValue);
    if (cents !== row.budgeted) {
      onUpdateBudgeted(row.itemId, cents);
    }
  }, [editValue, row.budgeted, row.itemId, onUpdateBudgeted]);

  const availableClass = row.available >= 0 ? styles.positive : styles.negative;

  return (
    <>
      <div className={styles.row} role="row">
        <div className={styles.category} role="cell">
          <span className={styles.dot} style={{ background: row.categoryColor }} />
          <span className={styles.name}>{row.categoryName}</span>
        </div>
        <input
          ref={inputRef}
          className={styles.budgetedInput}
          type="text"
          role="cell"
          value={editValue}
          onChange={(e) => setEditValue(e.target.value)}
          onBlur={handleBlur}
          onKeyDown={(e) => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }}
          aria-label={`Budgeted amount for ${row.categoryName}`}
        />
        <span className={styles.amount} role="cell">{formatCurrency(row.spent)}</span>
        <span className={cx(styles.available, availableClass)} role="cell">
          {formatCurrency(row.available)}
        </span>
        <input
          type="checkbox"
          className={styles.rolloverToggle}
          checked={row.rollover}
          onChange={() => onToggleRollover(row.itemId, !row.rollover)}
          title="Enable rollover"
          role="cell"
          aria-label={`Rollover for ${row.categoryName}`}
        />
      </div>
      <div className={styles.progressCell}>
        <BudgetProgressBar spent={row.spent} budgeted={row.budgeted} />
      </div>
    </>
  );
}
