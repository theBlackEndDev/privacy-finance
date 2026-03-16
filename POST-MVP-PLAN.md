# PrivacyFinance — Post-MVP Implementation Plan (Phases 11-20)

*Comprehensive plan for extending PrivacyFinance from transaction tracker to full budgeting system.*

---

## Quick Reference

### MVP Recap
- **Phases 0-10:** 136 tasks, ~15-16 hours, 11 commit points
- **Delivered:** Account CRUD, transaction CRUD with transfers, categories, CSV import with dedup, dashboard with charts, keyboard shortcuts, dark/light themes, toast notifications, settings

### Post-MVP Scope
- **Phases 11-20:** 186 tasks, ~22-24 hours, 10 commit points
- **Delivers:** Recurring transactions, FTS5 search, data export + backup, envelope budgeting, auto-categorization rules, split transactions + tags, comprehensive reports, account reconciliation + multi-currency, accessibility (ARIA), encryption + auto-update + polish

### New Dependencies

**Cargo.toml additions (by phase):**
```toml
# Phase 18 — Multi-currency
reqwest = { version = "0.12", features = ["json", "rustls-tls"], optional = true }

# Phase 20 — Encryption + Auto-update
rusqlite = { version = "0.31", features = ["bundled-sqlcipher"] }  # replaces "bundled"
tauri-plugin-updater = "2"
```

**package.json additions (by phase):**
```json
// Phase 17 — Reports
"recharts": "already installed from MVP"

// Phase 19 — Accessibility
"@testing-library/react": "^14"  // for ARIA testing
```

### Key Technical Decisions

1. **Recurring Transactions:** "Check on launch" pattern — no background scheduler. On app start, scan recurring_transactions for any due dates that have passed, generate the missing transactions, advance next_date. This keeps the app fully offline with zero daemon processes.

2. **FTS5 Search:** SQLite FTS5 virtual table mirroring transaction payee + description. Triggers keep it in sync. This gives sub-millisecond full-text search with ranking, zero external dependencies.

3. **Envelope Budgeting:** YNAB-inspired — every dollar assigned to a category budget. Budget periods are monthly. Rollover is configurable per category. "Available" = budgeted - spent. No "credit card float" complexity for v1.

4. **Rules Engine:** Pattern matching on payee/description with regex support. Rules have priority ordering. Auto-apply on import AND on manual entry (if payee matches). User can override — rule suggestions shown but not forced.

5. **Split Transactions:** Parent transaction keeps the total amount. Child splits reference parent via parent_transaction_id. Each split has its own category + amount. Sum of splits must equal parent amount. Parent amount field becomes read-only when splits exist.

6. **Tags:** Second classification axis orthogonal to categories. Many-to-many via junction table. Tags have names + optional colors. Filterable in transaction list and reportable.

7. **Multi-Currency:** Manual exchange rate entry by default. Optional periodic fetch via reqwest (behind feature flag). All amounts stored in account's native currency. Cross-currency transfers store both amounts + rate.

8. **Encryption at Rest:** Switch from `rusqlite bundled` to `bundled-sqlcipher`. User sets password on first launch or can enable later. Key derivation via PBKDF2 (SQLCipher default). Unencrypted → encrypted migration copies data to new encrypted DB.

---

## New Database Schema

```sql
-- ============================================================
-- PHASE 11: RECURRING TRANSACTIONS
-- ============================================================
CREATE TABLE IF NOT EXISTS recurring_transactions (
  id TEXT PRIMARY KEY,
  account_id TEXT NOT NULL,
  payee TEXT NOT NULL DEFAULT '',
  description TEXT NOT NULL DEFAULT '',
  amount INTEGER NOT NULL,              -- cents
  kind TEXT NOT NULL CHECK(kind IN ('income', 'expense', 'transfer')),
  category_id TEXT,
  transfer_to_account_id TEXT,          -- for recurring transfers
  frequency TEXT NOT NULL CHECK(frequency IN ('daily', 'weekly', 'biweekly', 'monthly', 'quarterly', 'yearly')),
  start_date TEXT NOT NULL,             -- YYYY-MM-DD
  end_date TEXT,                        -- YYYY-MM-DD, NULL = no end
  next_date TEXT NOT NULL,              -- YYYY-MM-DD, next occurrence
  is_active INTEGER NOT NULL DEFAULT 1,
  auto_post INTEGER NOT NULL DEFAULT 0, -- 1 = auto-create, 0 = prompt user
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
  FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL,
  FOREIGN KEY (transfer_to_account_id) REFERENCES accounts(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_recurring_next_date ON recurring_transactions(next_date);
CREATE INDEX IF NOT EXISTS idx_recurring_account ON recurring_transactions(account_id);
CREATE INDEX IF NOT EXISTS idx_recurring_active ON recurring_transactions(is_active);

-- ============================================================
-- PHASE 12: FULL-TEXT SEARCH (FTS5)
-- ============================================================
CREATE VIRTUAL TABLE IF NOT EXISTS transactions_fts USING fts5(
  payee,
  description,
  content='transactions',
  content_rowid='rowid'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS transactions_fts_insert AFTER INSERT ON transactions BEGIN
  INSERT INTO transactions_fts(rowid, payee, description) VALUES (new.rowid, new.payee, new.description);
END;

CREATE TRIGGER IF NOT EXISTS transactions_fts_delete BEFORE DELETE ON transactions BEGIN
  INSERT INTO transactions_fts(transactions_fts, rowid, payee, description) VALUES('delete', old.rowid, old.payee, old.description);
END;

CREATE TRIGGER IF NOT EXISTS transactions_fts_update AFTER UPDATE ON transactions BEGIN
  INSERT INTO transactions_fts(transactions_fts, rowid, payee, description) VALUES('delete', old.rowid, old.payee, old.description);
  INSERT INTO transactions_fts(rowid, payee, description) VALUES (new.rowid, new.payee, new.description);
END;

-- ============================================================
-- PHASE 14: ENVELOPE BUDGETING
-- ============================================================
CREATE TABLE IF NOT EXISTS budgets (
  id TEXT PRIMARY KEY,
  year INTEGER NOT NULL,
  month INTEGER NOT NULL CHECK(month BETWEEN 1 AND 12),
  notes TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(year, month)
);

CREATE TABLE IF NOT EXISTS budget_items (
  id TEXT PRIMARY KEY,
  budget_id TEXT NOT NULL,
  category_id TEXT NOT NULL,
  budgeted INTEGER NOT NULL DEFAULT 0,  -- cents
  rollover INTEGER NOT NULL DEFAULT 0,  -- 1 = roll unused to next month
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (budget_id) REFERENCES budgets(id) ON DELETE CASCADE,
  FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE,
  UNIQUE(budget_id, category_id)
);

CREATE INDEX IF NOT EXISTS idx_budget_items_budget ON budget_items(budget_id);
CREATE INDEX IF NOT EXISTS idx_budget_items_category ON budget_items(category_id);
CREATE INDEX IF NOT EXISTS idx_budgets_year_month ON budgets(year, month);

-- ============================================================
-- PHASE 15: AUTO-CATEGORIZATION RULES
-- ============================================================
CREATE TABLE IF NOT EXISTS categorization_rules (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  field TEXT NOT NULL CHECK(field IN ('payee', 'description', 'amount')),
  match_type TEXT NOT NULL CHECK(match_type IN ('contains', 'starts_with', 'ends_with', 'exact', 'regex')),
  pattern TEXT NOT NULL,
  category_id TEXT NOT NULL,
  priority INTEGER NOT NULL DEFAULT 0,  -- higher = checked first
  is_active INTEGER NOT NULL DEFAULT 1,
  apply_count INTEGER NOT NULL DEFAULT 0,  -- times this rule has been applied
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_rules_priority ON categorization_rules(priority DESC);
CREATE INDEX IF NOT EXISTS idx_rules_active ON categorization_rules(is_active);

-- ============================================================
-- PHASE 16: SPLIT TRANSACTIONS + TAGS
-- ============================================================
CREATE TABLE IF NOT EXISTS transaction_splits (
  id TEXT PRIMARY KEY,
  transaction_id TEXT NOT NULL,
  category_id TEXT,
  amount INTEGER NOT NULL,              -- cents
  description TEXT NOT NULL DEFAULT '',
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE,
  FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_splits_transaction ON transaction_splits(transaction_id);

CREATE TABLE IF NOT EXISTS tags (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  color TEXT NOT NULL DEFAULT '#6b7280',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS transaction_tags (
  transaction_id TEXT NOT NULL,
  tag_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (transaction_id, tag_id),
  FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE,
  FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_transaction_tags_tag ON transaction_tags(tag_id);

-- ============================================================
-- PHASE 18: RECONCILIATION + MULTI-CURRENCY
-- ============================================================
CREATE TABLE IF NOT EXISTS reconciliations (
  id TEXT PRIMARY KEY,
  account_id TEXT NOT NULL,
  statement_date TEXT NOT NULL,         -- YYYY-MM-DD
  statement_balance INTEGER NOT NULL,   -- cents
  status TEXT NOT NULL CHECK(status IN ('in_progress', 'completed')) DEFAULT 'in_progress',
  completed_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_reconciliations_account ON reconciliations(account_id);

CREATE TABLE IF NOT EXISTS reconciliation_items (
  reconciliation_id TEXT NOT NULL,
  transaction_id TEXT NOT NULL,
  is_reconciled INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (reconciliation_id, transaction_id),
  FOREIGN KEY (reconciliation_id) REFERENCES reconciliations(id) ON DELETE CASCADE,
  FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS exchange_rates (
  id TEXT PRIMARY KEY,
  from_currency TEXT NOT NULL,
  to_currency TEXT NOT NULL,
  rate REAL NOT NULL,                   -- 1 from_currency = rate to_currency
  effective_date TEXT NOT NULL,         -- YYYY-MM-DD
  source TEXT NOT NULL DEFAULT 'manual', -- 'manual' or 'api'
  created_at TEXT NOT NULL,
  UNIQUE(from_currency, to_currency, effective_date)
);

CREATE INDEX IF NOT EXISTS idx_exchange_rates_pair ON exchange_rates(from_currency, to_currency);
CREATE INDEX IF NOT EXISTS idx_exchange_rates_date ON exchange_rates(effective_date);
```

