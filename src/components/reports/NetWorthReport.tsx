import { useState, useEffect } from 'react';
import {
  LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid, Legend,
} from 'recharts';
import { NetWorthPoint } from '../../types';
import { reportsApi } from '../../api/reports';
import { formatCurrency } from '../../utils/formatCurrency';
import { TICK_STYLE, TOOLTIP_STYLE, dollarTickFormatter } from './chartConfig';
import styles from './NetWorthReport.module.css';

interface Props {
  startDate: string;
  endDate: string;
  onData?: (data: unknown) => void;
}

const COLORS = ['#0d9488', '#10b981', '#0ea5e9', '#f59e0b', '#8b5cf6', '#ef4444', '#64748b'];

export function NetWorthReport({ startDate, endDate, onData }: Props) {
  const [data, setData] = useState<NetWorthPoint[]>([]);
  const [showAccounts, setShowAccounts] = useState(false);

  useEffect(() => {
    if (!startDate || !endDate) return;
    let cancelled = false;
    reportsApi.getNetWorth(startDate, endDate).then((d) => {
      if (cancelled) return;
      setData(d);
      onData?.(d);
    });
    return () => { cancelled = true; };
  }, [startDate, endDate]);

  if (data.length === 0) {
    return <div className={styles.empty}>No net worth data for selected period</div>;
  }

  // Get unique account names
  const accountNames = data.length > 0
    ? data[0].accounts.map((a) => a.accountName)
    : [];

  const chartData = data.map((d) => {
    const point: Record<string, unknown> = {
      month: d.month.slice(5),
      total: d.total,
    };
    d.accounts.forEach((a) => {
      point[a.accountName] = a.balance;
    });
    return point;
  });

  return (
    <div className={styles.wrapper}>
      <div className={styles.controls}>
        <label className={styles.toggle}>
          <input
            type="checkbox"
            checked={showAccounts}
            onChange={(e) => setShowAccounts(e.target.checked)}
          />
          Show per-account breakdown
        </label>
      </div>

      <ResponsiveContainer width="100%" height={300}>
        <LineChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" stroke="var(--border-color)" vertical={false} />
          <XAxis dataKey="month" tick={TICK_STYLE} axisLine={false} tickLine={false} />
          <YAxis
            tickFormatter={dollarTickFormatter}
            tick={TICK_STYLE}
            axisLine={false}
            tickLine={false}
            width={60}
          />
          <Tooltip
            formatter={(value, name) => [formatCurrency(typeof value === 'number' ? value : 0), name === 'total' ? 'Net Worth' : name]}
            {...TOOLTIP_STYLE}
          />
          <Legend formatter={(value) => <span style={{ color: 'var(--text-secondary)', fontSize: 12 }}>{value === 'total' ? 'Net Worth' : value}</span>} />
          <Line
            type="monotone"
            dataKey="total"
            stroke="var(--color-primary)"
            strokeWidth={2}
            dot={{ r: 3 }}
          />
          {showAccounts && accountNames.map((name, i) => (
            <Line
              key={name}
              type="monotone"
              dataKey={name}
              stroke={COLORS[i % COLORS.length]}
              strokeWidth={1}
              strokeDasharray="4 4"
              dot={false}
            />
          ))}
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
