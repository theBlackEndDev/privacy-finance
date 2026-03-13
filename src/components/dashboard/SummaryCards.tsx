import { formatCurrency } from '../../utils/formatCurrency';

interface Props {
  income: number;
  expense: number;
  net: number;
}

export function SummaryCards({ income, expense, net }: Props) {
  return (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 16 }}>
      <Card label="Income" value={income} color="var(--color-income)" />
      <Card label="Expenses" value={Math.abs(expense)} color="var(--color-expense)" />
      <Card label="Net" value={net} color={net >= 0 ? 'var(--color-income)' : 'var(--color-expense)'} />
    </div>
  );
}

function Card({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div style={{
      background: 'var(--bg-card)',
      border: '1px solid var(--border-color)',
      borderRadius: 8,
      padding: '20px 24px',
    }}>
      <div style={{ fontSize: 12, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: 8 }}>
        {label}
      </div>
      <div style={{ fontSize: 28, fontFamily: 'var(--font-mono)', fontVariantNumeric: 'tabular-nums', color, fontWeight: 600 }}>
        {formatCurrency(value)}
      </div>
    </div>
  );
}
