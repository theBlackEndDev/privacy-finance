import {
  Account,
  AccountBalance,
  Category,
  CategorySpendingRow,
  DashboardData,
  IncomeExpenseReport,
  MonthSummary,
  NetWorthPoint,
  PayeeSpendingRow,
  TagBrief,
  TagSpendingRow,
  Transaction,
  TransactionFilters,
  TransactionPage,
  CashFlowPoint,
} from '../types';

const now = '2026-03-22T00:00:00Z';

const accounts: Account[] = [
  {
    id: 'acct-checking',
    name: 'Primary Checking',
    type: 'checking',
    currencyCode: 'USD',
    openingBalance: 250000,
    sortOrder: 0,
    isArchived: false,
    createdAt: now,
    updatedAt: now,
  },
  {
    id: 'acct-savings',
    name: 'High-Yield Savings',
    type: 'savings',
    currencyCode: 'USD',
    openingBalance: 1200000,
    sortOrder: 1,
    isArchived: false,
    createdAt: now,
    updatedAt: now,
  },
  {
    id: 'acct-card',
    name: 'Visa Rewards Card',
    type: 'credit_card',
    currencyCode: 'USD',
    openingBalance: 0,
    sortOrder: 2,
    isArchived: false,
    createdAt: now,
    updatedAt: now,
  },
];

const categories: Category[] = [
  { id: 'cat-salary', name: 'Salary', color: '#10b981', icon: 'briefcase', type: 'income', sortOrder: 0, isArchived: false, createdAt: now, updatedAt: now },
  { id: 'cat-freelance', name: 'Freelance', color: '#0ea5e9', icon: 'sparkles', type: 'income', sortOrder: 1, isArchived: false, createdAt: now, updatedAt: now },
  { id: 'cat-investments', name: 'Investments', color: '#8b5cf6', icon: 'chart', type: 'income', sortOrder: 2, isArchived: false, createdAt: now, updatedAt: now },
  { id: 'cat-housing', name: 'Housing', color: '#ef4444', icon: 'home', type: 'expense', sortOrder: 3, isArchived: false, createdAt: now, updatedAt: now },
  { id: 'cat-food', name: 'Food', color: '#f97316', icon: 'utensils', type: 'expense', sortOrder: 4, isArchived: false, createdAt: now, updatedAt: now },
  { id: 'cat-transport', name: 'Transportation', color: '#f59e0b', icon: 'car', type: 'expense', sortOrder: 5, isArchived: false, createdAt: now, updatedAt: now },
  { id: 'cat-utilities', name: 'Utilities', color: '#14b8a6', icon: 'bolt', type: 'expense', sortOrder: 6, isArchived: false, createdAt: now, updatedAt: now },
  { id: 'cat-shopping', name: 'Shopping', color: '#ec4899', icon: 'bag', type: 'expense', sortOrder: 7, isArchived: false, createdAt: now, updatedAt: now },
  { id: 'cat-health', name: 'Healthcare', color: '#e11d48', icon: 'heart', type: 'expense', sortOrder: 8, isArchived: false, createdAt: now, updatedAt: now },
  { id: 'cat-subs', name: 'Subscriptions', color: '#a855f7', icon: 'repeat', type: 'expense', sortOrder: 9, isArchived: false, createdAt: now, updatedAt: now },
  { id: 'cat-entertainment', name: 'Entertainment', color: '#6366f1', icon: 'play', type: 'expense', sortOrder: 10, isArchived: false, createdAt: now, updatedAt: now },
  { id: 'cat-personal', name: 'Personal', color: '#38bdf8', icon: 'user', type: 'expense', sortOrder: 11, isArchived: false, createdAt: now, updatedAt: now },
];

const tagCoffee: TagBrief = { id: 'tag-coffee', name: 'Coffee', color: '#a16207' };
const tagWork: TagBrief = { id: 'tag-work', name: 'Work', color: '#0ea5e9' };
const tagTravel: TagBrief = { id: 'tag-travel', name: 'Travel', color: '#7c3aed' };

