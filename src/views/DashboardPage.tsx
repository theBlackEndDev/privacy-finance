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

function MonthSelector({ month, onChange }: { month: string; onChange: (m: string) => void }) {
  return (
    <input
      type="month"
      value={month}
      onChange={e => onChange(e.target.value)}
      style={{
        background: 'var(--bg-input)',
        border: '1px solid var(--border-color)',
        borderRadius: 6,
        color: 'var(--text-primary)',
        padding: '6px 10px',
        fontSize: 13,
        fontFamily: 'var(--font-mono)',
      }}
    />
  );
}

function SectionTitle({ children }: { children: React.ReactNode }) {
  return (
    <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.05em', marginBottom: 12 }}>
      {children}
    </div>
  );
}

function Card({ children, style }: { children: React.ReactNode; style?: React.CSSProperties }) {
  return (
    <div style={{
      background: 'var(--bg-card)',
      border: '1px solid var(--border-color)',
      borderRadius: 8,
      padding: 20,
      ...style,
    }}>
      {children}
    </div>
  );
}

export function DashboardPage() {
  const { month, setMonth, data, trend, balances, loading, error } = useDashboard();
  const [recent, setRecent] = useState<Transaction[]>([]);

  useEffect(() => {
    transactionsApi.getAll({ dateFrom: `${month}-01`, dateTo: `${month}-31`, pageSize: 10, page: 1 })
      .then(p => setRecent(p.items))
      .catch(() => setRecent([]));
  }, [month]);

  if (loading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', paddingTop: 64 }}>
        <Spinner />
      </div>
    );
  }

  if (error) {
    return (
      <div style={{ color: 'var(--color-expense)', padding: 24 }}>
        Error loading dashboard: {error}
      </div>
    );
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      {/* Header row */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{ fontSize: 13, color: 'var(--text-muted)' }}>Monthly overview</div>
        <MonthSelector month={month} onChange={setMonth} />
      </div>

      {/* Summary cards */}
      {data && <SummaryCards income={data.income} expense={data.expense} net={data.net} />}

      {/* Middle row: pie + trend */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
        <Card>
          <SectionTitle>Spending by Category</SectionTitle>
          <SpendingPieChart data={data?.byCategory ?? []} />
        </Card>
        <Card>
          <SectionTitle>Monthly Trend</SectionTitle>
          <MonthlyTrendChart data={trend} />
        </Card>
      </div>

      {/* Bottom row: account balances + recent transactions */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
        <Card>
          <SectionTitle>Account Balances</SectionTitle>
          <AccountBalances balances={balances} />
        </Card>
        <Card>
          <SectionTitle>Recent Transactions</SectionTitle>
          <RecentTransactions transactions={recent} />
        </Card>
      </div>
    </div>
  );
}
