import { cx } from '../../utils/classNames';
import styles from './BudgetProgressBar.module.css';

interface Props {
  spent: number;
  budgeted: number;
}

export function BudgetProgressBar({ spent, budgeted }: Props) {
  if (budgeted <= 0) {
    return <div className={styles.bar}><div className={styles.fill} /></div>;
  }

  const ratio = spent / budgeted;
  const pct = Math.min(ratio * 100, 100);

  const colorClass =
    ratio > 1 ? styles.red :
    ratio >= 0.8 ? styles.yellow :
    styles.green;

  return (
    <div className={styles.bar}>
      <div
        className={cx(styles.fill, colorClass)}
        style={{ width: `${pct}%` }}
      />
    </div>
  );
}