---

## Architecture Additions

### New Rust Files

```
src-tauri/src/
├── models/
│   ├── recurring.rs        -- Phase 11
│   ├── budget.rs           -- Phase 14
│   ├── rule.rs             -- Phase 15
│   ├── split.rs            -- Phase 16
│   ├── tag.rs              -- Phase 16
│   ├── report.rs           -- Phase 17
│   ├── reconciliation.rs   -- Phase 18
│   └── exchange_rate.rs    -- Phase 18
├── commands/
│   ├── recurring.rs        -- Phase 11
│   ├── search.rs           -- Phase 12
│   ├── export.rs           -- Phase 13
│   ├── backup.rs           -- Phase 13
│   ├── budgets.rs          -- Phase 14
│   ├── rules.rs            -- Phase 15
│   ├── splits.rs           -- Phase 16
│   ├── tags.rs             -- Phase 16
│   ├── reports.rs          -- Phase 17
│   ├── reconciliation.rs   -- Phase 18
│   ├── exchange_rates.rs   -- Phase 18
│   └── encryption.rs       -- Phase 20
```

### New Frontend Files

```
src/
├── api/
│   ├── recurring.ts        -- Phase 11
│   ├── search.ts           -- Phase 12
│   ├── export.ts           -- Phase 13
│   ├── backup.ts           -- Phase 13
│   ├── budgets.ts          -- Phase 14
│   ├── rules.ts            -- Phase 15
│   ├── splits.ts           -- Phase 16
│   ├── tags.ts             -- Phase 16
│   ├── reports.ts          -- Phase 17
│   ├── reconciliation.ts   -- Phase 18
│   └── exchange_rates.ts   -- Phase 18
├── views/
│   ├── RecurringPage.tsx       -- Phase 11 (Ctrl+6)
│   ├── BudgetsPage.tsx         -- Phase 14 (Ctrl+B)
│   └── ReportsPage.tsx         -- Phase 17 (Ctrl+7)
├── components/
│   ├── recurring/
│   │   ├── RecurringList.tsx        -- Phase 11
│   │   ├── RecurringForm.tsx        -- Phase 11
│   │   └── RecurringDueModal.tsx    -- Phase 11
│   ├── search/
│   │   └── GlobalSearch.tsx         -- Phase 12
│   ├── export/
│   │   └── ExportModal.tsx          -- Phase 13
│   ├── budgets/
│   │   ├── BudgetGrid.tsx           -- Phase 14
│   │   ├── BudgetCategoryRow.tsx    -- Phase 14
│   │   ├── BudgetMonthNav.tsx       -- Phase 14
│   │   └── BudgetProgressBar.tsx    -- Phase 14
│   ├── rules/
│   │   ├── RulesList.tsx            -- Phase 15
│   │   ├── RuleForm.tsx             -- Phase 15
│   │   └── RuleTestModal.tsx        -- Phase 15
│   ├── splits/
│   │   └── SplitEditor.tsx          -- Phase 16
│   ├── tags/
│   │   ├── TagBadge.tsx             -- Phase 16
│   │   ├── TagPicker.tsx            -- Phase 16
│   │   └── TagManager.tsx           -- Phase 16
│   ├── reports/
│   │   ├── IncomeExpenseReport.tsx   -- Phase 17
│   │   ├── CategorySpendingReport.tsx-- Phase 17
│   │   ├── CashFlowReport.tsx       -- Phase 17
│   │   ├── NetWorthReport.tsx        -- Phase 17
│   │   ├── ReportDateRange.tsx       -- Phase 17
│   │   └── ReportExportButton.tsx    -- Phase 17
│   └── reconciliation/
│       ├── ReconciliationWizard.tsx  -- Phase 18
│       └── ReconciliationRow.tsx     -- Phase 18
```

---

## Build Order — Granular Task Breakdown

Each task is designed to be completable in 10-30 minutes and produce a working, saveable artifact. Commit after each phase completes. If a session limit hits mid-phase, at most one small task is lost.

**Dependency Legend:**
- → = must complete before
- ‖ = can run concurrently
- Tasks with no listed dependency depend only on their phase's prior tasks completing

**Recommended execution order:** 11 → 12 → 13 → 15 → 14 → 16 → 17 → 18 → 19 → 20
(Phases numbered sequentially but build order prioritizes user value — rules before budgets, search before export.)

---

