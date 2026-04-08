import {
  Account,
  AccountBalance,
  BudgetCategoryRow,
  BudgetSummary,
  BudgetWithItems,
  CashFlowPoint,
  CategorizationRule,
  Category,
  CategorySpendingRow,
  DashboardData,
  DueRecurringItem,
  IncomeExpenseReport,
  MonthSummary,
  NetWorthPoint,
  PayeeSpendingRow,
  RecurringTransaction,
  TagBrief,
  TagSpendingRow,
  Transaction,
  TransactionFilters,
  TransactionPage,
} from '../types';

function daysAgo(n: number): string {
  const d = new Date();
  d.setDate(d.getDate() - n);
  return d.toISOString().replace(/\.\d{3}Z$/, 'Z');
}

function dateStr(daysBack: number): string {
  const d = new Date();
  d.setDate(d.getDate() - daysBack);
  return d.toISOString().slice(0, 10);
}

function monthStr(monthsBack: number): string {
  const d = new Date();
  d.setMonth(d.getMonth() - monthsBack);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
}

const now = daysAgo(0);

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
  tx('txn-001', 'acct-checking', dateStr(0), 425000, 'income', 'cat-salary', 'Acme Corp', 'Salary — this month', false, []),
  tx('txn-002', 'acct-checking', dateStr(17), 425000, 'income', 'cat-salary', 'Acme Corp', 'Salary — this month', true, []),
  tx('txn-003', 'acct-checking', dateStr(26), 85000, 'income', 'cat-freelance', 'Apex Media', 'Website audit', true, [tagWork]),
  tx('txn-004', 'acct-savings', dateStr(0), 19700, 'income', 'cat-investments', 'Vanguard', 'Dividend', false, []),
  tx('txn-005', 'acct-checking', dateStr(30), 185000, 'expense', 'cat-housing', 'Sunset Apartments', 'Rent', true, []),
  tx('txn-006', 'acct-card', dateStr(27), 13700, 'expense', 'cat-food', 'Whole Foods', 'Weekly groceries', false, []),
  tx('txn-007', 'acct-card', dateStr(26), 6800, 'expense', 'cat-transport', 'Uber', 'Rides', false, [tagTravel]),
  tx('txn-008', 'acct-card', dateStr(26), 1400, 'expense', 'cat-subs', 'New York Times', 'Monthly', false, []),
  tx('txn-009', 'acct-card', dateStr(23), 8900, 'expense', 'cat-utilities', 'City Power & Gas', 'Electric + gas', false, []),
  tx('txn-010', 'acct-card', dateStr(21), 7200, 'expense', 'cat-utilities', 'StreamNet ISP', 'Internet', false, []),
  tx('txn-011', 'acct-card', dateStr(20), 11200, 'expense', 'cat-food', "Trader Joe's", 'Weekly groceries', false, []),
  tx('txn-012', 'acct-card', dateStr(17), 9000, 'expense', 'cat-entertainment', 'AMC Theatres', 'Movie night x2', false, []),
  tx('txn-013', 'acct-card', dateStr(16), 6400, 'expense', 'cat-food', 'The French Laundry', 'Birthday dinner', false, []),
  tx('txn-014', 'acct-card', dateStr(13), 12900, 'expense', 'cat-food', 'Whole Foods', 'Weekly groceries', false, [tagCoffee]),
  tx('txn-015', 'acct-card', dateStr(12), 8500, 'expense', 'cat-personal', 'Great Clips', 'Haircut', false, []),
  tx('txn-016', 'acct-card', dateStr(11), 5500, 'expense', 'cat-transport', 'Chevron', 'Gas', false, []),
  tx('txn-017', 'acct-card', dateStr(9), 20000, 'expense', 'cat-health', 'Dr. Chen', 'Annual physical', false, []),
  tx('txn-018', 'acct-card', dateStr(24), 22300, 'expense', 'cat-shopping', 'Amazon', 'Misc household', false, []),
  tx('txn-019', 'acct-checking', dateStr(26), 195000, 'transfer', null, 'Transfer', 'CC payment — last month', true, [] , 'grp-cc-pay'),
  tx('txn-020', 'acct-card', dateStr(26), 195000, 'transfer', null, 'Transfer', 'CC payment — last month', true, [] , 'grp-cc-pay'),
  tx('txn-021', 'acct-checking', dateStr(0), 50000, 'transfer', null, 'Transfer', 'Monthly savings transfer', false, [] , 'grp-save'),
  tx('txn-022', 'acct-savings', dateStr(0), 50000, 'transfer', null, 'Transfer', 'Monthly savings transfer', false, [] , 'grp-save'),
  tx('txn-023', 'acct-card', dateStr(36), 12100, 'expense', 'cat-food', "Trader Joe's", 'Weekly groceries', true, []),
  tx('txn-024', 'acct-checking', dateStr(33), 425000, 'income', 'cat-salary', 'Acme Corp', 'Salary — last month', true, []),
  tx('txn-025', 'acct-checking', dateStr(47), 425000, 'income', 'cat-salary', 'Acme Corp', 'Salary — last month', true, []),
  tx('txn-026', 'acct-checking', dateStr(61), 425000, 'income', 'cat-salary', 'Acme Corp', 'Salary — 2 months ago', true, []),
  tx('txn-027', 'acct-checking', dateStr(77), 425000, 'income', 'cat-salary', 'Acme Corp', 'Salary — 2 months ago', true, []),
];

