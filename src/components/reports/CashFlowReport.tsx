import { useState, useEffect } from 'react';
import {
  AreaChart, Area, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid, ReferenceLine,
} from 'recharts';
import { CashFlowPoint } from '../../types';
import { reportsApi } from '../../api/reports';
import { formatCurrency } from '../../utils/formatCurrency';
import { useToast } from '../ui/Toast';
import { TICK_STYLE, TOOLTIP_STYLE, dollarTickFormatter } from './chartConfig';
import styles from './CashFlowReport.module.css';

interface Props {
  startDate: string;
  endDate: string;
  onData?: (data: unknown) => void;
}

export function CashFlowReport({ startDate, endDate, onData }: Props) {
  const toast = useToast();
  const [data, setData] = useState<CashFlowPoint[]>([]);

  useEffect(() => {
    if (!startDate || !endDate) return;
    let cancelled = false;
    reportsApi.getCashFlow(startDate, endDate).then((d) => {
      if (cancelled) return;
      setData(d);
      onData?.(d);
    }).catch((err) => {
      if (cancelled) return;
      toast(`Failed to load cash flow: ${err}`, 'error');
    });
    return () => { cancelled = true; };
  }, [startDate, endDate, onData, toast]);

  if (data.length === 0) {
    return <div className={styles.empty}>No cash flow data for selected period</div>;
  }

  const chartData = data.map((d) => ({
    date: d.date.slice(5),
    cumulative: d.cumulative,
    net: d.net,
  }));

  return (
    <div className={styles.wrapper}>
      <ResponsiveContainer width="100%" height={300}>
        <AreaChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" stroke="var(--border-color)" vertical={false} />
          <XAxis dataKey="date" tick={TICK_STYLE} axisLine={false} tickLine={false} />
          <YAxis
            tickFormatter={dollarTickFormatter}
            tick={TICK_STYLE}
            axisLine={false}
            tickLine={false}
            width={60}
          />
          <ReferenceLine y={0} stroke="var(--text-muted)" strokeDasharray="4 4" />
          <Tooltip
            formatter={(value, name) => [
              formatCurrency(typeof value === 'number' ? value : 0),
              name === 'cumulative' ? 'Cumulative' : 'Net',
            ]}
            {...TOOLTIP_STYLE}
          />
          <Area
            type="monotone"
            dataKey="cumulative"
            stroke="var(--color-primary)"
            fill="var(--color-primary)"
            fillOpacity={0.15}
            strokeWidth={2}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}