### Phase 11: Recurring Transactions

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 11.1 | Write models/recurring.rs — RecurringTransaction struct, CreateRecurringInput, UpdateRecurringInput, Frequency enum with serde derives | Phase 10 | 10 min |
| 11.2 | Write commands/recurring.rs — create_recurring, get_recurring, get_all_recurring | 11.1 | 15 min |
| 11.3 | Write commands/recurring.rs — update_recurring, delete_recurring | 11.1 | 10 min |
| 11.4 | Write commands/recurring.rs — toggle_recurring_active (pause/resume) | 11.1 | 5 min |
| 11.5 | Write commands/recurring.rs — calculate_next_date helper: given frequency + current date, return next occurrence. Handle month boundaries (Jan 31 monthly → Feb 28), leap years | 11.1 | 15 min |
| 11.6 | Write commands/recurring.rs — check_due_recurring: scan all active recurring where next_date <= today, return list of due items with count of missed occurrences | 11.5 | 15 min |
| 11.7 | Write commands/recurring.rs — generate_recurring_transactions: for each due recurring, create transaction(s) for all missed dates, advance next_date. Wrap in DB transaction. Return count generated. | 11.5, 11.6 | 20 min |
| 11.8 | Add recurring_transactions CREATE TABLE + indexes to db/migrations.rs | Phase 10 | 10 min |
| 11.9 | Register all recurring commands in lib.rs invoke_handler | 11.2, 11.3, 11.4, 11.7 | 5 min |
| 11.10 | Run cargo check | 11.9 | 5 min |
| 11.11 | Write recurring CRUD tests — create, read, update, delete, toggle active | 11.2, 11.3, 11.4 | 15 min |
| 11.12 | Write calculate_next_date tests — daily, weekly, biweekly, monthly (with 31st edge), quarterly, yearly, leap year | 11.5 | 15 min |
| 11.13 | Write generation tests — due items generate correct count, next_date advances, end_date respected, auto_post vs manual | 11.7 | 15 min |
| 11.14 | Add Recurring types to types/index.ts — RecurringTransaction, CreateRecurringInput, Frequency | Phase 10 | 10 min |
| 11.15 | Write api/recurring.ts — invoke() wrappers for all recurring commands | 11.14 | 10 min |
| 11.16 | Write components/recurring/RecurringList.tsx + .module.css — table of recurring items: payee, amount, frequency, next date, active toggle, edit/delete | 11.15 | 15 min |
| 11.17 | Write components/recurring/RecurringForm.tsx + .module.css — create/edit form: account, payee, amount, kind, category, frequency, start date, end date, auto-post toggle | 11.15 | 20 min |
| 11.18 | Write components/recurring/RecurringDueModal.tsx + .module.css — on app launch, if due items exist, show modal listing them with "Generate All" and "Review Each" buttons | 11.15 | 15 min |
| 11.19 | Write views/RecurringPage.tsx — composes RecurringList + RecurringForm modal | 11.16, 11.17 | 10 min |
| 11.20 | Wire RecurringPage into App.tsx view switch + add Sidebar nav entry (Ctrl+6) + call check_due_recurring on app mount | 11.18, 11.19 | 10 min |
| 11.21 | Run cargo test | 11.11, 11.12, 11.13 | 5 min |
| 11.22 | Verify: create recurring, see it in list, wait for due date simulation, generate transactions, verify they appear in transaction list | 11.20, 11.21 | 10 min |

*Concurrency: 11.1 ‖ 11.8 ‖ 11.14 (independent). Then 11.2 ‖ 11.3 ‖ 11.4 ‖ 11.5 (after 11.1). Then 11.6 → 11.7 (after 11.5). Then 11.9 → 11.10. Tests: 11.11 ‖ 11.12 ‖ 11.13. Frontend: 11.15 → 11.16 ‖ 11.17 ‖ 11.18 → 11.19 → 11.20. Final: 11.21 → 11.22.*

*Commit point: After 11.22 verified — recurring transactions generate correctly on launch.*

---

### Phase 12: Full-Text Search (FTS5)

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 12.1 | Add FTS5 virtual table + sync triggers to db/migrations.rs | Phase 11 | 10 min |
| 12.2 | Write migration logic to populate FTS5 from existing transactions (for upgrades from MVP) | 12.1 | 10 min |
| 12.3 | Write commands/search.rs — search_transactions: FTS5 MATCH query with BM25 ranking, returns transaction IDs ordered by relevance, limit + offset | 12.1 | 15 min |
| 12.4 | Write commands/search.rs — search_transactions_full: join FTS results with transactions table to return full Transaction structs with highlight snippets | 12.3 | 15 min |
| 12.5 | Write commands/search.rs — search_suggest: FTS5 prefix query for autocomplete suggestions (top 5 matches) | 12.3 | 10 min |
| 12.6 | Register search commands in lib.rs | 12.4, 12.5 | 5 min |
| 12.7 | Run cargo check | 12.6 | 5 min |
| 12.8 | Write FTS5 tests — insert transaction triggers FTS update, search returns correct results, delete removes from FTS, update re-indexes | 12.4 | 15 min |
| 12.9 | Write search ranking tests — exact match ranks higher than partial, multi-word queries work | 12.4 | 10 min |
| 12.10 | Add Search types to types/index.ts — SearchResult, SearchSuggestion | Phase 11 | 5 min |
| 12.11 | Write api/search.ts — invoke() wrappers | 12.10 | 5 min |
| 12.12 | Write components/search/GlobalSearch.tsx + .module.css — search input in Header with dropdown results, keyboard navigation (up/down/enter), click to navigate to transaction | 12.11 | 20 min |
| 12.13 | Wire GlobalSearch into Header.tsx, replace placeholder search with FTS5 powered search | 12.12 | 10 min |
| 12.14 | Update Ctrl+F shortcut to focus GlobalSearch | 12.13 | 5 min |
| 12.15 | Run cargo test | 12.8, 12.9 | 5 min |
| 12.16 | Verify: type in search, see ranked results, click result navigates to transaction, Ctrl+F focuses search | 12.14, 12.15 | 10 min |

*Concurrency: 12.1 ‖ 12.10 (independent). Then 12.2 ‖ 12.3 (after 12.1). Then 12.4 ‖ 12.5 (after 12.3). Then 12.6. Tests: 12.8 ‖ 12.9. Frontend: 12.11 → 12.12 → 12.13 → 12.14. Final: 12.15 → 12.16.*

*Commit point: After 12.16 verified — FTS5 search works end-to-end.*

---

### Phase 13: Data Export + Backup/Restore

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 13.1 | Write commands/export.rs — export_transactions_csv: query transactions with filters, write CSV to user-chosen path via Tauri dialog. Columns: date, account, payee, description, category, amount, kind, cleared | Phase 12 | 15 min |
| 13.2 | Write commands/export.rs — export_transactions_json: same query, write as JSON array with pretty print | 13.1 | 10 min |
| 13.3 | Write commands/export.rs — export_all_data_json: complete database dump — accounts, categories, transactions, recurring, budgets, rules, tags — as structured JSON | 13.1 | 15 min |
| 13.4 | Write commands/backup.rs — create_backup: copy SQLite file to user-chosen path with timestamp suffix, verify integrity with PRAGMA integrity_check | Phase 12 | 15 min |
| 13.5 | Write commands/backup.rs — restore_backup: validate backup file (PRAGMA integrity_check), close current connections, replace DB file, reopen connections, verify schema version | 13.4 | 20 min |
| 13.6 | Write commands/backup.rs — get_backup_info: read metadata from backup file (table counts, date range of transactions, file size) | 13.4 | 10 min |
| 13.7 | Register export + backup commands in lib.rs | 13.3, 13.6 | 5 min |
| 13.8 | Run cargo check | 13.7 | 5 min |
| 13.9 | Write export tests — CSV output matches expected format, JSON is valid, filters applied correctly | 13.1, 13.2 | 15 min |
| 13.10 | Write backup tests — backup creates valid copy, restore replaces DB, integrity check catches corruption | 13.5 | 15 min |
| 13.11 | Add Export/Backup types to types/index.ts — ExportFormat, ExportFilters, BackupInfo | Phase 12 | 5 min |
| 13.12 | Write api/export.ts + api/backup.ts — invoke() wrappers | 13.11 | 10 min |
| 13.13 | Write components/export/ExportModal.tsx + .module.css — format selector (CSV/JSON/Full JSON), date range filter, account filter, "Export" button with Tauri save dialog | 13.12 | 15 min |
| 13.14 | Write backup UI in Settings section — "Create Backup" button, "Restore from Backup" button with file picker, backup info display, confirm dialog for restore | 13.12 | 15 min |
| 13.15 | Add export button to TransactionsPage header + wire ExportModal | 13.13 | 5 min |
| 13.16 | Run cargo test | 13.9, 13.10 | 5 min |
| 13.17 | Verify: export CSV opens in spreadsheet app, JSON is valid, backup creates file, restore replaces data correctly | 13.15, 13.16 | 10 min |

