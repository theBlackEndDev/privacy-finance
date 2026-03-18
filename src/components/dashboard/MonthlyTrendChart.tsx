import {
  BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid, Legend,
} from 'recharts';
import { MonthSummary } from '../../types';
import { formatCurrency } from '../../utils/formatCurrency';
import styles from './MonthlyTrendChart.module.css';

interface Props {
  data: MonthSummary[];
}

export function MonthlyTrendChart({ data }: Props) {
  if (data.length === 0) {
    return <div className={styles.empty}>No trend data available</div>;
  }

  const chartData = data.map(d => ({
    month: d.month.slice(5), // MM only for brevity
    income: d.income,
    expense: Math.abs(d.expense),
  }));

  const tickStyle = { fill: 'var(--text-muted)', fontSize: 11 };

  return (
    <ResponsiveContainer width="100%" height={240}>
      <BarChart data={chartData} barGap={4}>
        <CartesianGrid strokeDasharray="3 3" stroke="var(--border-color)" vertical={false} />
        <XAxis dataKey="month" tick={tickStyle} axisLine={false} tickLine={false} />
        <YAxis
          tickFormatter={(v) => `$${(v / 100).toFixed(0)}`}
          tick={tickStyle}
          axisLine={false}
          tickLine={false}
          width={56}
        />
        <Tooltip
          formatter={(value, name) => [formatCurrency(typeof value === 'number' ? value : 0), name === 'income' ? 'Income' : 'Expenses']}
          contentStyle={{ background: 'var(--bg-card)', border: '1px solid var(--border-color)', borderRadius: 6 }}
          labelStyle={{ color: 'var(--text-primary)' }}
          itemStyle={{ color: 'var(--text-secondary)' }}
        />
        <Legend
          formatter={(value) => <span style={{ color: 'var(--text-secondary)', fontSize: 12 }}>{value === 'income' ? 'Income' : 'Expenses'}</span>}
        />
        <Bar dataKey="income" fill="var(--color-income)" radius={[3, 3, 0, 0]} maxBarSize={32} />
        <Bar dataKey="expense" fill="var(--color-expense)" radius={[3, 3, 0, 0]} maxBarSize={32} />
      </BarChart>
    </ResponsiveContainer>
  );
}
