# PrivacyFinance — Full Implementation Plan

*Comprehensive plan for building PrivacyFinance, a privacy-first personal finance desktop app.*

---

## Quick Reference

### Stack
- **Tauri 2.x** (Rust + React + TypeScript)
- **rusqlite** (SQLite in Rust, bundled)
- **Recharts** for charts
- **PapaParse** for CSV
- **encoding_rs** for CSV encoding detection
- **sha2** for file hashing

### Key Technical Decisions

1. **Database:** rusqlite in Rust (NOT better-sqlite3) — Tauri's webview is NOT Node.js, so native addons won't work. Use rusqlite with the bundled feature (statically compiles SQLite).

2. **DB Connection Strategy:** Separate read/write connections — A single Mutex<Connection> over-serializes everything. SQLite with WAL supports concurrent readers. Use one write connection (mutex-protected) and a separate read connection (or small pool) for queries. This prevents import or batch insert from blocking autocomplete, dashboard aggregation, or transaction list queries.

3. **State Management:** React Context + useReducer — Only 5 views with non-overlapping data. No deeply nested prop drilling. Zustand/Redux add dependencies for zero gain at this scale. Context provides centralized state with predictable updates.

4. **CSS:** CSS Modules + CSS Custom Properties — No Tailwind (build complexity, class soup). No styled-components (runtime overhead). CSS Modules give scoped classes with zero runtime cost. CSS variables on :root provide the theming system.

5. **Components:** Custom, but pragmatic about complexity — Custom components give full theme and keyboard control. However, for MVP: use native `<select>` elements first, simple autocomplete (no combobox ARIA), and minimal modal system. Avoid sinking time into custom dropdown accessibility when native elements work. Upgrade post-MVP.

6. **Routing:** State-based (no react-router) — 5 views (Dashboard, Accounts, Transactions, Categories, Import). No nested routes, no URL params, no deep linking. A currentView state variable + switch statement.

7. **CSV Import:** Hybrid approach with idempotency — Frontend: PapaParse for instant preview of first 10 rows during column mapping. Backend: Rust csv crate for full import with deduplication and batch tracking. Every import creates an import_batch and every row gets a hash for dedupe. Imports are reversible by batch.

8. **Currency:** Integer cents (no floating point) — $42.99 stored as 4299. Eliminates all precision issues. Frontend formats for display.

9. **Transfers:** First-class from day one — kind IN ('income', 'expense', 'transfer') with transfer_group_id linking paired transactions. This correctly models checking-to-savings moves, credit card payments, and cash withdrawals without polluting income/expense reporting.

10. **Timestamps:** Consistent format everywhere — All created_at/updated_at fields use UTC RFC3339 strings (e.g. 2026-03-13T15:30:00Z). All date fields use YYYY-MM-DD. Rust helper function enforces updated_at on every write — no reliance on triggers or developer memory.

11. **Undo/Redo:** Backend-scoped, narrow — No frontend action history for persisted state. Undo is implemented as backend operations:
    - Undo last create/edit/delete transaction (current session only)
    - Undo last import batch (delete all transactions in that batch)
    - Not MVP: undo category merge, undo across restarts

---

## Database Schema

```sql
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

-- ============================================================
-- ACCOUNTS
-- ============================================================
CREATE TABLE IF NOT EXISTS accounts (
  id TEXT PRIMARY KEY, -- UUID v4
  name TEXT NOT NULL UNIQUE,
  type TEXT NOT NULL CHECK(type IN ('checking', 'savings', 'credit_card', 'cash', 'investment', 'other')),
  currency_code TEXT NOT NULL DEFAULT 'USD',
  opening_balance INTEGER NOT NULL DEFAULT 0, -- cents
  sort_order INTEGER NOT NULL DEFAULT 0,
  is_archived INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL, -- RFC3339 UTC
  updated_at TEXT NOT NULL -- RFC3339 UTC
);

-- ============================================================
-- CATEGORIES
-- ============================================================
CREATE TABLE IF NOT EXISTS categories (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  color TEXT NOT NULL DEFAULT '#6366f1',
  icon TEXT NOT NULL DEFAULT 'folder',
  type TEXT NOT NULL CHECK(type IN ('income', 'expense')) DEFAULT 'expense',
  sort_order INTEGER NOT NULL DEFAULT 0,
  is_archived INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- ============================================================
-- TRANSACTIONS
-- ============================================================
CREATE TABLE IF NOT EXISTS transactions (
  id TEXT PRIMARY KEY,
  account_id TEXT NOT NULL,
  date TEXT NOT NULL, -- YYYY-MM-DD
  amount INTEGER NOT NULL, -- cents (always positive)
  kind TEXT NOT NULL CHECK(kind IN ('income', 'expense', 'transfer')) DEFAULT 'expense',
  category_id TEXT,
  payee TEXT NOT NULL DEFAULT '',
  description TEXT NOT NULL DEFAULT '',
  is_cleared INTEGER NOT NULL DEFAULT 0,
  transfer_group_id TEXT, -- links paired transfer transactions
  import_batch_id TEXT, -- which import created this
  import_row_hash TEXT, -- for deduplication
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
  FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL,
  FOREIGN KEY (import_batch_id) REFERENCES import_batches(id) ON DELETE SET NULL
);

-- ============================================================
-- IMPORT BATCHES
-- ============================================================
CREATE TABLE IF NOT EXISTS import_batches (
  id TEXT PRIMARY KEY,
  filename TEXT NOT NULL,
  file_hash TEXT NOT NULL, -- SHA-256 of file contents
  account_id TEXT NOT NULL, -- which account this import targets
  profile_id TEXT,
  row_count INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  FOREIGN KEY (account_id) REFERENCES accounts(id),
  FOREIGN KEY (profile_id) REFERENCES import_profiles(id)
);

-- ============================================================
-- IMPORT PROFILES (saved CSV column mappings)
-- ============================================================
CREATE TABLE IF NOT EXISTS import_profiles (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  mapping TEXT NOT NULL, -- JSON column mappings
  created_at TEXT NOT NULL
);

-- ============================================================
-- SETTINGS (key-value store)
-- ============================================================
CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

-- ============================================================
-- INDEXES
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_transactions_account ON transactions(account_id);
CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date);
CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category_id);
CREATE INDEX IF NOT EXISTS idx_transactions_kind ON transactions(kind);
CREATE INDEX IF NOT EXISTS idx_transactions_account_date ON transactions(account_id, date);
CREATE INDEX IF NOT EXISTS idx_transactions_payee ON transactions(payee);
CREATE INDEX IF NOT EXISTS idx_transactions_import_batch ON transactions(import_batch_id);
CREATE INDEX IF NOT EXISTS idx_transactions_transfer_group ON transactions(transfer_group_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_transactions_dedupe ON transactions(account_id, import_row_hash);
```