*Concurrency: 13.1 ‖ 13.4 ‖ 13.11 (independent). Then 13.2 ‖ 13.3 (after 13.1). 13.5 ‖ 13.6 (after 13.4). Then 13.7. Tests: 13.9 ‖ 13.10. Frontend: 13.12 → 13.13 ‖ 13.14 → 13.15. Final: 13.16 → 13.17.*

*Commit point: After 13.17 verified — export + backup/restore works end-to-end.*

---

### Phase 14: Envelope Budgeting

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 14.1 | Write models/budget.rs — Budget, BudgetItem, BudgetSummary, BudgetCategoryRow structs. BudgetCategoryRow includes: category name/color, budgeted, spent (actual), available (budgeted - spent), rollover flag | Phase 13 | 15 min |
| 14.2 | Add budgets + budget_items CREATE TABLE + indexes to db/migrations.rs | Phase 13 | 10 min |
| 14.3 | Write commands/budgets.rs — create_budget: create budget for year/month, auto-create budget_items for all active expense categories with 0 budgeted | 14.1, 14.2 | 15 min |
| 14.4 | Write commands/budgets.rs — get_budget: return budget with all items joined to categories, include "spent" (SUM of expenses in that category for that month), calculate "available" | 14.1 | 20 min |
| 14.5 | Write commands/budgets.rs — update_budget_item: set budgeted amount for a single category, toggle rollover | 14.1 | 10 min |
| 14.6 | Write commands/budgets.rs — copy_budget: clone previous month's allocations to current month as starting point | 14.3 | 10 min |
| 14.7 | Write commands/budgets.rs — get_budget_summary: total budgeted, total spent, total available, unbudgeted income, "to be budgeted" (income - total budgeted) | 14.4 | 15 min |
| 14.8 | Write commands/budgets.rs — calculate_rollover: for each category with rollover=1, carry forward (budgeted - spent) from previous month. Negative rollover reduces next month's available. | 14.4 | 15 min |
| 14.9 | Register budget commands in lib.rs | 14.3, 14.4, 14.5, 14.6, 14.7, 14.8 | 5 min |
| 14.10 | Run cargo check | 14.9 | 5 min |
| 14.11 | Write budget CRUD tests — create budget, verify items for all categories, update allocation, verify available = budgeted - spent | 14.4, 14.5 | 15 min |
| 14.12 | Write rollover tests — positive rollover carries forward, negative rollover reduces next month, no rollover when flag is off | 14.8 | 15 min |
| 14.13 | Write copy budget tests — previous month's allocations copied correctly, categories added since last month get 0 | 14.6 | 10 min |
| 14.14 | Add Budget types to types/index.ts — Budget, BudgetItem, BudgetSummary, BudgetCategoryRow | Phase 13 | 10 min |
| 14.15 | Write api/budgets.ts — invoke() wrappers | 14.14 | 10 min |
| 14.16 | Write components/budgets/BudgetMonthNav.tsx + .module.css — prev/next month buttons, current month/year display, "Copy from last month" button | 14.15 | 10 min |
| 14.17 | Write components/budgets/BudgetProgressBar.tsx + .module.css — horizontal bar showing spent/budgeted ratio, color changes: green (<80%), yellow (80-100%), red (>100%) | none | 10 min |
| 14.18 | Write components/budgets/BudgetCategoryRow.tsx + .module.css — category name+color, editable budgeted amount input, spent (read-only), available (calculated), progress bar, rollover toggle | 14.15, 14.17 | 15 min |
| 14.19 | Write components/budgets/BudgetGrid.tsx + .module.css — header row (Category, Budgeted, Spent, Available), list of BudgetCategoryRows, summary row at bottom (totals), "To Be Budgeted" banner at top | 14.16, 14.18 | 20 min |
| 14.20 | Write views/BudgetsPage.tsx — composes BudgetMonthNav + BudgetGrid, auto-creates budget for current month if none exists | 14.19 | 10 min |
| 14.21 | Wire BudgetsPage into App.tsx + Sidebar nav entry (Ctrl+B) | 14.20 | 5 min |
| 14.22 | Run cargo test | 14.11, 14.12, 14.13 | 5 min |
| 14.23 | Verify: navigate to budgets, see all categories, set allocations, see spent update when transactions added, rollover works across months | 14.21, 14.22 | 10 min |

*Concurrency: 14.1 ‖ 14.2 ‖ 14.14 (independent). Then 14.3 ‖ 14.4 ‖ 14.5 (after 14.1 + 14.2). Then 14.6 ‖ 14.7 ‖ 14.8 (after 14.4). Then 14.9. Tests: 14.11 ‖ 14.12 ‖ 14.13. Frontend: 14.15 → 14.16 ‖ 14.17 ‖ 14.18 → 14.19 → 14.20 → 14.21. Final: 14.22 → 14.23.*

*Commit point: After 14.23 verified — envelope budgeting tracks spending vs allocations.*

---

### Phase 15: Auto-Categorization Rules

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 15.1 | Write models/rule.rs — CategorizationRule struct, CreateRuleInput, MatchType enum (Contains, StartsWith, EndsWith, Exact, Regex), MatchField enum (Payee, Description, Amount) | Phase 14 | 10 min |
| 15.2 | Add categorization_rules CREATE TABLE + indexes to db/migrations.rs | Phase 14 | 10 min |
| 15.3 | Write commands/rules.rs — create_rule, get_all_rules, update_rule, delete_rule | 15.1, 15.2 | 15 min |
| 15.4 | Write commands/rules.rs — reorder_rules: update priority values for a list of rule IDs | 15.1 | 10 min |
| 15.5 | Write commands/rules.rs — match_transaction: given a transaction's payee/description/amount, iterate rules by priority, return first matching category_id (or None). Regex compiled with timeout guard. | 15.1 | 15 min |
| 15.6 | Write commands/rules.rs — apply_rules_to_uncategorized: find all transactions where category_id IS NULL, run match_transaction on each, update matches. Return count updated. | 15.5 | 15 min |
| 15.7 | Write commands/rules.rs — test_rule: given a rule pattern + match_type, query transactions and return preview of what would match (first 20 results) without modifying anything | 15.5 | 10 min |
| 15.8 | Integrate rules into import flow: after CSV import completes, auto-run match_transaction on each newly imported transaction that has no category | 15.5 | 10 min |
| 15.9 | Integrate rules into transaction create: when user creates transaction with no category, suggest category from rules (show as hint, not auto-apply) | 15.5 | 10 min |
| 15.10 | Register rules commands in lib.rs | 15.3, 15.4, 15.6, 15.7 | 5 min |
| 15.11 | Run cargo check | 15.10 | 5 min |
| 15.12 | Write rules CRUD tests — create, read, update, delete, reorder priority | 15.3, 15.4 | 10 min |
| 15.13 | Write pattern matching tests — contains, starts_with, ends_with, exact, regex. Test priority ordering. Test regex timeout for catastrophic backtracking. | 15.5 | 15 min |
| 15.14 | Write auto-apply tests — uncategorized transactions get categorized, already-categorized transactions untouched | 15.6 | 10 min |
| 15.15 | Add Rule types to types/index.ts — CategorizationRule, CreateRuleInput, MatchType, RuleTestResult | Phase 14 | 10 min |
| 15.16 | Write api/rules.ts — invoke() wrappers | 15.15 | 5 min |
| 15.17 | Write components/rules/RulesList.tsx + .module.css — drag-to-reorder list of rules showing: name, pattern, match type, target category, apply count, active toggle, edit/delete | 15.16 | 20 min |
| 15.18 | Write components/rules/RuleForm.tsx + .module.css — create/edit: name, field (payee/description/amount), match type dropdown, pattern input, category selector, priority auto-set | 15.16 | 15 min |
| 15.19 | Write components/rules/RuleTestModal.tsx + .module.css — shows preview of transactions that would match current rule pattern, count of matches, "Apply" button | 15.16 | 15 min |
| 15.20 | Add rules management to Settings or dedicated section in Sidebar, compose RulesList + RuleForm + RuleTestModal | 15.17, 15.18, 15.19 | 10 min |
| 15.21 | Run cargo test | 15.12, 15.13, 15.14 | 5 min |
| 15.22 | Verify: create rule for "NETFLIX" → Subscriptions, import CSV with Netflix entries, verify auto-categorized, test rule preview shows correct matches | 15.20, 15.21 | 10 min |

