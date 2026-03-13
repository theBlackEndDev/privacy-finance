import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer, Legend } from 'recharts';
import { CategorySummary } from '../../types';
import { formatCurrency } from '../../utils/formatCurrency';

interface Props {
  data: CategorySummary[];
}

export function SpendingPieChart({ data }: Props) {
  if (data.length === 0) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: 200, color: 'var(--text-muted)', fontSize: 14 }}>
        No expense data this month
      </div>
    );
  }

  const chartData = data.map(d => ({
    name: d.categoryName,
    value: Math.abs(d.total),
    color: d.categoryColor,
  }));

  return (
    <ResponsiveContainer width="100%" height={260}>
      <PieChart>
        <Pie
          data={chartData}
          cx="50%"
          cy="50%"
          innerRadius={60}
          outerRadius={100}
          paddingAngle={2}
          dataKey="value"
        >
          {chartData.map((entry, i) => (
            <Cell key={i} fill={entry.color} />
          ))}
        </Pie>
        <Tooltip
          formatter={(value) => formatCurrency(typeof value === 'number' ? value : 0)}
          contentStyle={{ background: 'var(--bg-card)', border: '1px solid var(--border-color)', borderRadius: 6 }}
          labelStyle={{ color: 'var(--text-primary)' }}
          itemStyle={{ color: 'var(--text-secondary)' }}
        />
        <Legend
          iconType="circle"
          iconSize={8}
          formatter={(value) => <span style={{ color: 'var(--text-secondary)', fontSize: 12 }}>{value}</span>}
        />
      </PieChart>
    </ResponsiveContainer>
  );
}