### Default Seed Data

**Categories (income):**
- Salary (#22c55e), Freelance (#3b82f6), Investments (#8b5cf6), Other Income (#06b6d4)

**Categories (expense):**
- Housing (#ef4444), Food (#f97316), Transportation (#eab308), Utilities (#14b8a6), Healthcare (#ec4899), Entertainment (#8b5cf6), Shopping (#f43f5e), Education (#6366f1), Subscriptions (#a855f7), Personal (#0ea5e9), Other (#6b7280)

*Note: "Savings" removed as an expense category — savings is a transfer to a savings account, not spending.*

**Default account created on first launch:**
- "Primary Checking" (type: checking, opening_balance: 0)

---

## Architecture

### Rust Backend Structure

```
src-tauri/src/
├── lib.rs              -- App setup, state management, command registration
├── main.rs             -- Entry point (existing)
├── error.rs           -- AppError enum with thiserror
├── db/
│   ├── mod.rs         -- DbState (write conn + read conn), init, WAL/FK setup
│   └── migrations.rs  -- Schema creation + default seeding
├── commands/
│   ├── mod.rs
│   ├── accounts.rs    -- Account CRUD
│   ├── transactions.rs -- CRUD + filtering + pagination + payee autocomplete
│   ├── categories.rs  -- Category CRUD
│   ├── dashboard.rs   -- Aggregation queries (uses read connection)
│   ├── import.rs      -- CSV parse + validate + dedupe + batch insert
│   └── settings.rs    -- Key-value get/set
├── models/
│   ├── mod.rs
│   ├── account.rs     -- Account struct
│   ├── transaction.rs -- Transaction struct + transfer helpers
│   ├── category.rs
│   ├── import.rs      -- ImportBatch, ImportProfile structs
│   └── dashboard.rs   -- DashboardData, CategorySummary structs
└── helpers/
    ├── mod.rs
    └── timestamps.rs  -- now_utc() -> RFC3339 string
```

**Cargo.toml additions:**
```toml
rusqlite = { version = "0.31", features = ["bundled"] }
csv = "1.3"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
thiserror = "1"
tauri-plugin-dialog = "2"
encoding_rs = "0.8"
sha2 = "0.10"
```

**Remove from package.json:** better-sqlite3, @types/better-sqlite3

### Frontend Structure

```
src/
├── main.tsx
├── App.tsx              -- Providers + view router
├── types/
│   └── index.ts          -- All interfaces (Account, Transaction, Category, etc.)
├── api/
│   ├── accounts.ts       -- invoke() wrappers
│   ├── transactions.ts
│   ├── categories.ts
│   ├── dashboard.ts
│   ├── import.ts
│   └── settings.ts
├── context/
│   ├── AppContext.tsx    -- Navigation, settings, active account
│   ├── TransactionContext.tsx -- Transaction list + filters
│   ├── CategoryContext.tsx
│   └── AccountContext.tsx
├── hooks/
│   ├── useKeyboardShortcuts.ts
│   ├── useTransactions.ts
│   ├── useCategories.ts
│   ├── useAccounts.ts
│   ├── useDashboard.ts
│   └── useDebounce.ts
├── components/
│   ├── ui/               -- Button, Input, Modal, Table, Badge, ColorPicker, SearchInput, Spinner, EmptyState, Toast
│   ├── layout/           -- Sidebar, MainLayout, Header
│   ├── accounts/        -- AccountList, AccountForm
│   ├── transactions/    -- TransactionList, TransactionRow, TransactionForm, TransferForm, TransactionEditModal
│   ├── categories/      -- CategoryList, CategoryCard, CategoryForm, CategoryEditModal
│   ├── dashboard/       -- DashboardView, SummaryCards, SpendingPieChart, MonthlyTrendChart, RecentTransactions, AccountBalances
│   └── import/          -- ImportWizard, FileSelectStep, PreviewStep, ColumnMappingStep, ValidationStep, ImportResultStep
├── views/
│   ├── DashboardPage.tsx
│   ├── AccountsPage.tsx
│   ├── TransactionsPage.tsx
│   ├── CategoriesPage.tsx
│   └── ImportPage.tsx
├── styles/
│   ├── variables.css    -- CSS custom properties (theme)
│   ├── reset.css
│   └── global.css
└── utils/
    ├── formatCurrency.ts -- cents -> "$42.99"
    ├── formatDate.ts
    ├── classNames.ts
    └── constants.ts
```

---

## UI/UX Design

### Layout

```
+------------------+-----------------------------------------------+
|                  | Header (page title / month selector / search) |
| Sidebar          +-----------------------------------------------+
| (220px)          |                                               |
|                  | Main Content Area                             |
| [Logo]           | (scrollable, padded 24px)                      |
|                  |                                               |
| Dashboard        | Account selector in sidebar or header          |
| Accounts         | when viewing transactions/import              |
| Transactions     |                                               |
| Categories       |                                               |
| Import           |                                               |
|                  |                                               |
| [Settings]       |                                               |
+------------------+-----------------------------------------------+
```

- **Window:** 1200x800 minimum

### Dark Theme Palette

| Element | Color |
|---------|-------|
| Background (primary) | #0f1117 |
| Background (sidebar/cards) | #1a1d27 |
| Background (inputs) | #242736 |
| Background (hover) | #2d3145 |
| Text (primary) | #e8eaed (15.8:1 contrast) |
| Text (secondary) | #9aa0b2 |
| Text (muted) | #636980 |
| Accent | #6366f1 (indigo) |
| Income | #22c55e (green) |
| Expense | #ef4444 (red) |
| Transfer | #3b82f6 (blue) |

- **Font:** Inter (sans), JetBrains Mono (numbers in tables)
- **Numbers:** font-variant-numeric: tabular-nums for column alignment

### Transaction Quick-Entry Bar

*(top of Transactions page, always visible)*

```
[Account selector] [Date] [Payee] [Category] [Amount] [Kind: +/-/transfer] [Add]
```

- Tab flows left to right, Enter submits from any field
- When Kind=transfer: Category dropdown hides, "To Account" dropdown appears
- After submit: form clears, focus returns to Date, new row highlights briefly
- Payee: simple autocomplete from history (basic filtered list, not full combobox)

### Dashboard Layout

```
[Month/Year Selector]

[Account Balances Row] -- balance per account

[Income Card] [Expense Card] [Net Card] -- summary row

[Spending Pie Chart] [Monthly Trend Line Chart] -- charts row

[Recent Transactions (last 10)] -- list
```

### CSV Import Wizard (5 steps)

1. **File Select** — Drag-drop or browse (Tauri dialog). Select target account.
2. **Preview** — PapaParse shows first 10 rows, header checkbox, delimiter selector. Shows file hash for reference.
3. **Column Mapping** — Map columns to fields, date format selector, amount format (signed, debit/credit split, parentheses negatives). Save as profile. Load existing profile.
4. **Validation** — Rust validates full file. Shows: N valid, N duplicates (skipped), N errors (with details). Handles: BOM, blank lines, footer rows, quoted commas, locale decimals.
5. **Result** — "Imported N transactions (M duplicates skipped)" with "View imported" and "Undo import" buttons.

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+1-5 | Navigate views |
| Ctrl+N | Focus quick-entry form |
| Ctrl+F | Focus search |
| Escape | Close modal / clear search |

---

## Build Order — Granular Task Breakdown

Each task is designed to be completable in 10-30 minutes and produce a working, saveable artifact. Commit after each phase completes. If a session limit hits mid-phase, at most one small task is lost.

**Dependency Legend:**
- → = must complete before
- ‖ = can run concurrently
- Tasks with no listed dependency depend only on their phase's prior tasks completing

---

### Phase 0: Project Configuration + Capabilities

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 0.1 | Update tauri.conf.json — productName: "PrivacyFinance", identifier: com.privacyfinance.app, window 1200x800, title | none | 5 min |
| 0.2 | Update Cargo.toml — add rusqlite (bundled), csv, chrono (serde), uuid (v4, serde), thiserror, tauri-plugin-dialog, encoding_rs, sha2 | none | 5 min |
| 0.3 | Update package.json — remove better-sqlite3 + @types/better-sqlite3, run npm install | none | 5 min |
| 0.4 | Update capabilities/default.json — add dialog:default permission | none | 5 min |
| 0.5 | Create src/styles/variables.css — full dark theme CSS custom properties | none | 10 min |
| 0.6 | Create src/styles/reset.css + src/styles/global.css — normalize + typography | 0.5 | 10 min |
| 0.7 | Create frontend directory structure — empty files for types/, api/, context/, hooks/, components/ui/, views/, utils/ | none | 5 min |
| 0.8 | Create Rust directory structure — empty mod.rs files for db/, commands/, models/, helpers/ | none | 5 min |
| 0.9 | Remove boilerplate — delete default App.css content, default App.tsx content, logo assets, greeting UI | 0.5, 0.6 | 5 min |
| 0.10 | Run cargo check to verify Rust deps resolve | 0.2, 0.8 | 5 min |

*Concurrency: 0.1 ‖ 0.2 ‖ 0.3 ‖ 0.4 ‖ 0.5 ‖ 0.7 ‖ 0.8 (all independent). Then 0.6 → 0.9. Then 0.10.*

*Commit point: After 0.10 passes.*

---

### Phase 1: Database Core + Migrations

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 1.1 | Write helpers/timestamps.rs — now_utc() returns RFC3339 UTC string. Wire helpers/mod.rs | Phase 0 | 10 min |
| 1.2 | Write error.rs — AppError enum (Db, Io, Csv, Validation, NotFound) with thiserror + Serialize impl | Phase 0 | 10 min |
| 1.3 | Write db/migrations.rs — all CREATE TABLE + CREATE INDEX statements from schema spec + seed data (default categories + "Primary Checking" account) | 1.1 | 20 min |
| 1.4 | Write db/mod.rs — DbState struct with write Mutex + read Connection, init_db() function that opens DB in app data dir, sets WAL + FK pragmas, runs migrations | 1.2, 1.3 | 20 min |
| 1.5 | Update lib.rs — register tauri-plugin-dialog, call init_db() in setup, manage DbState | 1.4 | 10 min |
| 1.6 | Write commands/mod.rs + models/mod.rs — empty re-export modules | Phase 0 | 5 min |
| 1.7 | Run cargo check — verify everything compiles | 1.5, 1.6 | 5 min |
| 1.8 | Write migration tests — in-memory DB, verify all tables exist, seed data present, idempotent re-run | 1.4 | 15 min |
| 1.9 | Run cargo test — verify tests pass | 1.8 | 5 min |

*Concurrency: 1.1 ‖ 1.2 ‖ 1.6 (independent). Then 1.3 → 1.4 → 1.5 → 1.7. Also 1.4 → 1.8 → 1.9.*

*Commit point: After 1.9 passes.*

---

### Phase 2: Accounts + Categories (Rust Backend)

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 2.1 | Write models/account.rs — Account struct, CreateAccountInput, UpdateAccountInput with serde derives | Phase 1 | 10 min |
| 2.2 | Write models/category.rs — Category struct, CreateCategoryInput, UpdateCategoryInput | Phase 1 | 10 min |
| 2.3 | Write commands/accounts.rs — get_accounts, get_account, create_account, update_account, delete_account (with RESTRICT check) | 2.1 | 25 min |
| 2.4 | Write commands/categories.rs — get_categories, create_category, update_category, delete_category | 2.2 | 25 min |
| 2.5 | Register account + category commands in lib.rs invoke_handler | 2.3, 2.4 | 5 min |
| 2.6 | Run cargo check | 2.5 | 5 min |
| 2.7 | Write account CRUD tests — create, read, update, delete, delete-with-transactions-blocked | 2.3 | 15 min |
| 2.8 | Write category CRUD tests — create, read, update, delete, transactions-become-uncategorized | 2.4 | 15 min |
| 2.9 | Run cargo test | 2.7, 2.8 | 5 min |

*Concurrency: 2.1 ‖ 2.2. Then 2.3 ‖ 2.4 (independent). Then 2.5. Also 2.3 → 2.7 ‖ 2.4 → 2.8.*

*Commit point: After 2.9 passes.*

---

### Phase 3: Accounts + Categories (Frontend)

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 3.1 | Write types/index.ts — Account, Category, CreateAccountInput, CreateCategoryInput, UpdateAccountInput, UpdateCategoryInput interfaces | Phase 2 | 15 min |
| 3.2 | Write api/accounts.ts — invoke() wrappers for all account commands | 3.1 | 10 min |
| 3.3 | Write api/categories.ts — invoke() wrappers for all category commands | 3.1 | 10 min |
| 3.4 | Write utils/formatCurrency.ts — cents to display string | none | 5 min |
| 3.5 | Write utils/formatDate.ts — ISO date to display format | none | 5 min |
| 3.6 | Write utils/classNames.ts — CSS module class merging helper | none | 5 min |
| 3.7 | Write components/ui/Button.tsx + Button.module.css | 0.5 | 10 min |
| 3.8 | Write components/ui/Input.tsx + Input.module.css | 0.5 | 10 min |
| 3.9 | Write components/ui/Modal.tsx + Modal.module.css — minimal: overlay, content box, close button, focus trap | 0.5, 3.7 | 15 min |
| 3.10 | Write components/ui/ColorPicker.tsx + ColorPicker.module.css — grid of preset colors | 0.5 | 10 min |
| 3.11 | Write components/ui/Spinner.tsx + EmptyState.tsx — loading + empty indicators | 0.5 | 10 min |
| 3.12 | Write components/accounts/AccountList.tsx + AccountList.module.css | 3.2, 3.7, 3.8 | 15 min |
| 3.13 | Write components/accounts/AccountForm.tsx + AccountForm.module.css — create/edit form with native select for type | 3.2, 3.7, 3.8, 3.9 | 15 min |
| 3.14 | Write components/categories/CategoryList.tsx + CategoryList.module.css | 3.3, 3.7 | 15 min |
| 3.15 | Write components/categories/CategoryCard.tsx + CategoryCard.module.css | 3.3, 3.7 | 10 min |
| 3.16 | Write components/categories/CategoryForm.tsx + CategoryForm.module.css — with ColorPicker | 3.3, 3.7, 3.8, 3.9, 3.10 | 15 min |
| 3.17 | Write views/AccountsPage.tsx — composes AccountList + AccountForm modal | 3.12, 3.13 | 10 min |
| 3.18 | Write views/CategoriesPage.tsx — composes CategoryList + CategoryForm modal | 3.14, 3.15, 3.16 | 10 min |

*Concurrency: 3.1 ‖ 3.4 ‖ 3.5 ‖ 3.6 ‖ 3.7 ‖ 3.8 ‖ 3.10 ‖ 3.11. Then 3.2 ‖ 3.3 (after 3.1). Then 3.9 (after 3.7). Then 3.12 ‖ 3.13 ‖ 3.14 ‖ 3.15 ‖ 3.16 (after their deps). Then 3.17 ‖ 3.18.*

*Commit point: After 3.18 compiles.*

---

### Phase 4: App Shell / Layout / Navigation

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 4.1 | Write context/AppContext.tsx — currentView state, navigate function, active account | Phase 3 | 15 min |
| 4.2 | Write context/AccountContext.tsx — accounts list, loading, fetch on mount | 3.2 | 15 min |
| 4.3 | Write context/CategoryContext.tsx — categories list, loading, fetch on mount | 3.3 | 15 min |
| 4.4 | Write components/layout/Sidebar.tsx + Sidebar.module.css — nav items, active state, account list | 4.1, 4.2 | 20 min |
| 4.5 | Write components/layout/Header.tsx + Header.module.css — page title, search placeholder | 4.1 | 10 min |
| 4.6 | Write components/layout/MainLayout.tsx + MainLayout.module.css — sidebar + header + content slot | 4.4, 4.5 | 10 min |
| 4.7 | Rewrite App.tsx — wrap in providers (App, Account, Category), render MainLayout with view switch | 4.1, 4.2, 4.3, 4.6, 3.17, 3.18 | 15 min |
| 4.8 | Update main.tsx — import global styles (variables, reset, global) | 0.9 | 5 min |
| 4.9 | Write hooks/useKeyboardShortcuts.ts — Ctrl+1-5 navigation, Ctrl+N, Ctrl+F, Escape | 4.1 | 15 min |
| 4.10 | Wire keyboard shortcuts into App.tsx | 4.7, 4.9 | 5 min |
| 4.11 | Verify: npm run dev + cargo tauri dev — app launches, sidebar shows, navigation works, dark theme applied | 4.10 | 10 min |

*Concurrency: 4.1 ‖ 4.2 ‖ 4.3. Then 4.4 ‖ 4.5 (after 4.1). Then 4.6 → 4.7. Also 4.9 ‖ 4.8. Then 4.10 → 4.11.*

*Commit point: After 4.11 verified.*

---

### Phase 5: Transactions (Rust Backend)

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 5.1 | Write models/transaction.rs — Transaction struct (with joined category_name/color), CreateTransactionInput, UpdateTransactionInput, TransactionFilters, TransactionPage | Phase 2 | 15 min |
| 5.2 | Write commands/transactions.rs — create_transaction, get_transaction | 5.1 | 15 min |
| 5.3 | Write commands/transactions.rs — update_transaction, delete_transaction | 5.1 | 15 min |
| 5.4 | Write commands/transactions.rs — get_transactions (with filters, pagination, JOIN category) | 5.1 | 20 min |
| 5.5 | Write commands/transactions.rs — get_payees (DISTINCT payee LIKE query, LIMIT 20) | 5.1 | 10 min |
| 5.6 | Write commands/transactions.rs — create_transfer (creates two paired transactions with shared transfer_group_id, wrapped in DB transaction) | 5.2 | 20 min |
| 5.7 | Register all transaction commands in lib.rs | 5.2, 5.3, 5.4, 5.5, 5.6 | 5 min |
| 5.8 | Run cargo check | 5.7 | 5 min |
| 5.9 | Write transaction CRUD tests | 5.2, 5.3 | 15 min |
| 5.10 | Write transaction filter + pagination tests | 5.4 | 15 min |
| 5.11 | Write transfer creation tests — paired transactions, amounts match, delete cascades | 5.6 | 15 min |
| 5.12 | Run cargo test | 5.9, 5.10, 5.11 | 5 min |

*Concurrency: 5.2 ‖ 5.3 ‖ 5.4 ‖ 5.5 (all depend on 5.1). Then 5.6 (after 5.2). Then 5.7. Tests: 5.9 ‖ 5.10 ‖ 5.11.*

*Commit point: After 5.12 passes.*

---

### Phase 6: Transactions (Frontend)

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 6.1 | Add Transaction types to types/index.ts — Transaction, CreateTransactionInput, TransactionFilters, TransactionPage | Phase 5 | 10 min |
| 6.2 | Write api/transactions.ts — invoke() wrappers for all transaction commands | 6.1 | 10 min |
| 6.3 | Write context/TransactionContext.tsx — transactions list, filters, pagination, loading state, refetch | 6.2 | 20 min |
| 6.4 | Write hooks/useDebounce.ts — generic debounce hook for search | none | 5 min |
| 6.5 | Write components/transactions/TransactionForm.tsx + .module.css — quick entry bar: account select, date, payee (with simple autocomplete), category select, amount, kind toggle, submit | 6.2, 6.3, 3.7, 3.8 | 25 min |
| 6.6 | Write components/transactions/TransferForm.tsx + .module.css — from-account, to-account, amount, date, submit | 6.2, 3.7, 3.8 | 15 min |
| 6.7 | Write components/transactions/TransactionRow.tsx + .module.css — single row: date, payee, category badge, description, amount (green/red/blue), running balance | 6.1, 3.4 | 15 min |
| 6.8 | Write components/transactions/TransactionList.tsx + .module.css — table wrapper, column headers with sort, pagination controls, renders TransactionRows | 6.7, 6.3 | 20 min |
| 6.9 | Write components/transactions/TransactionFilters.tsx + .module.css — date range inputs, category native select, kind select, search input with debounce | 6.3, 6.4, 3.8 | 15 min |
| 6.10 | Write components/transactions/TransactionEditModal.tsx + .module.css — edit form in Modal | 6.2, 3.9, 3.8 | 15 min |
| 6.11 | Write views/TransactionsPage.tsx — composes TransactionForm + TransferForm toggle + TransactionFilters + TransactionList, wrapped in TransactionContext | 6.5, 6.6, 6.8, 6.9, 6.10 | 15 min |
| 6.12 | Wire TransactionsPage into App.tsx view switch | 6.11, 4.7 | 5 min |
| 6.13 | Verify: add transaction, see it in list, edit, delete, filter, transfer between accounts | 6.12 | 10 min |

*Concurrency: 6.1 ‖ 6.4. Then 6.2 → 6.3. Then 6.5 ‖ 6.6 ‖ 6.7 ‖ 6.9 ‖ 6.10 (after their deps). Then 6.8 (after 6.7 + 6.3). Then 6.11 → 6.12 → 6.13.*

*Commit point: After 6.13 verified.*

---

### Phase 7: Dashboard

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 7.1 | Write models/dashboard.rs — DashboardData, CategorySummary, MonthSummary, AccountBalance structs (add to models/mod.rs) | Phase 5 | 10 min |
| 7.2 | Write commands/dashboard.rs — get_dashboard_data: income/expense/net aggregation by month, by_category breakdown (excludes transfers) | 7.1 | 20 min |
| 7.3 | Write commands/dashboard.rs — get_monthly_trend: last N months income/expense/net | 7.1 | 15 min |
| 7.4 | Write commands/dashboard.rs — get_account_balances: opening_balance + sum(income) - sum(expense) +/- transfers per account | 7.1 | 15 min |
| 7.5 | Register dashboard commands in lib.rs | 7.2, 7.3, 7.4 | 5 min |
| 7.6 | Run cargo check | 7.5 | 5 min |
| 7.7 | Write dashboard aggregation tests — totals match, transfers excluded, monthly grouping correct | 7.2, 7.3, 7.4 | 20 min |
| 7.8 | Add Dashboard types to types/index.ts | Phase 6 | 5 min |
| 7.9 | Write api/dashboard.ts — invoke() wrappers | 7.8 | 5 min |
| 7.10 | Write hooks/useDashboard.ts — fetch dashboard data on month change | 7.9 | 10 min |
| 7.11 | Write components/dashboard/AccountBalances.tsx + .module.css — card per account showing name, type, balance | 7.10, 3.4 | 15 min |
| 7.12 | Write components/dashboard/SummaryCards.tsx + .module.css — income (green), expense (red), net cards with large amounts | 7.10, 3.4 | 15 min |
| 7.13 | Write components/dashboard/SpendingPieChart.tsx + .module.css — Recharts PieChart with category colors, tooltips, legend | 7.10 | 20 min |
| 7.14 | Write components/dashboard/MonthlyTrendChart.tsx + .module.css — Recharts LineChart, dual income/expense lines, 6 months | 7.10 | 20 min |
| 7.15 | Write components/dashboard/RecentTransactions.tsx + .module.css — last 10 transactions mini-table | 7.10, 6.7 | 10 min |
| 7.16 | Write month/year selector component (simple prev/next + display) | 3.7 | 10 min |
| 7.17 | Write views/DashboardPage.tsx — composes all dashboard components with month selector | 7.11, 7.12, 7.13, 7.14, 7.15, 7.16 | 15 min |
| 7.18 | Wire DashboardPage into App.tsx + set as default view | 7.17, 4.7 | 5 min |
| 7.19 | Verify: dashboard shows correct totals, pie chart, trend chart, account balances | 7.18 | 10 min |

*Concurrency: Rust: 7.2 ‖ 7.3 ‖ 7.4 (after 7.1). Then 7.5. Also 7.7 (after 7.2-7.4). Frontend: 7.8 → 7.9 → 7.10. Then 7.11 ‖ 7.12 ‖ 7.13 ‖ 7.14 ‖ 7.15 ‖ 7.16 (all independent). Then 7.17 → 7.18 → 7.19.*

*Commit point: After 7.19 verified.*

---

### Phase 8: CSV Import (Rust Backend)

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 8.1 | Write models/import.rs — ImportBatch, ImportProfile, ColumnMapping, ImportResult, ImportError structs | Phase 5 | 10 min |
| 8.2 | Write commands/import.rs — preview_csv: read first 10 rows of file, auto-detect delimiter, return headers + preview rows | 8.1 | 20 min |
| 8.3 | Write commands/import.rs — import_csv: full parse with encoding detection (encoding_rs), column mapping, amount parsing (signed, debit/credit, parentheses), date parsing, row hashing, batch creation, deduped insert in DB transaction | 8.1 | 30 min |
| 8.4 | Write commands/import.rs — rollback_import: delete all transactions by import_batch_id, delete the batch | 8.1 | 10 min |
| 8.5 | Write commands/import.rs — get_import_profiles, save_import_profile, delete_import_profile | 8.1 | 15 min |
| 8.6 | CSV edge case handling in import_csv: BOM stripping, blank line skipping, footer row detection, quoted commas, locale decimal formats, duplicate headers | 8.3 | 20 min |
| 8.7 | Register import commands in lib.rs | 8.2, 8.3, 8.4, 8.5 | 5 min |
| 8.8 | Run cargo check | 8.7 | 5 min |
| 8.9 | Write import tests — import file, verify rows, reimport same file (0 new), rollback removes all | 8.3, 8.4 | 20 min |
| 8.10 | Write CSV parsing edge case tests — BOM, parentheses negatives, quoted commas, encoding | 8.6 | 15 min |
| 8.11 | Run cargo test | 8.9, 8.10 | 5 min |

*Concurrency: 8.2 ‖ 8.3 ‖ 8.4 ‖ 8.5 (after 8.1). Then 8.6 (after 8.3). Then 8.7. Tests: 8.9 ‖ 8.10.*

*Commit point: After 8.11 passes.*

---

### Phase 9: CSV Import (Frontend)

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 9.1 | Add Import types to types/index.ts — ImportBatch, ColumnMapping, ImportResult, ImportProfile | Phase 8 | 10 min |
| 9.2 | Write api/import.ts — invoke() wrappers for preview, import, rollback, profiles | 9.1 | 10 min |
| 9.3 | Write components/import/FileSelectStep.tsx + .module.css — Tauri dialog file picker, account selector, drag-drop zone | 9.2, 3.7 | 15 min |
| 9.4 | Write components/import/PreviewStep.tsx + .module.css — PapaParse first 10 rows table, header checkbox, delimiter selector | 9.2 | 15 min |
| 9.5 | Write components/import/ColumnMappingStep.tsx + .module.css — native selects mapping CSV columns to fields, date format dropdown, amount format selector, save/load profile | 9.2, 3.7, 3.8 | 20 min |
| 9.6 | Write components/import/ValidationStep.tsx + .module.css — display valid/duplicate/error counts, error detail table | 9.2 | 15 min |
| 9.7 | Write components/import/ImportResultStep.tsx + .module.css — success summary, "View imported" button, "Undo import" button | 9.2, 3.7 | 10 min |
| 9.8 | Write components/import/ImportWizard.tsx + .module.css — step state machine, back/next/cancel, wires all 5 steps | 9.3, 9.4, 9.5, 9.6, 9.7 | 20 min |
| 9.9 | Write views/ImportPage.tsx — renders ImportWizard | 9.8 | 5 min |
| 9.10 | Wire ImportPage into App.tsx view switch | 9.9, 4.7 | 5 min |
| 9.11 | Verify: select CSV, preview rows, map columns, import, see results, undo import | 9.10 | 10 min |

*Concurrency: 9.1 → 9.2. Then 9.3 ‖ 9.4 ‖ 9.5 ‖ 9.6 ‖ 9.7 (all independent). Then 9.8 → 9.9 → 9.10 → 9.11.*

*Commit point: After 9.11 verified.*

---

### Phase 10: Settings + Polish

| Task | Description | Depends On | ~Time |
|------|-------------|-----------|-------|
| 10.1 | Write api/settings.ts — invoke() wrappers for get/set | Phase 4 | 5 min |
| 10.2 | Write commands/settings.rs — get_setting, set_setting | Phase 1 | 10 min |
| 10.3 | Register settings commands in lib.rs | 10.2 | 5 min |
| 10.4 | Write settings UI in Sidebar or dedicated Settings section — currency symbol, date format dropdowns | 10.1, 10.3 | 15 min |
| 10.5 | Write components/ui/Toast.tsx + Toast.module.css — success/error notification that auto-dismisses | 3.7 | 15 min |
| 10.6 | Add Toast provider to App.tsx, wire toast calls into all create/update/delete operations | 10.5, 4.7 | 15 min |
| 10.7 | Add confirm dialogs before destructive actions — delete account, delete category, delete transaction, undo import | 3.9 | 15 min |
| 10.8 | Add EmptyState components to all list views — transactions, categories, accounts, dashboard | 3.11 | 10 min |
| 10.9 | Add Spinner loading states to all data-fetching views | 3.11 | 10 min |
| 10.10 | Final keyboard shortcut audit + add Ctrl+? help overlay or info in settings | 4.9 | 10 min |
| 10.11 | Run full cargo test suite | all | 5 min |
| 10.12 | Run npm run tauri build — verify binary produced | all | 10 min |
| 10.13 | Manual smoke test: full user flow — create account, add categories, add transactions, transfer, view dashboard, import CSV, undo import, verify persistence after restart | 10.12 | 15 min |

*Concurrency: 10.1 ‖ 10.2 ‖ 10.5 ‖ 10.7 ‖ 10.8 ‖ 10.9 (all independent). Then 10.3 → 10.4. Then 10.6 (after 10.5). Then 10.10 → 10.11 → 10.12 → 10.13.*

*Commit point: After 10.13 passes — this is the final MVP commit.*

---

## Session Safety Summary

| Phase | Tasks | Estimated Time | Commit After |
|-------|-------|----------------|--------------|
| 0 — Config | 10 tasks | ~30 min | cargo check passes |
| 1 — DB Core | 9 tasks | ~60 min | cargo test passes |
| 2 — Acct+Cat Rust | 9 tasks | ~75 min | cargo test passes |
| 3 — Acct+Cat Frontend | 18 tasks | ~120 min | npm run dev compiles |
| 4 — App Shell | 11 tasks | ~80 min | app launches + navigates |
| 5 — Transactions Rust | 12 tasks | ~100 min | cargo test passes |
| 6 — Transactions Frontend | 13 tasks | ~100 min | transactions work E2E |
| 7 — Dashboard | 19 tasks | ~120 min | dashboard renders correctly |
| 8 — Import Rust | 11 tasks | ~90 min | cargo test passes |
| 9 — Import Frontend | 11 tasks | ~80 min | import wizard works E2E |
| 10 — Polish | 13 tasks | ~90 min | final build + smoke test |
| **Total** | **136 tasks** | **~15-16 hrs** | **11 commit points** |

If a session cuts out: worst case is losing one 10-30 minute task. The last commit point is always a working state. Resume by checking git log for last commit, finding which phase/task was in progress.

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| DB connection contention during imports | Separate read/write connections. Write conn mutex-protected. Read conn for queries. WAL mode enables concurrent reads. |
| Large transaction lists | Server-side pagination (50/page). Never load all rows. |
| CSV encoding issues (non-UTF8) | encoding_rs crate detects and converts. Added to Cargo.toml. |
| CSV edge cases (parentheses negatives, split debit/credit, BOM, locale decimals) | Explicit handling in Rust parser for each case. Amount format selector in mapping step. |
| Date parsing from diverse CSV formats | Explicit format selector in import wizard. chrono::NaiveDate::parse_from_str with user-selected format. |
| Running balance misleading with filters | Only show when viewing single account, unfiltered, date-sorted. Show warning otherwise. |
| Duplicate imports | Unique index on (account_id, import_row_hash). File hash warns if same file re-imported. |
| Custom dropdown/combobox accessibility cost | Use native `<select>` for MVP. Simple filtered list for autocomplete. Upgrade to ARIA combobox post-MVP. |
| Search performance at scale | Accept LIKE '%term%' for MVP. Plan for FTS5 virtual table post-MVP when needed. |

---

## Automated Tests (Rust)

All tests run against in-memory SQLite (:memory:) with the same migration applied.

1. **Migration tests** — Schema creates cleanly, seed data present, idempotent re-run
2. **Account CRUD** — Create, read, update, delete. Verify delete blocked when transactions exist (ON DELETE RESTRICT)
3. **Category CRUD** — Create, read, update, delete. Verify transactions become uncategorized on category delete
4. **Transaction CRUD** — Create, read, update, delete. Verify amount stored as integer cents. Verify foreign keys enforced.
5. **Transfer creation** — Creates two linked transactions with shared transfer_group_id. Verify amounts match. Verify deleting one deletes both (or flags orphan).
6. **Dashboard aggregation** — Verify totals match manual sum. Verify transfers excluded from income/expense. Verify monthly grouping.
7. **CSV import** — Import file, verify row count. Reimport same file, verify 0 new rows (dedupe). Import rollback deletes all batch rows.
8. **CSV parsing edge cases** — BOM handling, blank lines, parentheses negatives, quoted commas, encoding detection
9. **Timestamp consistency** — All created_at/updated_at are valid RFC3339 UTC strings

---

## Explicitly Not MVP

- Full undo/redo (only session-scoped last-action undo + import batch rollback)
- Multi-currency
- Recurring transactions
- Budgets / envelope system
- Complex ARIA combobox dropdowns
- FTS5 search
- Data export (CSV/JSON/PDF)
- Backup/restore UI
- Light theme toggle
- Delta comparisons on dashboard

---

## Verification Plan

1. **Database:** Launch app, verify privacyfinance.db created in app data dir with all 6 tables
2. **Accounts:** Create checking + savings accounts. Verify they appear in sidebar. Edit name. Archive one.
3. **Categories:** Verify 15 default categories seeded. Create custom. Edit color. Delete — verify transactions become uncategorized.
4. **Transactions:** Add income + expense to checking account. Verify running balance = opening_balance + income - expenses. Edit, delete.
5. **Transfers:** Transfer $100 from checking to savings. Verify checking balance decreases, savings increases. Verify transfer excluded from income/expense totals on dashboard.
6. **Filters:** Filter by date range, category, kind, search text. Verify running balance hidden when filters active.
7. **Dashboard:** Verify totals match sum of transactions for selected month. Verify pie chart. Verify trend chart. Verify account balances.
8. **CSV Import:** Import a bank CSV. Verify column mapping. Verify transactions appear in correct account. Re-import same file — verify 0 duplicates added. Click "Undo import" — verify all batch transactions removed.
9. **Keyboard shortcuts:** Verify Ctrl+1-5 navigation, Ctrl+N focus, Escape close.
10. **Build:** Run npm run tauri build. Verify binary produced and launches correctly.
11. **Data persistence:** Add data, close app, reopen — verify everything persists.
12. **Rust tests:** Run cargo test — all pass.

---

## Phase Summary Table

| Phase | Tasks | ~Time | Commit After |
|-------|-------|-------|--------------|
| 0 — Config | 10 | 30 min | cargo check |
| 1 — DB Core | 9 | 60 min | cargo test |
| 2 — Acct+Cat Rust | 9 | 75 min | cargo test |
| 3 — Acct+Cat Frontend | 18 | 120 min | compiles |
| 4 — App Shell | 11 | 80 min | app launches |
| 5 — Transactions Rust | 12 | 100 min | cargo test |
| 6 — Transactions Frontend | 13 | 100 min | E2E works |
| 7 — Dashboard | 19 | 120 min | renders |
| 8 — Import Rust | 11 | 90 min | cargo test |
| 9 — Import Frontend | 11 | 80 min | E2E works |
| 10 — Polish | 13 | 90 min | final build |

**Total: ~136 tasks, ~15-16 hours, 11 commit points**
