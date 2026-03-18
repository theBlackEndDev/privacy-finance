import { formatCurrency } from '../../utils/formatCurrency';
import { cx } from '../../utils/classNames';
import styles from './SummaryCards.module.css';

interface Props {
  income: number;
  expense: number;
  net: number;
}

export function SummaryCards({ income, expense, net }: Props) {
  return (
    <div className={styles.grid}>
      <SummaryCard label="Income" value={income} variant="income" />
      <SummaryCard label="Expenses" value={Math.abs(expense)} variant="expense" />
      <SummaryCard label="Net" value={net} variant={net >= 0 ? 'income' : 'expense'} />
    </div>
  );
}

type Variant = 'income' | 'expense';

function SummaryCard({ label, value, variant }: { label: string; value: number; variant: Variant }) {
  return (
    <div className={styles.card}>
      <div className={styles.label}>{label}</div>
      <div className={cx(styles.value, styles[variant])}>{formatCurrency(value)}</div>
    </div>
  );
}
