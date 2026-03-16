import { useState, useEffect } from 'react';
import {
  BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid, Legend,
} from 'recharts';
import { IncomeExpenseReport as IEReport } from '../../types';
import { reportsApi } from '../../api/reports';
import { formatCurrency } from '../../utils/formatCurrency';
import { TICK_STYLE, TOOLTIP_STYLE, dollarTickFormatter } from './chartConfig';
import styles from './IncomeExpenseReport.module.css';

interface Props {
  startDate: string;
  endDate: string;
  onData?: (data: unknown) => void;
}

export function IncomeExpenseReport({ startDate, endDate, onData }: Props) {
  const [data, setData] = useState<IEReport | null>(null);

  useEffect(() => {
    if (!startDate || !endDate) return;
    let cancelled = false;
    reportsApi.getIncomeExpense(startDate, endDate).then((d) => {
      if (cancelled) return;
      setData(d);
      onData?.(d.months);
    });
    return () => { cancelled = true; };
  }, [startDate, endDate]);

  if (!data || data.months.length === 0) {
    return <div className={styles.empty}>No data for selected period</div>;
  }

  const chartData = data.months.map((m) => ({
    month: m.month.slice(5),
    income: m.income,
    expense: m.expense,
  }));

  const totalIncome = data.months.reduce((s, m) => s + m.income, 0);
  const totalExpense = data.months.reduce((s, m) => s + m.expense, 0);
  const totalNet = totalIncome - totalExpense;

  const incomeDelta = data.previousIncome > 0
    ? ((totalIncome - data.previousIncome) / data.previousIncome * 100).toFixed(1)
    : null;
  const expenseDelta = data.previousExpense > 0
    ? ((totalExpense - data.previousExpense) / data.previousExpense * 100).toFixed(1)
    : null;

  return (
    <div className={styles.wrapper}>
      <div className={styles.summary}>
        <div className={styles.summaryCard}>
          <div className={styles.summaryLabel}>Total Income</div>
          <div className={styles.summaryValue} style={{ color: 'var(--color-income)' }}>
            {formatCurrency(totalIncome)}
          </div>
          {incomeDelta && (
            <div className={styles.delta}>
              {Number(incomeDelta) >= 0 ? '+' : ''}{incomeDelta}% vs prev period
            </div>
          )}
        </div>
        <div className={styles.summaryCard}>
          <div className={styles.summaryLabel}>Total Expenses</div>
          <div className={styles.summaryValue} style={{ color: 'var(--color-expense)' }}>
            {formatCurrency(totalExpense)}
          </div>
          {expenseDelta && (
            <div className={styles.delta}>
              {Number(expenseDelta) >= 0 ? '+' : ''}{expenseDelta}% vs prev period
            </div>
          )}
        </div>
        <div className={styles.summaryCard}>
          <div className={styles.summaryLabel}>Net</div>
          <div className={styles.summaryValue} style={{ color: totalNet >= 0 ? 'var(--color-income)' : 'var(--color-expense)' }}>
            {formatCurrency(totalNet)}
          </div>
        </div>
      </div>

      <ResponsiveContainer width="100%" height={300}>
        <BarChart data={chartData} barGap={4}>
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
            formatter={(value, name) => [formatCurrency(typeof value === 'number' ? value : 0), name === 'income' ? 'Income' : 'Expenses']}
            {...TOOLTIP_STYLE}
          />
          <Legend
            formatter={(value) => <span style={{ color: 'var(--text-secondary)', fontSize: 12 }}>{value === 'income' ? 'Income' : 'Expenses'}</span>}
          />
          <Bar dataKey="income" fill="var(--color-income)" radius={[3, 3, 0, 0]} maxBarSize={32} />
          <Bar dataKey="expense" fill="var(--color-expense)" radius={[3, 3, 0, 0]} maxBarSize={32} />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
