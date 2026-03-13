// ── Accounts ──────────────────────────────────────────────────────

export interface Account {
  id: string;
  name: string;
  type: 'checking' | 'savings' | 'credit_card' | 'cash' | 'investment' | 'other';
  currencyCode: string;
  openingBalance: number; // cents
  sortOrder: number;
  isArchived: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface CreateAccountInput {
  name: string;
  type: Account['type'];
  currencyCode?: string;
  openingBalance?: number;
  sortOrder?: number;
}

export interface UpdateAccountInput {
  name?: string;
  type?: Account['type'];
  currencyCode?: string;
  openingBalance?: number;
  sortOrder?: number;
  isArchived?: boolean;
}

// ── Categories ────────────────────────────────────────────────────

export interface Category {
  id: string;
  name: string;
  color: string;
  icon: string;
  type: 'income' | 'expense';
  sortOrder: number;
  isArchived: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface CreateCategoryInput {
  name: string;
  type: Category['type'];
  color?: string;
  icon?: string;
  sortOrder?: number;
}

export interface UpdateCategoryInput {
  name?: string;
  type?: Category['type'];
  color?: string;
  icon?: string;
  sortOrder?: number;
  isArchived?: boolean;
}

// ── Transactions ──────────────────────────────────────────────────

export interface Transaction {
  id: string;
  accountId: string;
  date: string;           // YYYY-MM-DD
  amount: number;         // cents, always positive
  kind: 'income' | 'expense' | 'transfer';
  categoryId: string | null;
  categoryName: string | null;
  categoryColor: string | null;
  payee: string;
  description: string;
  isCleared: boolean;
  transferGroupId: string | null;
  importBatchId: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreateTransactionInput {
  accountId: string;
  date: string;
  amount: number;
  kind: Transaction['kind'];
  categoryId?: string | null;
  payee?: string;
  description?: string;
  isCleared?: boolean;
}

export interface UpdateTransactionInput {
  date?: string;
  amount?: number;
  kind?: Transaction['kind'];
  categoryId?: string | null;
  payee?: string;
  description?: string;
  isCleared?: boolean;
}

export interface TransactionFilters {
  accountId?: string;
  kind?: string;
  categoryId?: string;
  search?: string;
  dateFrom?: string;
  dateTo?: string;
  page?: number;
  pageSize?: number;
}

export interface TransactionPage {
  items: Transaction[];
  total: number;
  page: number;
  pageSize: number;
}

export interface CreateTransferInput {
  fromAccountId: string;
  toAccountId: string;
  amount: number;
  date: string;
  description?: string;
}

// ── Dashboard ─────────────────────────────────────────────────────

export interface CategorySummary {
  categoryId: string | null;
  categoryName: string;
  categoryColor: string;
  total: number; // cents (negative for expenses)
}

export interface MonthSummary {
  month: string; // YYYY-MM
  income: number;
  expense: number;
  net: number;
}

export interface AccountBalance {
  accountId: string;
  accountName: string;
  accountType: string;
  balance: number; // cents
}

export interface DashboardData {
  income: number;
  expense: number;
  net: number;
  byCategory: CategorySummary[];
}

// ── Views ─────────────────────────────────────────────────────────

export type View = 'dashboard' | 'accounts' | 'transactions' | 'categories' | 'import';
