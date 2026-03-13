import { useState, useEffect, useCallback } from 'react';
import { dashboardApi } from '../api/dashboard';
import { DashboardData, MonthSummary, AccountBalance } from '../types';

function currentMonth(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
}

export function useDashboard() {
  const [month, setMonth] = useState<string>(currentMonth);
  const [data, setData] = useState<DashboardData | null>(null);
  const [trend, setTrend] = useState<MonthSummary[]>([]);
  const [balances, setBalances] = useState<AccountBalance[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [d, t, b] = await Promise.all([
        dashboardApi.getDashboardData(month),
        dashboardApi.getMonthlyTrend(12),
        dashboardApi.getAccountBalances(),
      ]);
      setData(d);
      setTrend(t);
      setBalances(b);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [month]);

  useEffect(() => { load(); }, [load]);

  return { month, setMonth, data, trend, balances, loading, error, refresh: load };
}
