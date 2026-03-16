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
      <div className={styles.row}>
        <div className={styles.category}>
          <span className={styles.dot} style={{ background: row.categoryColor }} />
          <span className={styles.name}>{row.categoryName}</span>
        </div>
        <input
          ref={inputRef}
          className={styles.budgetedInput}
          type="text"
          value={editValue}
          onChange={(e) => setEditValue(e.target.value)}
          onBlur={handleBlur}
          onKeyDown={(e) => { if (e.key === 'Enter') (e.target as HTMLInputElement).blur(); }}
        />
        <span className={styles.amount}>{formatCurrency(row.spent)}</span>
        <span className={cx(styles.available, availableClass)}>
          {formatCurrency(row.available)}
        </span>
        <input
          type="checkbox"
          className={styles.rolloverToggle}
          checked={row.rollover}
          onChange={() => onToggleRollover(row.itemId, !row.rollover)}
          title="Enable rollover"
        />
      </div>
      <div className={styles.progressCell}>
        <BudgetProgressBar spent={row.spent} budgeted={row.budgeted} />
      </div>
    </>
  );
}
