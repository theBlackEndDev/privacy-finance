import { invoke } from './invoke';
import { DashboardData, MonthSummary, AccountBalance } from '../types';

const emptyDashboard: DashboardData = { income: 0, expense: 0, net: 0, byCategory: [] };

export const dashboardApi = {
  getDashboardData: (month: string) =>
    invoke<DashboardData>('get_dashboard_data', { month }, emptyDashboard),
  getMonthlyTrend: (months: number) =>
    invoke<MonthSummary[]>('get_monthly_trend', { months }, []),
  getAccountBalances: () =>
    invoke<AccountBalance[]>('get_account_balances', {}, []),
};