*Concurrency: 15.1 ‖ 15.2 ‖ 15.15 (independent). Then 15.3 ‖ 15.4 ‖ 15.5 (after 15.1 + 15.2). Then 15.6 ‖ 15.7 ‖ 15.8 ‖ 15.9 (after 15.5). Then 15.10. Tests: 15.12 ‖ 15.13 ‖ 15.14. Frontend: 15.16 → 15.17 ‖ 15.18 ‖ 15.19 → 15.20. Final: 15.21 → 15.22.*

*Commit point: After 15.22 verified — rules auto-categorize transactions on import.*

---

### Phase 16: Split Transactions + Tags

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 16.1 | Write models/split.rs — TransactionSplit struct, CreateSplitInput. Splits reference parent transaction_id. | Phase 15 | 10 min |
| 16.2 | Write models/tag.rs — Tag struct, CreateTagInput, TransactionTag junction struct | Phase 15 | 10 min |
| 16.3 | Add transaction_splits, tags, transaction_tags CREATE TABLE + indexes to db/migrations.rs | Phase 15 | 10 min |
| 16.4 | Write commands/splits.rs — create_splits: given transaction_id + list of {category_id, amount, description}, validate sum equals parent amount, insert splits, mark parent as split | 16.1, 16.3 | 15 min |
| 16.5 | Write commands/splits.rs — get_splits: return all splits for a transaction_id | 16.1 | 5 min |
| 16.6 | Write commands/splits.rs — update_splits: replace all splits for a transaction (delete old, insert new, re-validate sum) | 16.4 | 10 min |
| 16.7 | Write commands/splits.rs — delete_splits: remove all splits, un-mark parent as split | 16.4 | 5 min |
| 16.8 | Update commands/dashboard.rs — budget and category reports use split categories when splits exist (not parent category) | 16.4 | 10 min |
| 16.9 | Write commands/tags.rs — create_tag, get_all_tags, update_tag, delete_tag | 16.2, 16.3 | 10 min |
| 16.10 | Write commands/tags.rs — tag_transaction, untag_transaction, get_transaction_tags | 16.2 | 10 min |
| 16.11 | Write commands/tags.rs — get_transactions_by_tag: return all transactions with a given tag_id | 16.10 | 10 min |
| 16.12 | Update commands/transactions.rs — include tags in transaction query results (LEFT JOIN through junction) | 16.10 | 10 min |
| 16.13 | Register split + tag commands in lib.rs | 16.6, 16.7, 16.9, 16.11 | 5 min |
| 16.14 | Run cargo check | 16.13 | 5 min |
| 16.15 | Write split tests — create splits summing to parent, reject mismatched sum, delete restores parent, dashboard uses split categories | 16.4, 16.8 | 15 min |
| 16.16 | Write tag tests — CRUD, tag/untag transaction, query by tag, delete tag removes junction entries | 16.9, 16.10, 16.11 | 15 min |
| 16.17 | Add Split + Tag types to types/index.ts | Phase 15 | 10 min |
| 16.18 | Write api/splits.ts + api/tags.ts — invoke() wrappers | 16.17 | 10 min |
| 16.19 | Write components/splits/SplitEditor.tsx + .module.css — inline split editor in TransactionEditModal: add split rows (category + amount + description), running total vs parent, validation indicator | 16.18 | 20 min |
| 16.20 | Integrate SplitEditor into TransactionEditModal — "Split" button toggles split mode, hides parent category when splits active | 16.19 | 10 min |
| 16.21 | Write components/tags/TagBadge.tsx + .module.css — small colored pill showing tag name | 16.18 | 5 min |
| 16.22 | Write components/tags/TagPicker.tsx + .module.css — dropdown with checkboxes for multi-select, create-new inline input | 16.18 | 15 min |
| 16.23 | Write components/tags/TagManager.tsx + .module.css — CRUD list of all tags with color picker, rename, delete | 16.18 | 15 min |
| 16.24 | Integrate TagPicker into TransactionForm + TransactionEditModal | 16.22 | 10 min |
| 16.25 | Add tag filter to TransactionFilters.tsx — multi-select tag filter | 16.22, 16.12 | 10 min |
| 16.26 | Show TagBadges in TransactionRow.tsx | 16.21, 16.12 | 5 min |
| 16.27 | Run cargo test | 16.15, 16.16 | 5 min |
| 16.28 | Verify: split a transaction into 3 categories, verify budget reflects splits, create tags, tag transactions, filter by tag | 16.20, 16.24, 16.25, 16.26, 16.27 | 10 min |

*Concurrency: 16.1 ‖ 16.2 ‖ 16.3 ‖ 16.17 (independent). Splits: 16.4 → 16.5 ‖ 16.6 ‖ 16.7 → 16.8. Tags: 16.9 ‖ 16.10 → 16.11 → 16.12. Then 16.13. Tests: 16.15 ‖ 16.16. Frontend: 16.18 → 16.19 ‖ 16.21 ‖ 16.22 ‖ 16.23 → 16.20 ‖ 16.24 ‖ 16.25 ‖ 16.26. Final: 16.27 → 16.28.*

*Commit point: After 16.28 verified — splits + tags work end-to-end.*

---