const transactions: Transaction[] = [
  tx('txn-001', 'acct-checking', '2026-03-31', 425000, 'income', 'cat-salary', 'Acme Corp', 'Salary — Mar', false, []),
  tx('txn-002', 'acct-checking', '2026-03-14', 425000, 'income', 'cat-salary', 'Acme Corp', 'Salary — Mar', true, []),
  tx('txn-003', 'acct-checking', '2026-03-05', 85000, 'income', 'cat-freelance', 'Apex Media', 'Website audit', true, [tagWork]),
  tx('txn-004', 'acct-savings', '2026-03-31', 19700, 'income', 'cat-investments', 'Vanguard', 'Dividend', false, []),
  tx('txn-005', 'acct-checking', '2026-03-01', 185000, 'expense', 'cat-housing', 'Sunset Apartments', 'Rent — March', true, []),
  tx('txn-006', 'acct-card', '2026-03-04', 13700, 'expense', 'cat-food', 'Whole Foods', 'Weekly groceries', false, []),
  tx('txn-007', 'acct-card', '2026-03-05', 6800, 'expense', 'cat-transport', 'Uber', 'Rides', false, [tagTravel]),
  tx('txn-008', 'acct-card', '2026-03-05', 1400, 'expense', 'cat-subs', 'New York Times', 'Monthly', false, []),
  tx('txn-009', 'acct-card', '2026-03-08', 8900, 'expense', 'cat-utilities', 'City Power & Gas', 'Electric + gas', false, []),
  tx('txn-010', 'acct-card', '2026-03-10', 7200, 'expense', 'cat-utilities', 'StreamNet ISP', 'Internet', false, []),
  tx('txn-011', 'acct-card', '2026-03-11', 11200, 'expense', 'cat-food', "Trader Joe's", 'Weekly groceries', false, []),
  tx('txn-012', 'acct-card', '2026-03-14', 9000, 'expense', 'cat-entertainment', 'AMC Theatres', 'Movie night x2', false, []),
  tx('txn-013', 'acct-card', '2026-03-15', 6400, 'expense', 'cat-food', 'The French Laundry', 'Birthday dinner', false, []),
  tx('txn-014', 'acct-card', '2026-03-18', 12900, 'expense', 'cat-food', 'Whole Foods', 'Weekly groceries', false, [tagCoffee]),
  tx('txn-015', 'acct-card', '2026-03-19', 8500, 'expense', 'cat-personal', 'Great Clips', 'Haircut', false, []),
  tx('txn-016', 'acct-card', '2026-03-20', 5500, 'expense', 'cat-transport', 'Chevron', 'Gas', false, []),
  tx('txn-017', 'acct-card', '2026-03-22', 20000, 'expense', 'cat-health', 'Dr. Chen', 'Annual physical', false, []),
  tx('txn-018', 'acct-card', '2026-03-07', 22300, 'expense', 'cat-shopping', 'Amazon', 'Misc household', false, []),
  tx('txn-019', 'acct-checking', '2026-03-05', 195000, 'transfer', null, 'Transfer', 'CC payment — February', true, [] , 'grp-cc-pay'),
  tx('txn-020', 'acct-card', '2026-03-05', 195000, 'transfer', null, 'Transfer', 'CC payment — February', true, [] , 'grp-cc-pay'),
  tx('txn-021', 'acct-checking', '2026-03-31', 50000, 'transfer', null, 'Transfer', 'Monthly savings transfer', false, [] , 'grp-save'),
  tx('txn-022', 'acct-savings', '2026-03-31', 50000, 'transfer', null, 'Transfer', 'Monthly savings transfer', false, [] , 'grp-save'),
  tx('txn-023', 'acct-card', '2026-02-25', 12100, 'expense', 'cat-food', "Trader Joe's", 'Weekly groceries', true, []),
  tx('txn-024', 'acct-checking', '2026-02-28', 425000, 'income', 'cat-salary', 'Acme Corp', 'Salary — Feb', true, []),
  tx('txn-025', 'acct-checking', '2026-02-14', 425000, 'income', 'cat-salary', 'Acme Corp', 'Salary — Feb', true, []),
  tx('txn-026', 'acct-checking', '2026-01-31', 425000, 'income', 'cat-salary', 'Acme Corp', 'Salary — Jan', true, []),
  tx('txn-027', 'acct-checking', '2026-01-15', 425000, 'income', 'cat-salary', 'Acme Corp', 'Salary — Jan', true, []),
];

const dashboardByMonth: Record<string, DashboardData> = {
  '2026-03': {
    income: 954700,
    expense: -288800,
    net: 665900,
    byCategory: [
      { categoryId: 'cat-housing', categoryName: 'Housing', categoryColor: '#ef4444', total: -185000 },
      { categoryId: 'cat-food', categoryName: 'Food', categoryColor: '#f97316', total: -44200 },
      { categoryId: 'cat-shopping', categoryName: 'Shopping', categoryColor: '#ec4899', total: -22300 },
      { categoryId: 'cat-health', categoryName: 'Healthcare', categoryColor: '#e11d48', total: -20000 },
      { categoryId: 'cat-utilities', categoryName: 'Utilities', categoryColor: '#14b8a6', total: -16100 },
      { categoryId: 'cat-transport', categoryName: 'Transportation', categoryColor: '#f59e0b', total: -12300 },
      { categoryId: 'cat-entertainment', categoryName: 'Entertainment', categoryColor: '#6366f1', total: -9000 },
    ],
  },
  '2026-02': {
    income: 871200,
    expense: -254800,
    net: 616400,
    byCategory: [
      { categoryId: 'cat-housing', categoryName: 'Housing', categoryColor: '#ef4444', total: -185000 },
      { categoryId: 'cat-food', categoryName: 'Food', categoryColor: '#f97316', total: -46900 },
      { categoryId: 'cat-shopping', categoryName: 'Shopping', categoryColor: '#ec4899', total: -52500 },
    ],
  },
  '2026-01': {
    income: 988400,
    expense: -301700,
    net: 686700,
    byCategory: [
      { categoryId: 'cat-housing', categoryName: 'Housing', categoryColor: '#ef4444', total: -185000 },
      { categoryId: 'cat-food', categoryName: 'Food', categoryColor: '#f97316', total: -68000 },
      { categoryId: 'cat-entertainment', categoryName: 'Entertainment', categoryColor: '#6366f1', total: -23000 },
    ],
  },
};

