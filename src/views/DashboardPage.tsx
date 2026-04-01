import { useState, useEffect } from 'react';
import { useDashboard } from '../hooks/useDashboard';
import { transactionsApi } from '../api/transactions';
import { Transaction } from '../types';
import { SummaryCards } from '../components/dashboard/SummaryCards';
import { SpendingPieChart } from '../components/dashboard/SpendingPieChart';
import { MonthlyTrendChart } from '../components/dashboard/MonthlyTrendChart';
import { AccountBalances } from '../components/dashboard/AccountBalances';
import { RecentTransactions } from '../components/dashboard/RecentTransactions';
import { Spinner } from '../components/ui/Spinner';
import { cx } from '../utils/classNames';
import styles from './DashboardPage.module.css';

function MonthSelector({ month, onChange }: { month: string; onChange: (m: string) => void }) {
  return (
    <input
      type="month"
      value={month}
      onChange={e => onChange(e.target.value)}
      className={styles.monthInput}
    />
  );
}

function SectionTitle({ children }: { children: React.ReactNode }) {
  return <div className={styles.sectionTitle}>{children}</div>;
}

function Card({ children, className, testId }: { children: React.ReactNode; className?: string; testId?: string }) {
  return (
    <div className={cx(styles.card, className)} data-testid={testId}>
      {children}
    </div>
  );
}

function lastDayOfMonth(month: string): string {
  const [year, mon] = month.split('-').map(Number);
  const last = new Date(year, mon, 0).getDate();
  return `${month}-${String(last).padStart(2, '0')}`;
}

export function DashboardPage() {
  const { month, setMonth, data, trend, balances, loading, error } = useDashboard();
  const [recent, setRecent] = useState<Transaction[]>([]);

  useEffect(() => {
    let cancelled = false;
    transactionsApi.getAll({ dateFrom: `${month}-01`, dateTo: lastDayOfMonth(month), pageSize: 10, page: 1 })
      .then(p => { if (!cancelled) setRecent(p.items); })
      .catch(() => { if (!cancelled) setRecent([]); });
    return () => { cancelled = true; };
  }, [month]);

  if (loading) {
    return (
      <div className={styles.centered}>
        <Spinner />
      </div>
    );
  }

  if (error) {
    return <div className={styles.error}>Error loading dashboard: {error}</div>;
  }

  return (
    <div className={styles.page} data-testid="dashboard-summary">

      {/* Header row */}
      <div className={styles.headerRow}>
        <div className={styles.headerLabel}>Monthly overview</div>
        <MonthSelector month={month} onChange={setMonth} />
      </div>

      {/* Summary cards */}
      {data && <SummaryCards income={data.income} expense={data.expense} net={data.net} />}

      {/* Middle row: pie + trend */}
      <div className={styles.gridTwo}>
        <Card>
          <SectionTitle>Spending by Category</SectionTitle>
          <SpendingPieChart data={data?.byCategory ?? []} />
        </Card>
        <Card testId="dashboard-chart">
          <SectionTitle>Monthly Trend</SectionTitle>
          <MonthlyTrendChart data={trend} />
        </Card>
      </div>

      {/* Bottom row: account balances + recent transactions */}
      <div className={styles.gridTwo}>
        <Card>
          <SectionTitle>Account Balances</SectionTitle>
          <AccountBalances balances={balances} />
        </Card>
        <Card testId="dashboard-recent-tx">
          <SectionTitle>Recent Transactions</SectionTitle>
          <RecentTransactions transactions={recent} />
        </Card>
      </div>
    </div>
  );
}