### Phase 17: Comprehensive Reports

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 17.1 | Write models/report.rs — ReportDateRange, IncomeExpenseReport, CategorySpendingReport, CashFlowReport, NetWorthReport, NetWorthPoint structs | Phase 16 | 10 min |
| 17.2 | Write commands/reports.rs — get_income_expense_report: income vs expense by month for date range, with delta vs previous period. Uses split categories when available. | 17.1 | 20 min |
| 17.3 | Write commands/reports.rs — get_category_spending_report: spending breakdown by category for date range, with percentage of total, sorted by amount. Respects splits. | 17.1 | 15 min |
| 17.4 | Write commands/reports.rs — get_cash_flow_report: daily/weekly/monthly cash flow (income - expense) for date range, running cumulative total | 17.1 | 15 min |
| 17.5 | Write commands/reports.rs — get_net_worth_report: account balances over time (monthly snapshots), calculated from opening_balance + cumulative transactions. Returns time series for chart. | 17.1 | 20 min |
| 17.6 | Write commands/reports.rs — get_tag_spending_report: spending grouped by tag for date range | 17.1 | 10 min |
| 17.7 | Write commands/reports.rs — get_payee_spending_report: top N payees by total amount for date range | 17.1 | 10 min |
| 17.8 | Register report commands in lib.rs | 17.2, 17.3, 17.4, 17.5, 17.6, 17.7 | 5 min |
| 17.9 | Run cargo check | 17.8 | 5 min |
| 17.10 | Write report tests — income/expense totals match, category percentages sum to 100%, net worth calculation correct, cash flow cumulative correct | 17.2, 17.3, 17.4, 17.5 | 20 min |
| 17.11 | Add Report types to types/index.ts | Phase 16 | 10 min |
| 17.12 | Write api/reports.ts — invoke() wrappers | 17.11 | 10 min |
| 17.13 | Write components/reports/ReportDateRange.tsx + .module.css — date range picker with presets (This Month, Last Month, This Quarter, This Year, Last 6 Months, Custom) | 17.12 | 15 min |
| 17.14 | Write components/reports/IncomeExpenseReport.tsx + .module.css — stacked bar chart (Recharts) income vs expense by month, delta comparison row showing +/-% vs previous period | 17.12, 17.13 | 20 min |
| 17.15 | Write components/reports/CategorySpendingReport.tsx + .module.css — horizontal bar chart showing spending by category, sorted descending, percentage labels, category colors | 17.12, 17.13 | 15 min |
| 17.16 | Write components/reports/CashFlowReport.tsx + .module.css — area chart showing cumulative cash flow over time, zero line highlighted | 17.12, 17.13 | 15 min |
| 17.17 | Write components/reports/NetWorthReport.tsx + .module.css — line chart showing net worth over time, per-account area breakdown option | 17.12, 17.13 | 15 min |
| 17.18 | Write components/reports/ReportExportButton.tsx — export current report view as CSV or JSON | 17.12 | 10 min |
| 17.19 | Write views/ReportsPage.tsx — tab nav for report types (Income/Expense, Categories, Cash Flow, Net Worth, Tags, Payees), each tab renders its report component with shared ReportDateRange | 17.14, 17.15, 17.16, 17.17, 17.18 | 15 min |
| 17.20 | Wire ReportsPage into App.tsx + Sidebar nav entry (Ctrl+7) | 17.19 | 5 min |
| 17.21 | Run cargo test | 17.10 | 5 min |
| 17.22 | Verify: each report renders with test data, date range filtering works, charts display correctly, export produces valid files | 17.20, 17.21 | 15 min |

*Concurrency: 17.1 ‖ 17.11 (independent). Then 17.2 ‖ 17.3 ‖ 17.4 ‖ 17.5 ‖ 17.6 ‖ 17.7 (all after 17.1). Then 17.8. Frontend: 17.12 → 17.13 → 17.14 ‖ 17.15 ‖ 17.16 ‖ 17.17 ‖ 17.18 → 17.19 → 17.20. Final: 17.21 → 17.22.*

*Commit point: After 17.22 verified — all reports render with correct data.*

---

### Phase 18: Account Reconciliation + Multi-Currency

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 18.1 | Write models/reconciliation.rs — Reconciliation, ReconciliationItem, ReconciliationSummary structs | Phase 17 | 10 min |
| 18.2 | Write models/exchange_rate.rs — ExchangeRate struct, CreateExchangeRateInput | Phase 17 | 10 min |
| 18.3 | Add reconciliations, reconciliation_items, exchange_rates CREATE TABLE + indexes to db/migrations.rs | Phase 17 | 10 min |
| 18.4 | Write commands/reconciliation.rs — start_reconciliation: create reconciliation with account_id + statement_date + statement_balance, load unreconciled transactions for that account up to statement_date | 18.1, 18.3 | 15 min |
| 18.5 | Write commands/reconciliation.rs — toggle_reconciliation_item: mark/unmark a transaction as reconciled within the session | 18.1 | 10 min |
| 18.6 | Write commands/reconciliation.rs — get_reconciliation_diff: calculated balance (opening + reconciled transactions) vs statement balance, show difference | 18.4 | 10 min |
| 18.7 | Write commands/reconciliation.rs — complete_reconciliation: verify diff is zero, mark all reconciled transactions as is_cleared=1, set reconciliation status to completed | 18.6 | 10 min |
| 18.8 | Write commands/reconciliation.rs — cancel_reconciliation: delete reconciliation record, leave transactions unchanged | 18.1 | 5 min |
| 18.9 | Write commands/exchange_rates.rs — create_rate, get_rate (nearest date for currency pair), get_all_rates, delete_rate | 18.2, 18.3 | 15 min |
| 18.10 | Write commands/exchange_rates.rs — convert_amount: given amount + from_currency + to_currency + date, find nearest rate and convert | 18.9 | 10 min |
| 18.11 | Update commands/transactions.rs — cross-currency transfer support: store original_amount + original_currency + exchange_rate on transaction when currencies differ | 18.10 | 15 min |
| 18.12 | Update commands/dashboard.rs — multi-currency account balances: show native currency balance + converted-to-primary balance | 18.10 | 10 min |
| 18.13 | Register reconciliation + exchange rate commands in lib.rs | 18.7, 18.8, 18.9 | 5 min |
| 18.14 | Run cargo check | 18.13 | 5 min |
| 18.15 | Write reconciliation tests — start session, toggle items, verify diff calculation, complete when balanced, reject when unbalanced, cancel cleans up | 18.7, 18.8 | 15 min |
| 18.16 | Write exchange rate tests — CRUD, nearest-date lookup, conversion accuracy, cross-currency transfer stores both amounts | 18.10, 18.11 | 15 min |
| 18.17 | Add Reconciliation + ExchangeRate types to types/index.ts | Phase 17 | 10 min |
| 18.18 | Write api/reconciliation.ts + api/exchange_rates.ts — invoke() wrappers | 18.17 | 10 min |
| 18.19 | Write components/reconciliation/ReconciliationWizard.tsx + .module.css — step 1: enter statement date + balance. Step 2: show transactions with checkboxes, running diff at top (green when zero). Step 3: confirm + complete. | 18.18 | 25 min |
| 18.20 | Write components/reconciliation/ReconciliationRow.tsx + .module.css — transaction row with reconcile checkbox, amount, running total indicator | 18.18 | 10 min |
| 18.21 | Add "Reconcile" button to account actions in AccountList/Sidebar | 18.19, 18.20 | 5 min |
| 18.22 | Add exchange rate management to Settings — currency pair input, rate input, date, list of saved rates | 18.18 | 15 min |
| 18.23 | Run cargo test | 18.15, 18.16 | 5 min |
| 18.24 | Verify: start reconciliation, check off transactions until diff is zero, complete, verify cleared flags set. Enter exchange rate, create cross-currency transfer, verify conversion. | 18.21, 18.22, 18.23 | 10 min |

*Concurrency: 18.1 ‖ 18.2 ‖ 18.3 ‖ 18.17 (independent). Reconciliation: 18.4 → 18.5 ‖ 18.6 → 18.7 ‖ 18.8. Currency: 18.9 → 18.10 → 18.11 ‖ 18.12. Then 18.13. Tests: 18.15 ‖ 18.16. Frontend: 18.18 → 18.19 ‖ 18.20 ‖ 18.22 → 18.21. Final: 18.23 → 18.24.*

*Commit point: After 18.24 verified — reconciliation + multi-currency work end-to-end.*

---

