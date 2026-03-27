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
  groupId?: string;
  groupName?: string;
  createdAt: string;
  updatedAt: string;
}

export interface CategoryGroup {
  id: string;
  name: string;
  sortOrder: number;
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
  isSplit: boolean;
  transferGroupId: string | null;
  importBatchId: string | null;
  createdAt: string;
  updatedAt: string;
  tags: TagBrief[];
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
  tagId?: string;
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

// ── Import ────────────────────────────────────────────────────────

export interface CsvPreview {
  headers: string[];
  rows: string[][];
  delimiter: string;
}

export interface ColumnMapping {
  csvColumn: string;
  field: string; // 'date' | 'payee' | 'description' | 'amount' | 'debit' | 'credit'
}

export interface ImportInput {
  filePath: string;
  accountId: string;
  mappings: ColumnMapping[];
  dateFormat: string;
  amountFormat: string; // 'single' | 'debit_credit'
  profileId?: string | null;
}

export interface ImportRowError {
  row: number;
  message: string;
}

export interface ImportResult {
  batchId: string;
  imported: number;
  duplicates: number;
  errors: ImportRowError[];
}

export interface ImportBatch {
  id: string;
  filename: string;
  fileHash: string;
  accountId: string;
  profileId: string | null;
  rowCount: number;
  createdAt: string;
}

export interface ImportProfile {
  id: string;
  name: string;
  mapping: string; // JSON string
  createdAt: string;
}

// ── Recurring Transactions ────────────────────────────────────────

export type Frequency = 'daily' | 'weekly' | 'biweekly' | 'monthly' | 'quarterly' | 'yearly';

export interface RecurringTransaction {
  id: string;
  accountId: string;
  payee: string;
  description: string;
  amount: number;
  kind: 'income' | 'expense' | 'transfer';
  categoryId: string | null;
  transferToAccountId: string | null;
  frequency: Frequency;
  startDate: string;
  endDate: string | null;
  nextDate: string;
  isActive: boolean;
  autoPost: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface CreateRecurringInput {
  accountId: string;
  payee?: string;
  description?: string;
  amount: number;
  kind: 'income' | 'expense' | 'transfer';
  categoryId?: string;
  transferToAccountId?: string;
  frequency: Frequency;
  startDate: string;
  endDate?: string;
  autoPost?: boolean;
}

export interface UpdateRecurringInput {
  accountId?: string;
  payee?: string;
  description?: string;
  amount?: number;
  kind?: string;
  categoryId?: string;
  transferToAccountId?: string;
  frequency?: string;
  startDate?: string;
  endDate?: string;
  nextDate?: string;
  isActive?: boolean;
  autoPost?: boolean;
}

export interface DueRecurringItem {
  recurring: RecurringTransaction;
  missedCount: number;
}

export interface GenerateResult {
  generatedCount: number;
  itemsProcessed: number;
}

// ── Search ────────────────────────────────────────────────────────

export interface SearchResult extends Transaction {
  snippet: string;
}

export interface SearchSuggestion {
  text: string;
  field: string;
}

// ── Export + Backup ──────────────────────────────────────────────

export type ExportFormat = 'csv' | 'json' | 'full-json';

export interface ExportFilters {
  dateFrom?: string;
  dateTo?: string;
  accountId?: string;
}

export interface BackupInfo {
  fileSizeBytes: number;
  accountCount: number;
  categoryCount: number;
  transactionCount: number;
  recurringCount: number;
  dateRangeFrom: string | null;
  dateRangeTo: string | null;
}

// ── Budgeting ────────────────────────────────────────────────────

export interface Budget {
  id: string;
  year: number;
  month: number;
  notes: string;
  createdAt: string;
  updatedAt: string;
}

export interface BudgetCategoryRow {
  itemId: string;
  categoryId: string;
  categoryName: string;
  categoryColor: string;
  budgeted: number;  // cents
  spent: number;     // cents
  available: number; // cents (budgeted - spent)
  rollover: boolean;
  groupId?: string;
  groupName?: string;
  targetAmount?: number | null;
}

export interface BudgetWithItems {
  budget: Budget;
  items: BudgetCategoryRow[];
}

export interface BudgetSummary {
  totalBudgeted: number;
  totalSpent: number;
  totalAvailable: number;
  totalIncome: number;
  toBeBudgeted: number;
}

export interface UpdateBudgetItemInput {
  budgeted?: number;
  rollover?: boolean;
  targetAmount?: number | null;
}

// ── Categorization Rules ──────────────────────────────────────────

export type MatchField = 'payee' | 'description' | 'amount';
export type MatchType = 'contains' | 'starts_with' | 'ends_with' | 'exact' | 'regex';

export interface CategorizationRule {
  id: string;
  name: string;
  matchField: MatchField;
  matchType: MatchType;
  pattern: string;
  categoryId: string;
  priority: number;
  isActive: boolean;
  applyCount: number;
  createdAt: string;
  updatedAt: string;
}

export interface CreateRuleInput {
  name: string;
  matchField: MatchField;
  matchType: MatchType;
  pattern: string;
  categoryId: string;
}

export interface UpdateRuleInput {
  name?: string;
  matchField?: MatchField;
  matchType?: MatchType;
  pattern?: string;
  categoryId?: string;
  isActive?: boolean;
}

export interface RuleTestResult {
  transactionId: string;
  date: string;
  payee: string;
  description: string;
  amount: number;
  matchedFieldValue: string;
}

// ── Splits ────────────────────────────────────────────────────────

export interface TransactionSplit {
  id: string;
  transactionId: string;
  categoryId: string;
  categoryName: string | null;
  categoryColor: string | null;
  amount: number;
  description: string;
  createdAt: string;
}

export interface CreateSplitInput {
  categoryId: string;
  amount: number;
  description: string;
}

// ── Tags ──────────────────────────────────────────────────────────

export interface Tag {
  id: string;
  name: string;
  color: string | null;
  createdAt: string;
}

export interface CreateTagInput {
  name: string;
  color?: string | null;
}

export interface TagBrief {
  id: string;
  name: string;
  color: string | null;
}

// ── Reports (Phase 17) ──────────────────────────────────────────

export interface ReportDateRange {
  startDate: string;
  endDate: string;
}

export interface IncomeExpenseMonth {
  month: string;
  income: number;
  expense: number;
  net: number;
}

export interface IncomeExpenseReport {
  months: IncomeExpenseMonth[];
  previousIncome: number;
  previousExpense: number;
}

export interface CategorySpendingRow {
  categoryId: string | null;
  categoryName: string;
  categoryColor: string;
  total: number;
  percentage: number;
}

export interface CashFlowPoint {
  date: string;
  income: number;
  expense: number;
  net: number;
  cumulative: number;
}

export interface NetWorthAccount {
  accountId: string;
  accountName: string;
  balance: number;
}

export interface NetWorthPoint {
  month: string;
  total: number;
  accounts: NetWorthAccount[];
}

export interface TagSpendingRow {
  tagId: string;
  tagName: string;
  tagColor: string | null;
  total: number;
}

export interface PayeeSpendingRow {
  payee: string;
  total: number;
  count: number;
}

// ── Reconciliation (Phase 18) ────────────────────────────────────

export interface Reconciliation {
  id: string;
  accountId: string;
  statementDate: string;
  statementBalance: number;
  status: 'pending' | 'completed' | 'cancelled';
  createdAt: string;
}

export interface ReconciliationItemDetail {
  transactionId: string;
  date: string;
  payee: string;
  amount: number;
  kind: string;
  isReconciled: boolean;
}

export interface ReconciliationSummary {
  reconciliation: Reconciliation;
  items: ReconciliationItemDetail[];
  clearedBalance: number;
  statementBalance: number;
  difference: number;
}

export interface StartReconciliationInput {
  accountId: string;
  statementDate: string;
  statementBalance: number;
}

export interface ExchangeRate {
  id: string;
  fromCurrency: string;
  toCurrency: string;
  rate: number;
  date: string;
  createdAt: string;
}

export interface CreateExchangeRateInput {
  fromCurrency: string;
  toCurrency: string;
  rate: number;
  date: string;
}

// ── Encryption ───────────────────────────────────────────────────

export interface EncryptionStatus {
  isEncrypted: boolean;
}

// ── Agent Gateway ─────────────────────────────────────────────────

export interface Agent {
  id: string;
  name: string;
  publisher: string;
  version: string;
  description?: string;
  manifestUrl?: string;
  iconUrl?: string;
  status: 'pending' | 'approved' | 'denied' | 'revoked';
  createdAt: string;
  updatedAt: string;
}

export interface CreateAgentInput {
  name: string;
  publisher: string;
  version?: string;
  description?: string;
  manifestUrl?: string;
  iconUrl?: string;
}

export interface UpdateAgentInput {
  name?: string;
  publisher?: string;
  version?: string;
  description?: string;
  manifestUrl?: string;
  iconUrl?: string;
}

export interface AgentKey {
  id: string;
  agentId: string;
  scopes: string;
  expiresAt?: string;
  lastUsedAt?: string;
  createdAt: string;
}

export interface AgentKeyCreated {
  id: string;
  agentId: string;
  rawKey: string;
  scopes: string;
  expiresAt?: string;
  createdAt: string;
}

export interface ScopeGrant {
  id: string;
  agentId: string;
  scope: string;
  accessMode: 'always' | 'ask_each_time' | 'never';
}

export interface AuditEntry {
  id: string;
  agentId: string;
  scopeRequested: string;
  scopeGranted: string;
  status: string;
  resultSummary?: string;
  createdAt: string;
}

// ── Views ─────────────────────────────────────────────────────────

export type View = 'dashboard' | 'accounts' | 'account-detail' | 'transactions' | 'categories' | 'import' | 'recurring' | 'budgets' | 'rules' | 'reports' | 'reconciliation' | 'agents';
