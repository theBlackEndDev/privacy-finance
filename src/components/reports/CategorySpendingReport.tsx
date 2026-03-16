import { useState, useEffect } from 'react';
import {
  BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, Cell,
} from 'recharts';
import { CategorySpendingRow } from '../../types';
import { reportsApi } from '../../api/reports';
import { formatCurrency } from '../../utils/formatCurrency';
import { TICK_STYLE, TOOLTIP_STYLE, dollarTickFormatter } from './chartConfig';
import styles from './CategorySpendingReport.module.css';

interface Props {
  startDate: string;
  endDate: string;
  onData?: (data: unknown) => void;
}

export function CategorySpendingReport({ startDate, endDate, onData }: Props) {
  const [data, setData] = useState<CategorySpendingRow[]>([]);

  useEffect(() => {
    if (!startDate || !endDate) return;
    let cancelled = false;
    reportsApi.getCategorySpending(startDate, endDate).then((d) => {
      if (cancelled) return;
      setData(d);
      onData?.(d);
    });
    return () => { cancelled = true; };
  }, [startDate, endDate]);

  if (data.length === 0) {
    return <div className={styles.empty}>No spending data for selected period</div>;
  }

  const chartData = data.map((d) => ({
    name: d.categoryName,
    total: d.total,
    color: d.categoryColor,
    percentage: d.percentage,
  }));

  return (
    <div className={styles.wrapper}>
      <ResponsiveContainer width="100%" height={Math.max(200, data.length * 40 + 40)}>
        <BarChart data={chartData} layout="vertical" barSize={20}>
          <XAxis
            type="number"
            tickFormatter={dollarTickFormatter}
            tick={TICK_STYLE}
            axisLine={false}
            tickLine={false}
          />
          <YAxis
            type="category"
            dataKey="name"
            tick={TICK_STYLE}
            axisLine={false}
            tickLine={false}
            width={120}
          />
          <Tooltip
            formatter={(value) => [formatCurrency(typeof value === 'number' ? value : 0), 'Spending']}
            {...TOOLTIP_STYLE}
          />
          <Bar dataKey="total" radius={[0, 4, 4, 0]}>
            {chartData.map((entry, i) => (
              <Cell key={i} fill={entry.color} />
            ))}
          </Bar>
        </BarChart>
      </ResponsiveContainer>

      <div className={styles.table}>
        {data.map((row) => (
          <div key={row.categoryId ?? 'uncategorized'} className={styles.row}>
            <span className={styles.dot} style={{ background: row.categoryColor }} />
            <span className={styles.catName}>{row.categoryName}</span>
            <span className={styles.pct}>{row.percentage.toFixed(1)}%</span>
            <span className={styles.amount}>{formatCurrency(row.total)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