const dashboardByMonth: Record<string, DashboardData> = {
  [monthStr(0)]: {
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
  [monthStr(1)]: {
    income: 871200,
    expense: -254800,
    net: 616400,
    byCategory: [
      { categoryId: 'cat-housing', categoryName: 'Housing', categoryColor: '#ef4444', total: -185000 },
      { categoryId: 'cat-food', categoryName: 'Food', categoryColor: '#f97316', total: -46900 },
      { categoryId: 'cat-shopping', categoryName: 'Shopping', categoryColor: '#ec4899', total: -52500 },
    ],
  },
  [monthStr(2)]: {
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
  { month: monthStr(5), income: 812000, expense: -246000, net: 566000 },
  { month: monthStr(4), income: 825000, expense: -258000, net: 567000 },
  { month: monthStr(3), income: 841000, expense: -272000, net: 569000 },
  { month: monthStr(2), income: 988400, expense: -301700, net: 686700 },
  { month: monthStr(1), income: 871200, expense: -254800, net: 616400 },
  { month: monthStr(0), income: 954700, expense: -288800, net: 665900 },
];

const balances: AccountBalance[] = [
  { accountId: 'acct-checking', accountName: 'Primary Checking', accountType: 'checking', balance: 1068500 },
  { accountId: 'acct-savings', accountName: 'High-Yield Savings', accountType: 'savings', balance: 1339700 },
  { accountId: 'acct-card', accountName: 'Visa Rewards Card', accountType: 'credit_card', balance: -93800 },
];

const incomeExpenseReport: IncomeExpenseReport = {
  months: [
    { month: monthStr(2), income: 988400, expense: -301700, net: 686700 },
    { month: monthStr(1), income: 871200, expense: -254800, net: 616400 },
    { month: monthStr(0), income: 954700, expense: -288800, net: 665900 },
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
  { date: dateStr(30), income: 0, expense: -185000, net: -185000, cumulative: 815000 },
  { date: dateStr(26), income: 85000, expense: -30500, net: 54500, cumulative: 869500 },
  { date: dateStr(23), income: 0, expense: -8900, net: -8900, cumulative: 860600 },
  { date: dateStr(17), income: 425000, expense: -9000, net: 416000, cumulative: 1276600 },
  { date: dateStr(13), income: 0, expense: -12900, net: -12900, cumulative: 1263700 },
  { date: dateStr(0), income: 444700, expense: 0, net: 444700, cumulative: 1708400 },
];

const netWorth: NetWorthPoint[] = [
  {
    month: monthStr(2),
    total: 2135200,
    accounts: [
      { accountId: 'acct-checking', accountName: 'Primary Checking', balance: 915000 },
      { accountId: 'acct-savings', accountName: 'High-Yield Savings', balance: 1290000 },
      { accountId: 'acct-card', accountName: 'Visa Rewards Card', balance: -69800 },
    ],
  },
  {
    month: monthStr(1),
    total: 2269400,
    accounts: [
      { accountId: 'acct-checking', accountName: 'Primary Checking', balance: 1002500 },
      { accountId: 'acct-savings', accountName: 'High-Yield Savings', balance: 1310700 },
      { accountId: 'acct-card', accountName: 'Visa Rewards Card', balance: -43800 },
    ],
  },
  {
    month: monthStr(0),
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

const budgetItems: BudgetCategoryRow[] = [
  {
    itemId: 'budget-housing',
    categoryId: 'cat-housing',
    categoryName: 'Housing',
    categoryColor: '#ef4444',
    budgeted: 185000,
    spent: 185000,
    available: 0,
    rollover: false,
    groupId: 'grp-fixed',
    groupName: 'Fixed Costs',
    targetAmount: 185000,
  },
  {
    itemId: 'budget-utilities',
    categoryId: 'cat-utilities',
    categoryName: 'Utilities',
    categoryColor: '#14b8a6',
    budgeted: 18000,
    spent: 16100,
    available: 1900,
    rollover: true,
    groupId: 'grp-fixed',
    groupName: 'Fixed Costs',
    targetAmount: 18000,
  },
  {
    itemId: 'budget-food',
    categoryId: 'cat-food',
    categoryName: 'Food',
    categoryColor: '#f97316',
    budgeted: 52000,
    spent: 44200,
    available: 7800,
    rollover: true,
    groupId: 'grp-flex',
    groupName: 'Flexible Spending',
    targetAmount: 50000,
  },
  {
    itemId: 'budget-transport',
    categoryId: 'cat-transport',
    categoryName: 'Transportation',
    categoryColor: '#f59e0b',
    budgeted: 18000,
    spent: 12300,
    available: 5700,
    rollover: true,
    groupId: 'grp-flex',
    groupName: 'Flexible Spending',
    targetAmount: 15000,
  },
  {
    itemId: 'budget-shopping',
    categoryId: 'cat-shopping',
    categoryName: 'Shopping',
    categoryColor: '#ec4899',
    budgeted: 15000,
    spent: 22300,
    available: -7300,
    rollover: false,
    groupId: 'grp-flex',
    groupName: 'Flexible Spending',
    targetAmount: 12000,
  },
  {
    itemId: 'budget-entertainment',
    categoryId: 'cat-entertainment',
    categoryName: 'Entertainment',
    categoryColor: '#6366f1',
    budgeted: 12000,
    spent: 9000,
    available: 3000,
    rollover: true,
    groupId: 'grp-flex',
    groupName: 'Flexible Spending',
    targetAmount: 10000,
  },
  {
    itemId: 'budget-health',
    categoryId: 'cat-health',
    categoryName: 'Healthcare',
    categoryColor: '#e11d48',
    budgeted: 22000,
    spent: 20000,
    available: 2000,
    rollover: true,
    groupId: 'grp-goals',
    groupName: 'Goals & Protection',
    targetAmount: 22000,
  },
  {
    itemId: 'budget-subs',
    categoryId: 'cat-subs',
    categoryName: 'Subscriptions',
    categoryColor: '#a855f7',
    budgeted: 2500,
    spent: 1400,
    available: 1100,
    rollover: true,
    groupId: 'grp-fixed',
    groupName: 'Fixed Costs',
    targetAmount: 2500,
  },
];

const budgetData: BudgetWithItems = {
  budget: {
    id: `budget-${monthStr(0)}`,
    year: new Date().getFullYear(),
    month: new Date().getMonth() + 1,
    notes: 'Monthly plan for household spending and savings targets',
    createdAt: now,
    updatedAt: now,
  },
  items: budgetItems,
};

const budgetSummary: BudgetSummary = {
  totalBudgeted: 324500,
  totalSpent: 310300,
  totalAvailable: 14200,
  totalIncome: 954700,
  toBeBudgeted: 630200,
};

const rules: CategorizationRule[] = [
  {
    id: 'rule-1',
    name: 'Whole Foods → Food',
    matchField: 'payee',
    matchType: 'contains',
    pattern: 'whole foods',
    categoryId: 'cat-food',
    priority: 1,
    isActive: true,
    applyCount: 18,
    createdAt: now,
    updatedAt: now,
  },
  {
    id: 'rule-2',
    name: 'Uber → Transportation',
    matchField: 'payee',
    matchType: 'contains',
    pattern: 'uber',
    categoryId: 'cat-transport',
    priority: 2,
    isActive: true,
    applyCount: 9,
    createdAt: now,
    updatedAt: now,
  },
  {
    id: 'rule-3',
    name: 'Salary deposits',
    matchField: 'description',
    matchType: 'contains',
    pattern: 'salary',
    categoryId: 'cat-salary',
    priority: 3,
    isActive: true,
    applyCount: 6,
    createdAt: now,
    updatedAt: now,
  },
  {
    id: 'rule-4',
    name: 'Prime / Amazon purchases',
    matchField: 'payee',
    matchType: 'contains',
    pattern: 'amazon',
    categoryId: 'cat-shopping',
    priority: 4,
    isActive: true,
    applyCount: 11,
    createdAt: now,
    updatedAt: now,
  },
  {
    id: 'rule-5',
    name: 'Streaming & news subscriptions',
    matchField: 'description',
    matchType: 'contains',
    pattern: 'monthly',
    categoryId: 'cat-subs',
    priority: 5,
    isActive: false,
    applyCount: 3,
    createdAt: now,
    updatedAt: now,
  },
];

const recurringItems: RecurringTransaction[] = [
  {
    id: 'rec-1',
    accountId: 'acct-checking',
    payee: 'Sunset Apartments',
    description: 'Monthly rent',
    amount: 185000,
    kind: 'expense',
    categoryId: 'cat-housing',
    transferToAccountId: null,
    frequency: 'monthly',
    startDate: dateStr(365),
    endDate: null,
    nextDate: dateStr(-1),
    isActive: true,
    autoPost: true,
    createdAt: now,
    updatedAt: now,
  },
  {
    id: 'rec-2',
    accountId: 'acct-checking',
    payee: 'Acme Corp',
    description: 'Payroll deposit',
    amount: 425000,
    kind: 'income',
    categoryId: 'cat-salary',
    transferToAccountId: null,
    frequency: 'biweekly',
    startDate: dateStr(365),
    endDate: null,
    nextDate: dateStr(3),
    isActive: true,
    autoPost: true,
    createdAt: now,
    updatedAt: now,
  },
  {
    id: 'rec-3',
    accountId: 'acct-card',
    payee: 'New York Times',
    description: 'News subscription',
    amount: 1400,
    kind: 'expense',
    categoryId: 'cat-subs',
    transferToAccountId: null,
    frequency: 'monthly',
    startDate: dateStr(240),
    endDate: null,
    nextDate: dateStr(-5),
    isActive: true,
    autoPost: false,
    createdAt: now,
    updatedAt: now,
  },
  {
    id: 'rec-4',
    accountId: 'acct-checking',
    payee: 'Savings Transfer',
    description: 'Monthly savings transfer',
    amount: 50000,
    kind: 'transfer',
    categoryId: null,
    transferToAccountId: 'acct-savings',
    frequency: 'monthly',
    startDate: dateStr(280),
    endDate: null,
    nextDate: dateStr(0),
    isActive: true,
    autoPost: true,
    createdAt: now,
    updatedAt: now,
  },
];

const dueRecurringItems: DueRecurringItem[] = [];

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
      if (key === 'theme') return 'light';
      if (key === 'onboarding_complete') return 'true';
      if (key === 'currency_symbol') return '$';
      if (key === 'date_format') return 'MMM D';
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
      const month = String((args?.month as string | undefined) ?? monthStr(0));
      return dashboardByMonth[month] ?? dashboardByMonth[monthStr(0)];
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
    case 'get_budget':
      return budgetData;
    case 'get_budget_summary':
      return budgetSummary;
    case 'get_category_activity': {
      const categoryId = String((args?.categoryId as string | undefined) ?? '');
      const [year, month] = [Number(args?.year ?? 2026), Number(args?.month ?? 3)];
      const prefix = `${year}-${String(month).padStart(2, '0')}`;
      return transactions.filter((t) => t.categoryId === categoryId && t.date.startsWith(prefix));
    }
    case 'get_all_rules':
      return rules;
    case 'apply_rules_to_uncategorized':
      return 4;
    case 'test_rule':
      return transactions.slice(0, 3).map((t) => ({
        transactionId: t.id,
        date: t.date,
        payee: t.payee,
        description: t.description,
        amount: t.amount,
        matchedFieldValue: t.payee,
      }));
    case 'get_all_recurring':
      return recurringItems;
    case 'check_due_recurring':
      return dueRecurringItems;
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