const monthlyTrend: MonthSummary[] = [
  { month: '2025-10', income: 812000, expense: -246000, net: 566000 },
  { month: '2025-11', income: 825000, expense: -258000, net: 567000 },
  { month: '2025-12', income: 841000, expense: -272000, net: 569000 },
  { month: '2026-01', income: 988400, expense: -301700, net: 686700 },
  { month: '2026-02', income: 871200, expense: -254800, net: 616400 },
  { month: '2026-03', income: 954700, expense: -288800, net: 665900 },
];

const balances: AccountBalance[] = [
  { accountId: 'acct-checking', accountName: 'Primary Checking', accountType: 'checking', balance: 1068500 },
  { accountId: 'acct-savings', accountName: 'High-Yield Savings', accountType: 'savings', balance: 1339700 },
  { accountId: 'acct-card', accountName: 'Visa Rewards Card', accountType: 'credit_card', balance: -93800 },
];

const incomeExpenseReport: IncomeExpenseReport = {
  months: [
    { month: '2026-01', income: 988400, expense: -301700, net: 686700 },
    { month: '2026-02', income: 871200, expense: -254800, net: 616400 },
    { month: '2026-03', income: 954700, expense: -288800, net: 665900 },
  ],
  previousIncome: 812000,
  previousExpense: 246000,
};

const categorySpending: CategorySpendingRow[] = [
  { categoryId: 'cat-housing', categoryName: 'Housing', categoryColor: '#ef4444', total: -185000, percentage: 64.1 },
  { categoryId: 'cat-food', categoryName: 'Food', categoryColor: '#f97316', total: -44200, percentage: 15.3 },
  { categoryId: 'cat-shopping', categoryName: 'Shopping', categoryColor: '#ec4899', total: -22300, percentage: 7.7 },
  { categoryId: 'cat-health', categoryName: 'Healthcare', categoryColor: '#e11d48', total: -20000, percentage: 6.9 },
  { categoryId: 'cat-utilities', categoryName: 'Utilities', categoryColor: '#14b8a6', total: -16100, percentage: 5.6 },
  { categoryId: 'cat-transport', categoryName: 'Transportation', categoryColor: '#f59e0b', total: -12300, percentage: 4.2 },
];

const cashFlow: CashFlowPoint[] = [
  { date: '2026-03-01', income: 0, expense: -185000, net: -185000, cumulative: 815000 },
  { date: '2026-03-05', income: 85000, expense: -30500, net: 54500, cumulative: 869500 },
  { date: '2026-03-08', income: 0, expense: -8900, net: -8900, cumulative: 860600 },
  { date: '2026-03-14', income: 425000, expense: -9000, net: 416000, cumulative: 1276600 },
  { date: '2026-03-18', income: 0, expense: -12900, net: -12900, cumulative: 1263700 },
  { date: '2026-03-31', income: 444700, expense: 0, net: 444700, cumulative: 1708400 },
];

const netWorth: NetWorthPoint[] = [
  {
    month: '2026-01',
    total: 2135200,
    accounts: [
      { accountId: 'acct-checking', accountName: 'Primary Checking', balance: 915000 },
      { accountId: 'acct-savings', accountName: 'High-Yield Savings', balance: 1290000 },
      { accountId: 'acct-card', accountName: 'Visa Rewards Card', balance: -69800 },
    ],
  },
  {
    month: '2026-02',
    total: 2269400,
    accounts: [
      { accountId: 'acct-checking', accountName: 'Primary Checking', balance: 1002500 },
      { accountId: 'acct-savings', accountName: 'High-Yield Savings', balance: 1310700 },
      { accountId: 'acct-card', accountName: 'Visa Rewards Card', balance: -43800 },
    ],
  },
  {
    month: '2026-03',
    total: 2314400,
    accounts: [
      { accountId: 'acct-checking', accountName: 'Primary Checking', balance: 1068500 },
      { accountId: 'acct-savings', accountName: 'High-Yield Savings', balance: 1339700 },
      { accountId: 'acct-card', accountName: 'Visa Rewards Card', balance: -93800 },
    ],
  },
];

