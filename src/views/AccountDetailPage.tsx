import { useState, useEffect, useCallback, useMemo } from 'react';
import { Account, Transaction, TransactionPage, AccountBalance } from '../types';
import { accountsApi } from '../api/accounts';
import { transactionsApi } from '../api/transactions';
import { dashboardApi } from '../api/dashboard';
import { useApp } from '../context/AppContext';
import { useCategories } from '../context/CategoryContext';
import { formatCurrency } from '../utils/formatCurrency';
import { ACCOUNT_TYPE_LABELS } from '../utils/accountTypes';
import { Spinner } from '../components/ui/Spinner';
import { TransactionList } from '../components/transactions/TransactionList';
import { TransactionEditModal } from '../components/transactions/TransactionEditModal';
import { useToast } from '../components/ui/Toast';
import styles from './AccountDetailPage.module.css';

export function AccountDetailPage() {
  const { activeAccountId, navigate } = useApp();
  const { categories } = useCategories();
  const toast = useToast();

  const [account, setAccount] = useState<Account | null>(null);
  const [balance, setBalance] = useState<number | null>(null);
  const [page, setPage] = useState<TransactionPage | null>(null);
  const [currentPage, setCurrentPage] = useState(0);
  const [loadingAccount, setLoadingAccount] = useState(true);
  const [loadingTxns, setLoadingTxns] = useState(true);
  const [editing, setEditing] = useState<Transaction | null>(null);

  // Redirect if no account selected
  if (!activeAccountId) {
    navigate('accounts');
    return null;
  }

  // Load account + balance (only on mount or account change)
  useEffect(() => {
    let cancelled = false;
    setLoadingAccount(true);
    Promise.all([
      accountsApi.getOne(activeAccountId),
      dashboardApi.getAccountBalances(),
    ])
      .then(([acct, balances]) => {
        if (cancelled) return;
        setAccount(acct);
        const acctBalance = balances.find((b: AccountBalance) => b.accountId === activeAccountId);
        setBalance(acctBalance?.balance ?? acct.openingBalance);
      })
      .catch(() => {
        if (!cancelled) navigate('accounts');
      })
      .finally(() => {
        if (!cancelled) setLoadingAccount(false);
      });
    return () => { cancelled = true; };
  }, [activeAccountId]); // eslint-disable-line react-hooks/exhaustive-deps

  // Load transactions (on mount, page change, or after edit)
  const loadTransactions = useCallback((pageNum: number) => {
    setLoadingTxns(true);
    transactionsApi
      .getAll({ accountId: activeAccountId, page: pageNum, pageSize: 50 })
      .then(setPage)
      .catch(err => toast(`Failed to load transactions: ${err}`, 'error'))
      .finally(() => setLoadingTxns(false));
  }, [activeAccountId, toast]);

  useEffect(() => {
    loadTransactions(currentPage);
  }, [currentPage, loadTransactions]);

  function handleBack() {
    navigate('accounts');
  }

  function handleEditSaved() {
    setEditing(null);
    loadTransactions(currentPage);
    // Refresh balance after edit
    dashboardApi.getAccountBalances().then(balances => {
      const acctBalance = balances.find((b: AccountBalance) => b.accountId === activeAccountId);
      if (acctBalance) setBalance(acctBalance.balance);
    });
  }

  // Compute income/expense/net from current page
  const { totalIncome, totalExpense, net } = useMemo(() => {
    const items = page?.items ?? [];
    let inc = 0;
    let exp = 0;
    for (const t of items) {
      if (t.kind === 'income') inc += t.amount;
      else if (t.kind === 'expense') exp += t.amount;
    }
    return { totalIncome: inc, totalExpense: exp, net: inc - exp };
  }, [page]);

  if (loadingAccount && !account) {
    return (
      <div className={styles.centered}>
        <Spinner />
      </div>
    );
  }

  if (!account) return null;

  return (
    <div className={styles.container}>
      {/* Header */}
      <div className={styles.header}>
        <button className={styles.backBtn} onClick={handleBack} aria-label="Back to accounts">
          ← Back
        </button>
        <div className={styles.headerInfo}>
          <h2 className={styles.accountName}>{account.name}</h2>
          <span className={styles.accountType}>{ACCOUNT_TYPE_LABELS[account.type]}</span>
        </div>
        <div className={styles.balanceBlock}>
          <div className={styles.balance}>{formatCurrency(balance ?? 0)}</div>
          <div className={styles.balanceLabel}>Current Balance</div>
        </div>
      </div>

      {/* Summary bar — reflects current page of transactions */}
      <div className={styles.summaryBar}>
        <div className={styles.summaryItem}>
          <span className={styles.summaryLabel}>Income (this page)</span>
          <span className={`${styles.summaryValue} ${styles.income}`}>{formatCurrency(totalIncome)}</span>
        </div>
        <div className={styles.summaryItem}>
          <span className={styles.summaryLabel}>Expense (this page)</span>
          <span className={`${styles.summaryValue} ${styles.expense}`}>{formatCurrency(totalExpense)}</span>
        </div>
        <div className={styles.summaryItem}>
          <span className={styles.summaryLabel}>Net (this page)</span>
          <span className={styles.summaryValue}>{formatCurrency(net)}</span>
        </div>
      </div>

      {/* Transaction list */}
      <div className={styles.content}>
        {loadingTxns ? (
          <div className={styles.centeredSmall}>
            <Spinner />
          </div>
        ) : (
          <TransactionList
            page={page}
            onPageChange={setCurrentPage}
            onRowClick={setEditing}
          />
        )}
      </div>

      {/* Edit modal */}
      {editing && (
        <TransactionEditModal
          transaction={editing}
          categories={categories}
          onSaved={handleEditSaved}
          onClose={() => setEditing(null)}
        />
      )}
    </div>
  );
}