### Phase 19: Accessibility (ARIA)

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 19.1 | Audit all interactive components — identify missing ARIA roles, labels, states. Create checklist of fixes per component. | Phase 18 | 15 min |
| 19.2 | Add ARIA landmarks to layout — Sidebar: role="navigation" + aria-label, Main content: role="main", Header: role="banner" | 19.1 | 10 min |
| 19.3 | Fix Button.tsx — aria-label for icon-only buttons, aria-disabled, aria-pressed for toggle buttons | 19.1 | 10 min |
| 19.4 | Fix Modal.tsx — role="dialog", aria-modal="true", aria-labelledby pointing to title, focus trap with Tab/Shift+Tab, return focus on close | 19.1 | 15 min |
| 19.5 | Fix Toast.tsx — role="alert", aria-live="polite" for success, aria-live="assertive" for errors | 19.1 | 5 min |
| 19.6 | Fix TransactionList.tsx — role="table", proper th/td markup, aria-sort on sortable columns, aria-label for action buttons per row | 19.1 | 10 min |
| 19.7 | Fix TransactionForm.tsx — label associations with htmlFor, aria-required on required fields, aria-invalid + aria-describedby on validation errors | 19.1 | 10 min |
| 19.8 | Fix GlobalSearch.tsx — role="combobox", aria-expanded, aria-activedescendant for keyboard-navigated results, aria-label | 19.1 | 15 min |
| 19.9 | Fix Sidebar navigation — aria-current="page" on active nav item, keyboard navigation with arrow keys between nav items | 19.1 | 10 min |
| 19.10 | Fix BudgetGrid.tsx — role="grid", aria-label, keyboard navigation between cells, editable cells have role="spinbutton" | 19.1 | 15 min |
| 19.11 | Fix TagPicker.tsx — role="listbox" with role="option" children, aria-selected, aria-multiselectable="true" | 19.1 | 10 min |
| 19.12 | Fix SplitEditor.tsx — aria-label per row, running total announced via aria-live region | 19.1 | 10 min |
| 19.13 | Add skip-to-content link — hidden link at top of page, visible on focus, jumps to main content area | 19.1 | 5 min |
| 19.14 | Add keyboard shortcut help dialog — Ctrl+? opens overlay showing all shortcuts, role="dialog", auto-focus close button | 19.1 | 10 min |
| 19.15 | Run automated accessibility audit — aXe or similar against all views, fix any remaining violations | 19.2 through 19.14 | 15 min |
| 19.16 | Manual screen reader test — VoiceOver on macOS: navigate all views, verify announcements make sense, forms are operable, modals trap focus correctly | 19.15 | 15 min |

*Concurrency: 19.1 first. Then 19.2 ‖ 19.3 ‖ 19.4 ‖ 19.5 ‖ 19.6 ‖ 19.7 ‖ 19.8 ‖ 19.9 ‖ 19.10 ‖ 19.11 ‖ 19.12 ‖ 19.13 ‖ 19.14 (all independent fixes). Then 19.15 → 19.16.*

*Commit point: After 19.16 verified — all views pass aXe + VoiceOver basic navigation.*

---

### Phase 20: Encryption at Rest + Auto-Update + Final Polish

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 20.1 | Update Cargo.toml — switch rusqlite from "bundled" to "bundled-sqlcipher" feature. Verify cargo check passes. | Phase 19 | 10 min |
| 20.2 | Write commands/encryption.rs — check_encryption_status: detect if current DB is encrypted or plaintext | 20.1 | 10 min |
| 20.3 | Write commands/encryption.rs — enable_encryption: given password, create new encrypted DB, copy all data via ATTACH + INSERT INTO ... SELECT, swap files, verify integrity | 20.1 | 20 min |
| 20.4 | Write commands/encryption.rs — unlock_database: given password, attempt to open encrypted DB, return success/failure | 20.1 | 10 min |
| 20.5 | Write commands/encryption.rs — change_password: PRAGMA rekey with new password | 20.4 | 10 min |
| 20.6 | Write commands/encryption.rs — disable_encryption: decrypt to plaintext (reverse of enable), requires current password | 20.3 | 10 min |
| 20.7 | Update db/mod.rs — on startup, check if DB is encrypted. If encrypted, emit event requesting password before proceeding. | 20.2 | 10 min |
| 20.8 | Register encryption commands in lib.rs | 20.3, 20.4, 20.5, 20.6 | 5 min |
| 20.9 | Run cargo check | 20.8 | 5 min |
| 20.10 | Write encryption tests — enable encryption, verify DB unreadable without password, unlock with correct password, change password, disable encryption | 20.3, 20.4, 20.5, 20.6 | 15 min |
| 20.11 | Write unlock screen frontend — password input, submit, error display for wrong password, "Forgot password" warning (data recovery impossible) | 20.7 | 15 min |
| 20.12 | Write encryption settings UI — enable/disable encryption toggle with password entry, change password form | 20.2 | 15 min |
| 20.13 | Add tauri-plugin-updater to Cargo.toml + tauri.conf.json — configure update endpoint (GitHub releases), check interval | Phase 19 | 10 min |
| 20.14 | Write auto-update check on launch — check for updates, show notification banner "Update available: v1.x.x" with "Install" button, download + install on click | 20.13 | 15 min |
| 20.15 | Performance audit — test with 10,000 transactions: verify transaction list pagination <200ms, dashboard aggregation <500ms, FTS5 search <100ms. Add indexes if needed. | Phase 19 | 15 min |
| 20.16 | Add app icon — design simple lock/shield icon matching privacy theme, set in tauri.conf.json for all platforms | Phase 19 | 10 min |
| 20.17 | Add about dialog — app name, version (from Cargo.toml), "Privacy-first. Offline-only. No subscriptions." tagline, links to repo | Phase 19 | 10 min |
| 20.18 | Final keyboard shortcut audit — ensure all views reachable, all modals escapeable, no shortcut conflicts | Phase 19 | 10 min |
| 20.19 | Run full cargo test suite — all phases | 20.10 | 5 min |
| 20.20 | Run npm run tauri build — verify binary produced for current platform | 20.19 | 10 min |
| 20.21 | Final smoke test: create account, add transactions, import CSV, create recurring, set budget, create rules, split transaction, add tags, run reports, reconcile, export data, backup + restore, encrypt + unlock, verify all persists after restart | 20.20 | 20 min |
| 20.22 | Cross-platform build verification — build for macOS (dmg), Windows (msi/nsis), Linux (deb/AppImage) if CI available | 20.20 | 15 min |
| 20.23 | Update README.md — feature list, screenshots placeholder, build instructions, privacy statement | 20.21 | 10 min |

*Concurrency: 20.1 ‖ 20.13 ‖ 20.15 ‖ 20.16 ‖ 20.17 ‖ 20.18 (independent). Encryption: 20.2 ‖ 20.3 ‖ 20.4 (after 20.1) → 20.5 ‖ 20.6 → 20.7 → 20.8. Frontend: 20.11 ‖ 20.12 (after 20.7). Update: 20.14 (after 20.13). Then 20.19 → 20.20 → 20.21 → 20.22 ‖ 20.23.*

*Commit point: After 20.21 passes — this is the v1.0 release candidate commit.*

---

## Session Safety Summary

| Phase | Feature | Tasks | ~Time | Commit After |
|-------|---------|-------|-------|--------------|
| 11 | Recurring Transactions | 22 | ~130 min | generation verified |
| 12 | Full-Text Search (FTS5) | 16 | ~105 min | search works E2E |
| 13 | Data Export + Backup/Restore | 17 | ~115 min | export + restore verified |
| 14 | Envelope Budgeting | 23 | ~150 min | budget tracking works |
| 15 | Auto-Categorization Rules | 22 | ~140 min | rules auto-apply |
| 16 | Split Transactions + Tags | 28 | ~160 min | splits + tags E2E |
| 17 | Comprehensive Reports | 22 | ~155 min | reports render |
| 18 | Reconciliation + Multi-Currency | 24 | ~155 min | reconciliation works |
| 19 | Accessibility (ARIA) | 16 | ~120 min | screen reader verified |
| 20 | Encryption + Auto-Update + Polish | 23 | ~170 min | final build + smoke |
| **Total** | | **213** | **~23-25 hrs** | **10 commit points** |