const tagSpending: TagSpendingRow[] = [
  { tagId: 'tag-work', tagName: 'Work', tagColor: '#0ea5e9', total: -8700 },
  { tagId: 'tag-coffee', tagName: 'Coffee', tagColor: '#a16207', total: -12900 },
  { tagId: 'tag-travel', tagName: 'Travel', tagColor: '#7c3aed', total: -6800 },
];

const payeeSpending: PayeeSpendingRow[] = [
  { payee: 'Sunset Apartments', total: -185000, count: 1 },
  { payee: 'Whole Foods', total: -26600, count: 2 },
  { payee: 'Trader Joe\'s', total: -11200, count: 1 },
  { payee: 'Amazon', total: -22300, count: 1 },
  { payee: 'City Power & Gas', total: -8900, count: 1 },
];

function tx(
  id: string,
  accountId: string,
  date: string,
  amount: number,
  kind: Transaction['kind'],
  categoryId: string | null,
  payee: string,
  description: string,
  isCleared: boolean,
  tags: TagBrief[],
  transferGroupId: string | null = null,
): Transaction {
  const category = categories.find((c) => c.id === categoryId) ?? null;
  return {
    id,
    accountId,
    date,
    amount,
    kind,
    categoryId,
    categoryName: category?.name ?? null,
    categoryColor: category?.color ?? null,
    payee,
    description,
    isCleared,
    isSplit: false,
    transferGroupId,
    importBatchId: null,
    createdAt: now,
    updatedAt: now,
    tags,
  };
}

function applyTransactionFilters(items: Transaction[], filters: TransactionFilters = {}): TransactionPage {
  let filtered = [...items];
  if (filters.accountId) filtered = filtered.filter((t) => t.accountId === filters.accountId);
  if (filters.kind) filtered = filtered.filter((t) => t.kind === filters.kind);
  if (filters.categoryId) filtered = filtered.filter((t) => t.categoryId === filters.categoryId);
  if (filters.search) {
    const q = filters.search.toLowerCase();
    filtered = filtered.filter((t) =>
      t.payee.toLowerCase().includes(q) ||
      t.description.toLowerCase().includes(q) ||
      (t.categoryName ?? '').toLowerCase().includes(q)
    );
  }
  if (filters.dateFrom) filtered = filtered.filter((t) => t.date >= filters.dateFrom!);
  if (filters.dateTo) filtered = filtered.filter((t) => t.date <= filters.dateTo!);
  filtered.sort((a, b) => (a.date < b.date ? 1 : a.date > b.date ? -1 : a.id.localeCompare(b.id)));

  const page = filters.page ?? 0;
  const pageSize = filters.pageSize ?? 50;
  const start = page * pageSize;
  const end = start + pageSize;
  return {
    items: filtered.slice(start, end),
    total: filtered.length,
    page,
    pageSize,
  };
}

export function getDemoInvokeFallback(cmd: string, args?: Record<string, unknown>): unknown {
  switch (cmd) {
    case 'get_setting': {
      const key = String((args?.key as string | undefined) ?? '');
      if (key === 'theme') return 'dark';
      if (key === 'onboarding_complete') return 'true';
      return null;
    }
    case 'set_setting':
      return undefined;
    case 'get_accounts':
      return accounts;
    case 'get_account':
      return accounts.find((a) => a.id === args?.id) ?? accounts[0];
    case 'get_categories':
      return categories;
    case 'get_dashboard_data': {
      const month = String((args?.month as string | undefined) ?? '2026-03');
      return dashboardByMonth[month] ?? dashboardByMonth['2026-03'];
    }
    case 'get_monthly_trend': {
      const months = Number((args?.months as number | undefined) ?? 12);
      return monthlyTrend.slice(-months);
    }
    case 'get_account_balances':
      return balances;
    case 'get_transactions':
      return applyTransactionFilters(transactions, (args?.filters as TransactionFilters | undefined) ?? {});
    case 'get_transaction':
      return transactions.find((t) => t.id === args?.id) ?? transactions[0];
    case 'get_payees': {
      const search = String((args?.search as string | undefined) ?? '').toLowerCase();
      const payees = [...new Set(transactions.map((t) => t.payee))];
      return payees.filter((p) => p.toLowerCase().includes(search)).slice(0, 8);
    }
    case 'get_income_expense_report':
      return incomeExpenseReport;
    case 'get_category_spending_report':
      return categorySpending;
    case 'get_cash_flow_report':
      return cashFlow;
    case 'get_net_worth_report':
      return netWorth;
    case 'get_tag_spending_report':
      return tagSpending;
    case 'get_payee_spending_report':
      return payeeSpending;
    default:
      return undefined;
  }
}
