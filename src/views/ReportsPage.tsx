import { useState, useCallback } from 'react';
import { ReportDateRange } from '../components/reports/ReportDateRange';
import { IncomeExpenseReport } from '../components/reports/IncomeExpenseReport';
import { CategorySpendingReport } from '../components/reports/CategorySpendingReport';
import { CashFlowReport } from '../components/reports/CashFlowReport';
import { NetWorthReport } from '../components/reports/NetWorthReport';
import { TagSpendingReport } from '../components/reports/TagSpendingReport';
import { PayeeSpendingReport } from '../components/reports/PayeeSpendingReport';
import { ReportExportButton } from '../components/reports/ReportExportButton';
import { formatDateISO } from '../utils/formatDate';
import styles from './ReportsPage.module.css';

type Tab = 'income-expense' | 'categories' | 'cash-flow' | 'net-worth' | 'tags' | 'payees';

const TABS: { key: Tab; label: string }[] = [
  { key: 'income-expense', label: 'Income/Expense' },
  { key: 'categories', label: 'Categories' },
  { key: 'cash-flow', label: 'Cash Flow' },
  { key: 'net-worth', label: 'Net Worth' },
  { key: 'tags', label: 'Tags' },
  { key: 'payees', label: 'Payees' },
];

function getDefaultDates(): [string, string] {
  const now = new Date();
  const start = new Date(now.getFullYear(), now.getMonth(), 1);
  const end = new Date(now.getFullYear(), now.getMonth() + 1, 0);
  return [formatDateISO(start), formatDateISO(end)];
}

export function ReportsPage() {
  const [defaults] = useState(getDefaultDates);
  const [startDate, setStartDate] = useState(defaults[0]);
  const [endDate, setEndDate] = useState(defaults[1]);
  const [activeTab, setActiveTab] = useState<Tab>('income-expense');
  const [exportData, setExportData] = useState<unknown>(null);

  const handleData = useCallback((data: unknown) => {
    setExportData(data);
  }, []);

  function handleDateChange(start: string, end: string) {
    setStartDate(start);
    setEndDate(end);
    setExportData(null);
  }

  function handleTabChange(tab: Tab) {
    setActiveTab(tab);
    setExportData(null);
  }

  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <h2 className={styles.title}>Reports</h2>
        <div className={styles.exportBtns}>
          {exportData !== null && (
            <>
              <ReportExportButton data={exportData} filename={`report-${activeTab}`} format="csv" />
              <ReportExportButton data={exportData} filename={`report-${activeTab}`} format="json" />
            </>
          )}
        </div>
      </div>

      <ReportDateRange startDate={startDate} endDate={endDate} onChange={handleDateChange} />

      <div className={styles.tabs}>
        {TABS.map(({ key, label }) => (
          <button
            key={key}
            className={`${styles.tab} ${activeTab === key ? styles.tabActive : ''}`}
            onClick={() => handleTabChange(key)}
          >
            {label}
          </button>
        ))}
      </div>

      <div className={styles.content}>
        {activeTab === 'income-expense' && <IncomeExpenseReport startDate={startDate} endDate={endDate} onData={handleData} />}
        {activeTab === 'categories' && <CategorySpendingReport startDate={startDate} endDate={endDate} onData={handleData} />}
        {activeTab === 'cash-flow' && <CashFlowReport startDate={startDate} endDate={endDate} onData={handleData} />}
        {activeTab === 'net-worth' && <NetWorthReport startDate={startDate} endDate={endDate} onData={handleData} />}
        {activeTab === 'tags' && <TagSpendingReport startDate={startDate} endDate={endDate} onData={handleData} />}
        {activeTab === 'payees' && <PayeeSpendingReport startDate={startDate} endDate={endDate} onData={handleData} />}
      </div>
    </div>
  );
}
