import { invoke } from '@tauri-apps/api/core';
import { DashboardData, MonthSummary, AccountBalance } from '../types';

export const dashboardApi = {
  getDashboardData: (month: string) =>
    invoke<DashboardData>('get_dashboard_data', { month }),
  getMonthlyTrend: (months: number) =>
    invoke<MonthSummary[]>('get_monthly_trend', { months }),
  getAccountBalances: () =>
    invoke<AccountBalance[]>('get_account_balances'),
};