If a session cuts out: worst case is losing one 10-30 minute task. The last commit point is always a working state. Resume by checking git log for last commit, finding which phase/task was in progress.

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| SQLCipher build complexity (cross-platform OpenSSL) | Use `bundled-sqlcipher-vendored-openssl` if standard bundled fails. Test on CI early in Phase 20. |
| Budget period boundary edge cases (month lengths, leap years) | Extensive chrono tests in Phase 14. Use last-day-of-month logic, not fixed 28/30/31. |
| Regex ReDoS in rules engine | Compile regex with `regex` crate's default linear-time engine. Limit pattern length to 500 chars. Test with pathological patterns. |
| FTS5 sync drift (triggers fail silently) | Phase 12 includes rebuild-FTS command for manual re-sync. Integrity check on startup. |
| Performance at 10k+ transactions | Phase 20 includes explicit performance audit. Indexes added per phase. Dashboard uses read connection. Reports use date-bounded queries. |
| ARIA combobox complexity (GlobalSearch) | Follow WAI-ARIA 1.2 combobox pattern exactly. Test with VoiceOver. Fall back to simpler pattern if needed. |
| Multi-currency exchange rate staleness | Default to manual entry. API fetch is opt-in behind feature flag. Show "last updated" date on rates. |
| Backup file corruption | PRAGMA integrity_check before and after restore. Keep backup of current DB before restore overwrites it. |
| Split transaction sum validation bypass | Enforce sum check in Rust (not frontend). DB constraint would be ideal but SQLite lacks CHECK across tables — use application-level validation. |
| Reconciliation with filtered/hidden transactions | Show ALL unreconciled transactions during reconciliation regardless of current filters. Clear warning in UI. |

---

## Automated Tests (Rust)

All tests run against in-memory SQLite (:memory:) with the same migration applied.

### Phase 11 Tests
1. **Recurring CRUD** — Create, read, update, delete, toggle active
2. **Next date calculation** — All 6 frequencies, month boundary (31st → 28th Feb), leap year, quarterly boundary
3. **Generation** — Due items generate correct transaction count, next_date advances, end_date terminates generation, auto_post creates without prompt

### Phase 12 Tests
4. **FTS5 sync** — Insert transaction triggers FTS insert, update re-indexes, delete removes
5. **Search ranking** — Exact match > partial match, multi-word AND semantics, prefix queries

### Phase 13 Tests
6. **Export** — CSV format correct (header + data), JSON valid, filters applied (date range, account)
7. **Backup/Restore** — Backup creates valid copy, integrity check passes, restore replaces DB, corrupted file rejected

### Phase 14 Tests
8. **Budget CRUD** — Create budget auto-creates items for all categories, update allocation
9. **Budget calculation** — Available = budgeted - spent, "to be budgeted" = income - total budgeted
10. **Rollover** — Positive surplus carries forward, negative deficit reduces next month, no rollover when flag off
11. **Copy budget** — Previous month cloned, new categories get 0

### Phase 15 Tests
12. **Rules CRUD** — Create, read, update, delete, reorder priority
13. **Pattern matching** — Contains, starts_with, ends_with, exact, regex. Priority ordering. Regex timeout on catastrophic backtracking.
14. **Auto-apply** — Uncategorized transactions matched, already-categorized untouched

### Phase 16 Tests
15. **Splits** — Sum validation (reject mismatch), delete restores parent, dashboard uses split categories
16. **Tags** — CRUD, tag/untag, query by tag, cascade delete

### Phase 17 Tests
17. **Reports** — Income/expense totals match, category percentages sum to 100%, net worth matches cumulative calculation, cash flow running total correct

### Phase 18 Tests
18. **Reconciliation** — Diff calculation correct, complete only when balanced, cleared flags set, cancel leaves transactions unchanged
19. **Exchange rates** — CRUD, nearest-date lookup, conversion accuracy, cross-currency transfer stores both amounts

### Phase 20 Tests
20. **Encryption** — Enable makes DB unreadable without password, unlock succeeds with correct password, change password works, disable decrypts

---

## Explicitly Not in Post-MVP

- Cloud sync / account linking / Plaid integration
- Mobile app (Tauri mobile is experimental)
- Multi-user / shared access
- Investment tracking (stock prices, portfolio)
- Bill negotiation / subscription detection
- AI-powered insights or predictions
- Plugin / extension system
- Custom currencies (crypto)

---

## Verification Plan

### Phase 11: Recurring Transactions
1. Create monthly recurring for "Rent" $1500 expense
2. Create biweekly recurring for "Salary" $3000 income
3. Advance system date (or modify next_date in DB) to simulate due items
4. Verify RecurringDueModal appears on launch
5. Click "Generate All" — verify correct number of transactions created
6. Verify next_date advanced correctly for each frequency

### Phase 12: FTS5 Search
7. Add 50+ transactions with varied payees/descriptions
8. Search "grocery" — verify relevant results ranked by relevance
9. Search partial word (prefix) — verify autocomplete suggestions
10. Delete a transaction — verify it no longer appears in search

### Phase 13: Export + Backup
11. Export CSV — open in Numbers/Excel, verify columns and data correct
12. Export JSON — validate with JSON parser
13. Create backup — verify file created at chosen path
14. Modify data, restore backup — verify data reverted to backup state

### Phase 14: Budgeting
15. Navigate to Budgets page — verify all expense categories listed
16. Set $500 budget for Food, spend $300 — verify Available shows $200
17. Overspend Food to $600 — verify progress bar red, Available negative
18. Enable rollover on Food, advance to next month — verify surplus/deficit carried forward
19. Click "Copy from last month" — verify allocations match

### Phase 15: Rules
20. Create rule: payee contains "NETFLIX" → category "Subscriptions"
21. Import CSV with Netflix entries — verify auto-categorized
22. Test rule preview — verify shows correct transaction matches
23. Create regex rule — verify complex patterns work
24. Disable rule — verify it stops matching

### Phase 16: Splits + Tags
25. Split $100 restaurant transaction: $60 Food, $20 Drinks, $20 Tips
26. Verify budget page reflects split categories, not parent
27. Create "vacation" and "business" tags
28. Tag transactions, filter by tag — verify correct results

### Phase 17: Reports
29. View Income vs Expense — verify chart matches manual calculation
30. View Category Spending — verify percentages sum to 100%
31. View Cash Flow — verify running total correct
32. View Net Worth — verify matches account balance history
33. Change date range — verify reports update

### Phase 18: Reconciliation
34. Start reconciliation for checking account, enter statement balance
35. Check off transactions until diff = 0, complete — verify cleared flags
36. Enter exchange rate USD/EUR, create cross-currency transfer — verify conversion

### Phase 19: Accessibility
37. Navigate entire app with keyboard only — all controls reachable
38. Run aXe audit — zero violations
39. Test with VoiceOver — all content announced sensibly

### Phase 20: Final
40. Enable encryption — verify DB file unreadable without app
41. Close and reopen — verify password prompt, unlock works
42. Check for updates (mock endpoint) — verify notification shown
43. Full smoke test: all features from Phase 11-19 still work
44. Build binary — verify launches on clean machine

---

## Phase Summary Table

| Phase | Feature | Tasks | ~Time | Commit After |
|-------|---------|-------|-------|--------------|
| 11 | Recurring Transactions | 22 | 130 min | generation verified |
| 12 | FTS5 Search | 16 | 105 min | search E2E |
| 13 | Export + Backup | 17 | 115 min | export verified |
| 14 | Envelope Budgeting | 23 | 150 min | budgets work |
| 15 | Auto-Categorization | 22 | 140 min | rules auto-apply |
| 16 | Splits + Tags | 28 | 160 min | splits + tags E2E |
| 17 | Reports | 22 | 155 min | reports render |
| 18 | Reconciliation + Currency | 24 | 155 min | reconciliation works |
| 19 | Accessibility | 16 | 120 min | VoiceOver verified |
| 20 | Encryption + Polish | 23 | 170 min | final build + smoke |

**Total: 213 tasks, ~23-25 hours, 10 commit points**

**Combined with MVP: 349 tasks, ~38-41 hours, 21 commit points**
