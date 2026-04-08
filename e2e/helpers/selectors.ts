/**
 * Selector helpers for E2E tests.
 * All selectors use data-testid attributes for stability.
 */

/** Select by data-testid attribute */
export function byTestId(id: string) {
  return `[data-testid="${id}"]`;
}

/** Select by data-testid with a suffix (e.g., row-123) */
export function byTestIdPrefix(prefix: string) {
  return `[data-testid^="${prefix}"]`;
}

/** Common selectors used across tests */
export const sel = {
  // Layout
  sidebar: byTestId('sidebar'),
  mainContent: byTestId('main-content'),
  header: byTestId('header'),

  // Sidebar navigation
  navDashboard: byTestId('nav-dashboard'),
  navAccounts: byTestId('nav-accounts'),
  navTransactions: byTestId('nav-transactions'),
  navCategories: byTestId('nav-categories'),
  navImport: byTestId('nav-import'),
  navRecurring: byTestId('nav-recurring'),
  navBudgets: byTestId('nav-budgets'),
  navRules: byTestId('nav-rules'),
  navReports: byTestId('nav-reports'),
  navReconciliation: byTestId('nav-reconciliation'),
  navSettings: byTestId('nav-settings'),

  // Common UI
  modal: byTestId('modal'),
  modalClose: byTestId('modal-close'),
  confirmDialog: byTestId('confirm-dialog'),
  confirmYes: byTestId('confirm-yes'),
  confirmNo: byTestId('confirm-no'),
  toast: byTestId('toast'),
  spinner: byTestId('spinner'),
  emptyState: byTestId('empty-state'),

  // Forms
  submitBtn: byTestId('submit-btn'),
  cancelBtn: byTestId('cancel-btn'),
  deleteBtn: byTestId('delete-btn'),

  // Search
  globalSearch: byTestId('global-search'),
  searchInput: byTestId('search-input'),
  searchResults: byTestId('search-results'),
  aiSearchCard: byTestId('ai-search-card'),

  // Onboarding
  onboardingWizard: byTestId('onboarding-wizard'),
  onboardingNext: byTestId('onboarding-next'),
  onboardingSkip: byTestId('onboarding-skip'),
  onboardingFinish: byTestId('onboarding-finish'),

  // Account form
  accountNameInput: byTestId('account-name-input'),
  accountTypeSelect: byTestId('account-type-select'),
  accountBalanceInput: byTestId('account-balance-input'),
  createAccountBtn: byTestId('create-account-btn'),

  // Transaction form
  txPayeeInput: byTestId('tx-payee-input'),
  txAmountInput: byTestId('tx-amount-input'),
  txDateInput: byTestId('tx-date-input'),
  txCategorySelect: byTestId('tx-category-select'),
  txKindSelect: byTestId('tx-kind-select'),
  txDescriptionInput: byTestId('tx-description-input'),
  txSubmitBtn: byTestId('tx-submit-btn'),
  txRow: byTestIdPrefix('tx-row-'),

  // Transaction edit modal
  txEditModal: byTestId('tx-edit-modal'),
  txEditPayee: byTestId('tx-edit-payee'),
  txEditAmount: byTestId('tx-edit-amount'),
  txEditSave: byTestId('tx-edit-save'),
  txEditDelete: byTestId('tx-edit-delete'),

  // Transaction filters
  txFilterAccount: byTestId('tx-filter-account'),
  txFilterCategory: byTestId('tx-filter-category'),
  txFilterDateFrom: byTestId('tx-filter-date-from'),
  txFilterDateTo: byTestId('tx-filter-date-to'),

  // Category form
  catNameInput: byTestId('cat-name-input'),
  catTypeSelect: byTestId('cat-type-select'),
  catColorPicker: byTestId('cat-color-picker'),
  catSubmitBtn: byTestId('cat-submit-btn'),
  catRow: byTestIdPrefix('cat-row-'),
  createCategoryBtn: byTestId('create-category-btn'),

  // Budget
  budgetMonthNav: byTestId('budget-month-nav'),
  budgetNextMonth: byTestId('budget-next-month'),
  budgetPrevMonth: byTestId('budget-prev-month'),
  budgetCopyBtn: byTestId('budget-copy-btn'),
  budgetAmountInput: byTestIdPrefix('budget-amount-'),
  budgetProgressBar: byTestIdPrefix('budget-progress-'),

  // Recurring
  recurringCreateBtn: byTestId('recurring-create-btn'),
  recurringPayeeInput: byTestId('recurring-payee-input'),
  recurringAmountInput: byTestId('recurring-amount-input'),
  recurringFreqSelect: byTestId('recurring-freq-select'),
  recurringSubmitBtn: byTestId('recurring-submit-btn'),
  recurringRow: byTestIdPrefix('recurring-row-'),
  recurringGenerateBtn: byTestId('recurring-generate-btn'),
  recurringToggle: byTestIdPrefix('recurring-toggle-'),

  // Rules
  ruleCreateBtn: byTestId('rule-create-btn'),
  ruleFieldSelect: byTestId('rule-field-select'),
  ruleMatchTypeSelect: byTestId('rule-match-type-select'),
  rulePatternInput: byTestId('rule-pattern-input'),
  ruleCategorySelect: byTestId('rule-category-select'),
  ruleSubmitBtn: byTestId('rule-submit-btn'),
  ruleTestBtn: byTestId('rule-test-btn'),
  ruleApplyAllBtn: byTestId('rule-apply-all-btn'),
  ruleRow: byTestIdPrefix('rule-row-'),

  // Tags
  tagPicker: byTestId('tag-picker'),
  tagInput: byTestId('tag-input'),
  tagCreateBtn: byTestId('tag-create-btn'),
  tagBadge: byTestIdPrefix('tag-badge-'),
  tagRemoveBtn: byTestIdPrefix('tag-remove-'),

  // Import
  importFileInput: byTestId('import-file-input'),
  importPreviewBtn: byTestId('import-preview-btn'),
  importMappingNext: byTestId('import-mapping-next'),
  importExecuteBtn: byTestId('import-execute-btn'),
  importResultSummary: byTestId('import-result-summary'),

  // Export
  exportBtn: byTestId('export-btn'),
  exportFormatSelect: byTestId('export-format-select'),
  exportDownloadBtn: byTestId('export-download-btn'),

  // Reports
  reportTypeSelect: byTestId('report-type-select'),
  reportDateRange: byTestId('report-date-range'),
  reportChart: byTestId('report-chart'),
  reportIncomeExpense: byTestId('report-income-expense'),
  reportCategorySpending: byTestId('report-category-spending'),
  reportNetWorth: byTestId('report-net-worth'),

  // Settings
  settingsModal: byTestId('settings-modal'),
  settingsCurrency: byTestId('settings-currency'),
  settingsTheme: byTestId('settings-theme'),
  settingsDateFormat: byTestId('settings-date-format'),

  // Reconciliation
  reconStartBtn: byTestId('recon-start-btn'),
  reconAccountSelect: byTestId('recon-account-select'),
  reconBalanceInput: byTestId('recon-balance-input'),
  reconDateInput: byTestId('recon-date-input'),

  // Reconciliation wizard (step 2+)
  reconDiffValue: '[class*="diffValue"]',
  reconCompleteBtn: 'button*=Complete Reconciliation',
  reconCancelBtn: 'button*=Cancel',
  reconDoneBtn: 'button*=Done',

  // Transfer
  transferFromAccount: byTestId('transfer-from-account'),
  transferToAccount: byTestId('transfer-to-account'),
  transferAmount: byTestId('transfer-amount'),
  transferSubmitBtn: byTestId('transfer-submit-btn'),

  // Dashboard
  dashboardSummary: byTestId('dashboard-summary'),
  dashboardIncomeCard: byTestId('dashboard-income-card'),
  dashboardExpenseCard: byTestId('dashboard-expenses-card'),
  dashboardNetCard: byTestId('dashboard-net-card'),
  dashboardChart: byTestId('dashboard-chart'),
  dashboardRecentTx: byTestId('dashboard-recent-tx'),
} as const;
