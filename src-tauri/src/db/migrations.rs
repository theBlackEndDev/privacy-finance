use rusqlite::Connection;
use crate::error::AppResult;
use crate::helpers::timestamps::now_utc;

pub fn run_migrations(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(SCHEMA)?;
    seed_defaults(conn)?;
    fix_negative_expense_amounts(conn)?;
    populate_fts(conn)?;
    add_is_split_column(conn)?;
    Ok(())
}

/// Idempotent: add is_split column to transactions if it doesn't exist.
fn add_is_split_column(conn: &Connection) -> AppResult<()> {
    let has_column: bool = conn
        .prepare("PRAGMA table_info(transactions)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .any(|name| name.as_deref() == Ok("is_split"));
    if !has_column {
        conn.execute_batch("ALTER TABLE transactions ADD COLUMN is_split INTEGER NOT NULL DEFAULT 0")?;
    }
    Ok(())
}

/// Populate FTS5 index from existing transactions that aren't already indexed.
/// Idempotent: only inserts rows missing from FTS.
fn populate_fts(conn: &Connection) -> AppResult<()> {
    conn.execute(
        "INSERT INTO transactions_fts(transaction_id, payee, description)
         SELECT t.id, t.payee, t.description FROM transactions t
         LEFT JOIN transactions_fts fts ON fts.transaction_id = t.id
         WHERE fts.transaction_id IS NULL",
        [],
    )?;
    Ok(())
}

/// Rebuild FTS5 index from scratch — drop all FTS rows and repopulate.
pub fn rebuild_fts(conn: &Connection) -> AppResult<()> {
    conn.execute("DELETE FROM transactions_fts", [])?;
    conn.execute(
        "INSERT INTO transactions_fts(transaction_id, payee, description)
         SELECT id, payee, description FROM transactions",
        [],
    )?;
    Ok(())
}

/// One-time idempotent fix: older seed data stored expense amounts as negative.
/// Normalize all expense amounts to positive so the frontend's sign prefix works correctly.
fn fix_negative_expense_amounts(conn: &Connection) -> AppResult<()> {
    conn.execute(
        "UPDATE transactions SET amount = ABS(amount) WHERE kind = 'expense' AND amount < 0",
        [],
    )?;
    Ok(())
}

const SCHEMA: &str = "
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS accounts (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,
    type            TEXT NOT NULL CHECK(type IN ('checking', 'savings', 'credit_card', 'cash', 'investment', 'other')),
    currency_code   TEXT NOT NULL DEFAULT 'USD',
    opening_balance INTEGER NOT NULL DEFAULT 0,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    is_archived     INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS categories (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    color       TEXT NOT NULL DEFAULT '#6366f1',
    icon        TEXT NOT NULL DEFAULT 'folder',
    type        TEXT NOT NULL CHECK(type IN ('income', 'expense')) DEFAULT 'expense',
    sort_order  INTEGER NOT NULL DEFAULT 0,
    is_archived INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS import_batches (
    id          TEXT PRIMARY KEY,
    filename    TEXT NOT NULL,
    file_hash   TEXT NOT NULL,
    account_id  TEXT NOT NULL,
    profile_id  TEXT,
    row_count   INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);

CREATE TABLE IF NOT EXISTS import_profiles (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    mapping     TEXT NOT NULL,
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS transactions (
    id                TEXT PRIMARY KEY,
    account_id        TEXT NOT NULL,
    date              TEXT NOT NULL,
    amount            INTEGER NOT NULL,
    kind              TEXT NOT NULL CHECK(kind IN ('income', 'expense', 'transfer')) DEFAULT 'expense',
    category_id       TEXT,
    payee             TEXT NOT NULL DEFAULT '',
    description       TEXT NOT NULL DEFAULT '',
    is_cleared        INTEGER NOT NULL DEFAULT 0,
    transfer_group_id TEXT,
    import_batch_id   TEXT,
    import_row_hash   TEXT,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL,
    FOREIGN KEY (import_batch_id) REFERENCES import_batches(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS recurring_transactions (
    id                      TEXT PRIMARY KEY,
    account_id              TEXT NOT NULL,
    payee                   TEXT NOT NULL DEFAULT '',
    description             TEXT NOT NULL DEFAULT '',
    amount                  INTEGER NOT NULL,
    kind                    TEXT NOT NULL CHECK(kind IN ('income', 'expense', 'transfer')),
    category_id             TEXT,
    transfer_to_account_id  TEXT,
    frequency               TEXT NOT NULL CHECK(frequency IN ('daily', 'weekly', 'biweekly', 'monthly', 'quarterly', 'yearly')),
    start_date              TEXT NOT NULL,
    end_date                TEXT,
    next_date               TEXT NOT NULL,
    is_active               INTEGER NOT NULL DEFAULT 1,
    auto_post               INTEGER NOT NULL DEFAULT 0,
    created_at              TEXT NOT NULL,
    updated_at              TEXT NOT NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE RESTRICT,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL,
    FOREIGN KEY (transfer_to_account_id) REFERENCES accounts(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_transactions_account ON transactions(account_id);
CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date);
CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category_id);
CREATE INDEX IF NOT EXISTS idx_transactions_kind ON transactions(kind);
CREATE INDEX IF NOT EXISTS idx_transactions_account_date ON transactions(account_id, date);
CREATE INDEX IF NOT EXISTS idx_transactions_payee ON transactions(payee);
CREATE INDEX IF NOT EXISTS idx_transactions_import_batch ON transactions(import_batch_id);
CREATE INDEX IF NOT EXISTS idx_transactions_transfer_group ON transactions(transfer_group_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_transactions_dedupe ON transactions(account_id, import_row_hash);

CREATE INDEX IF NOT EXISTS idx_recurring_next_date ON recurring_transactions(next_date);
CREATE INDEX IF NOT EXISTS idx_recurring_account ON recurring_transactions(account_id);
CREATE INDEX IF NOT EXISTS idx_recurring_active ON recurring_transactions(is_active);

-- FTS5 full-text search for transactions
CREATE VIRTUAL TABLE IF NOT EXISTS transactions_fts USING fts5(
    transaction_id UNINDEXED,
    payee,
    description,
    tokenize='unicode61'
);

-- Sync triggers: keep FTS5 in sync with transactions table
CREATE TRIGGER IF NOT EXISTS trg_transactions_fts_insert
AFTER INSERT ON transactions
BEGIN
    INSERT INTO transactions_fts(transaction_id, payee, description)
    VALUES (NEW.id, NEW.payee, NEW.description);
END;

CREATE TRIGGER IF NOT EXISTS trg_transactions_fts_update
AFTER UPDATE OF payee, description ON transactions
BEGIN
    DELETE FROM transactions_fts WHERE transaction_id = OLD.id;
    INSERT INTO transactions_fts(transaction_id, payee, description)
    VALUES (NEW.id, NEW.payee, NEW.description);
END;

CREATE TRIGGER IF NOT EXISTS trg_transactions_fts_delete
AFTER DELETE ON transactions
BEGIN
    DELETE FROM transactions_fts WHERE transaction_id = OLD.id;
END;

-- Envelope Budgeting
CREATE TABLE IF NOT EXISTS budgets (
    id          TEXT PRIMARY KEY,
    year        INTEGER NOT NULL,
    month       INTEGER NOT NULL CHECK(month BETWEEN 1 AND 12),
    notes       TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    UNIQUE(year, month)
);

CREATE TABLE IF NOT EXISTS budget_items (
    id          TEXT PRIMARY KEY,
    budget_id   TEXT NOT NULL,
    category_id TEXT NOT NULL,
    budgeted    INTEGER NOT NULL DEFAULT 0,
    rollover    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    FOREIGN KEY (budget_id) REFERENCES budgets(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE,
    UNIQUE(budget_id, category_id)
);

CREATE INDEX IF NOT EXISTS idx_budget_items_budget ON budget_items(budget_id);
CREATE INDEX IF NOT EXISTS idx_budget_items_category ON budget_items(category_id);
CREATE INDEX IF NOT EXISTS idx_budgets_year_month ON budgets(year, month);

-- Auto-Categorization Rules
CREATE TABLE IF NOT EXISTS categorization_rules (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    match_field TEXT NOT NULL CHECK(match_field IN ('payee', 'description', 'amount')),
    match_type  TEXT NOT NULL CHECK(match_type IN ('contains', 'starts_with', 'ends_with', 'exact', 'regex')),
    pattern     TEXT NOT NULL,
    category_id TEXT NOT NULL,
    priority    INTEGER NOT NULL DEFAULT 0,
    is_active   INTEGER NOT NULL DEFAULT 1,
    apply_count INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_rules_active_priority ON categorization_rules(is_active, priority);

-- Phase 16: Split Transactions + Tags

-- Add is_split flag to transactions (SQLite ALTER TABLE ADD COLUMN)
-- Cannot use IF NOT EXISTS with ALTER TABLE, so we create a trigger-safe approach:
-- We'll handle this in Rust code instead.

CREATE TABLE IF NOT EXISTS transaction_splits (
    id              TEXT PRIMARY KEY,
    transaction_id  TEXT NOT NULL,
    category_id     TEXT NOT NULL,
    amount          INTEGER NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    created_at      TEXT NOT NULL,
    FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_splits_transaction_id ON transaction_splits(transaction_id);

CREATE TABLE IF NOT EXISTS tags (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    color       TEXT,
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS transaction_tags (
    transaction_id  TEXT NOT NULL,
    tag_id          TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    PRIMARY KEY (transaction_id, tag_id),
    FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_transaction_tags_tag_id ON transaction_tags(tag_id);

-- Account Reconciliation
CREATE TABLE IF NOT EXISTS reconciliations (
    id                TEXT PRIMARY KEY,
    account_id        TEXT NOT NULL,
    statement_date    TEXT NOT NULL,
    statement_balance INTEGER NOT NULL,
    status            TEXT NOT NULL CHECK(status IN ('pending', 'completed', 'cancelled')) DEFAULT 'pending',
    created_at        TEXT NOT NULL,
    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS reconciliation_items (
    reconciliation_id TEXT NOT NULL,
    transaction_id    TEXT NOT NULL,
    is_reconciled     INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (reconciliation_id, transaction_id),
    FOREIGN KEY (reconciliation_id) REFERENCES reconciliations(id) ON DELETE CASCADE,
    FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_reconciliation_items_recon_id ON reconciliation_items(reconciliation_id);

-- Exchange Rates
CREATE TABLE IF NOT EXISTS exchange_rates (
    id            TEXT PRIMARY KEY,
    from_currency TEXT NOT NULL,
    to_currency   TEXT NOT NULL,
    rate          REAL NOT NULL,
    date          TEXT NOT NULL,
    created_at    TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_exchange_rates_pair_date ON exchange_rates(from_currency, to_currency, date);
";

fn insert_category(conn: &Connection, name: &str, color: &str, kind: &str, sort: i64, now: &str) -> AppResult<String> {
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO categories (id, name, color, icon, type, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'folder', ?4, ?5, ?6, ?6)",
        rusqlite::params![id, name, color, kind, sort, now],
    )?;
    Ok(id)
}

fn insert_account(conn: &Connection, name: &str, kind: &str, opening_balance: i64, sort: i64, now: &str) -> AppResult<String> {
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO accounts (id, name, type, currency_code, opening_balance, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'USD', ?4, ?5, ?6, ?6)",
        rusqlite::params![id, name, kind, opening_balance, sort, now],
    )?;
    Ok(id)
}

fn insert_tx(
    conn: &Connection,
    account_id: &str,
    date: &str,
    amount: i64,
    kind: &str,
    category_id: Option<&str>,
    payee: &str,
    description: &str,
    cleared: bool,
    now: &str,
) -> AppResult<()> {
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO transactions
         (id, account_id, date, amount, kind, category_id, payee, description, is_cleared, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?10)",
        rusqlite::params![id, account_id, date, amount.abs(), kind, category_id, payee, description, cleared as i64, now],
    )?;
    Ok(())
}

fn insert_transfer(conn: &Connection, from_id: &str, to_id: &str, amount: i64, date: &str, desc: &str, now: &str) -> AppResult<()> {
    let group_id = uuid::Uuid::new_v4().to_string();
    let id1 = uuid::Uuid::new_v4().to_string();
    let id2 = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO transactions (id, account_id, date, amount, kind, payee, description, is_cleared, transfer_group_id, created_at, updated_at)
         VALUES (?1,?2,?3,?4,'transfer','Transfer',?5,1,?6,?7,?7)",
        rusqlite::params![id1, from_id, date, -amount, desc, group_id, now],
    )?;
    conn.execute(
        "INSERT INTO transactions (id, account_id, date, amount, kind, payee, description, is_cleared, transfer_group_id, created_at, updated_at)
         VALUES (?1,?2,?3,?4,'transfer','Transfer',?5,1,?6,?7,?7)",
        rusqlite::params![id2, to_id, date, amount, desc, group_id, now],
    )?;
    Ok(())
}

fn seed_defaults(conn: &Connection) -> AppResult<()> {
    // Only seed if categories table is empty
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;
    if count > 0 {
        return Ok(());
    }

    let now = now_utc();

    // ── Categories ───────────────────────────────────────────────────────────

    let cat_salary      = insert_category(conn, "Salary",        "#22c55e", "income",  0, &now)?;
    let cat_freelance   = insert_category(conn, "Freelance",     "#3b82f6", "income",  1, &now)?;
    let cat_investments = insert_category(conn, "Investments",   "#8b5cf6", "income",  2, &now)?;
    let _cat_oi         = insert_category(conn, "Other Income",  "#06b6d4", "income",  3, &now)?;

    let cat_housing     = insert_category(conn, "Housing",        "#ef4444", "expense",  0, &now)?;
    let cat_food        = insert_category(conn, "Food",           "#f97316", "expense",  1, &now)?;
    let cat_transport   = insert_category(conn, "Transportation", "#eab308", "expense",  2, &now)?;
    let cat_utilities   = insert_category(conn, "Utilities",      "#14b8a6", "expense",  3, &now)?;
    let cat_health      = insert_category(conn, "Healthcare",     "#ec4899", "expense",  4, &now)?;
    let cat_entertain   = insert_category(conn, "Entertainment",  "#8b5cf6", "expense",  5, &now)?;
    let cat_shopping    = insert_category(conn, "Shopping",       "#f43f5e", "expense",  6, &now)?;
    let _cat_education  = insert_category(conn, "Education",      "#6366f1", "expense",  7, &now)?;
    let cat_subs        = insert_category(conn, "Subscriptions",  "#a855f7", "expense",  8, &now)?;
    let cat_personal    = insert_category(conn, "Personal",       "#0ea5e9", "expense",  9, &now)?;
    let _cat_other      = insert_category(conn, "Other",          "#6b7280", "expense", 10, &now)?;

    // ── Accounts ─────────────────────────────────────────────────────────────

    let checking = insert_account(conn, "Primary Checking", "checking",    250_000, 0, &now)?; // $2,500 opening
    let savings  = insert_account(conn, "High-Yield Savings", "savings",  1_200_000, 1, &now)?; // $12,000
    let cc       = insert_account(conn, "Visa Rewards Card", "credit_card", 0,      2, &now)?;

    // ── Transactions — 3 months of realistic data ─────────────────────────────
    // Month -2 (two months ago)

    // Salaries
    insert_tx(conn, &checking, "2026-01-15", 425_000, "income", Some(&cat_salary), "Acme Corp", "Salary — Jan", true, &now)?;
    insert_tx(conn, &checking, "2026-01-31", 425_000, "income", Some(&cat_salary), "Acme Corp", "Salary — Jan", true, &now)?;
    insert_tx(conn, &checking, "2026-02-14", 425_000, "income", Some(&cat_salary), "Acme Corp", "Salary — Feb", true, &now)?;
    insert_tx(conn, &checking, "2026-02-28", 425_000, "income", Some(&cat_salary), "Acme Corp", "Salary — Feb", true, &now)?;
    insert_tx(conn, &checking, "2026-03-14", 425_000, "income", Some(&cat_salary), "Acme Corp", "Salary — Mar", true, &now)?;
    insert_tx(conn, &checking, "2026-03-31", 425_000, "income", Some(&cat_salary), "Acme Corp", "Salary — Mar", false, &now)?;

    // Freelance
    insert_tx(conn, &checking, "2026-01-20", 120_000, "income", Some(&cat_freelance), "Studio Nine", "Logo design project", true, &now)?;
    insert_tx(conn, &checking, "2026-03-05", 85_000, "income", Some(&cat_freelance), "Apex Media", "Website audit", true, &now)?;

    // Investment dividends
    insert_tx(conn, &savings, "2026-01-31", 18_400, "income", Some(&cat_investments), "Vanguard", "Dividend Q4", true, &now)?;
    insert_tx(conn, &savings, "2026-02-29", 21_200, "income", Some(&cat_investments), "Vanguard", "Dividend", true, &now)?;
    insert_tx(conn, &savings, "2026-03-31", 19_700, "income", Some(&cat_investments), "Vanguard", "Dividend", false, &now)?;

    // Housing — rent on checking
    insert_tx(conn, &checking, "2026-01-01", -185_000, "expense", Some(&cat_housing), "Sunset Apartments", "Rent — January", true, &now)?;
    insert_tx(conn, &checking, "2026-02-01", -185_000, "expense", Some(&cat_housing), "Sunset Apartments", "Rent — February", true, &now)?;
    insert_tx(conn, &checking, "2026-03-01", -185_000, "expense", Some(&cat_housing), "Sunset Apartments", "Rent — March", true, &now)?;

    // Utilities on CC
    insert_tx(conn, &cc, "2026-01-08", -9_400, "expense", Some(&cat_utilities), "City Power & Gas", "Electric + gas", true, &now)?;
    insert_tx(conn, &cc, "2026-01-10", -7_200, "expense", Some(&cat_utilities), "StreamNet ISP", "Internet", true, &now)?;
    insert_tx(conn, &cc, "2026-02-08", -9_800, "expense", Some(&cat_utilities), "City Power & Gas", "Electric + gas", true, &now)?;
    insert_tx(conn, &cc, "2026-02-10", -7_200, "expense", Some(&cat_utilities), "StreamNet ISP", "Internet", true, &now)?;
    insert_tx(conn, &cc, "2026-03-08", -8_900, "expense", Some(&cat_utilities), "City Power & Gas", "Electric + gas", false, &now)?;
    insert_tx(conn, &cc, "2026-03-10", -7_200, "expense", Some(&cat_utilities), "StreamNet ISP", "Internet", false, &now)?;

    // Food / Groceries on CC
    insert_tx(conn, &cc, "2026-01-03", -12_400, "expense", Some(&cat_food), "Whole Foods", "Weekly groceries", true, &now)?;
    insert_tx(conn, &cc, "2026-01-09", -4_200, "expense", Some(&cat_food), "Blue Bottle Coffee", "Coffee & pastries", true, &now)?;
    insert_tx(conn, &cc, "2026-01-14", -13_100, "expense", Some(&cat_food), "Trader Joe's", "Weekly groceries", true, &now)?;
    insert_tx(conn, &cc, "2026-01-18", -8_700, "expense", Some(&cat_food), "Chipotle", "Team lunch", true, &now)?;
    insert_tx(conn, &cc, "2026-01-21", -11_900, "expense", Some(&cat_food), "Whole Foods", "Weekly groceries", true, &now)?;
    insert_tx(conn, &cc, "2026-01-25", -3_500, "expense", Some(&cat_food), "Starbucks", "Coffee runs", true, &now)?;
    insert_tx(conn, &cc, "2026-01-28", -14_200, "expense", Some(&cat_food), "Trader Joe's", "Weekly groceries", true, &now)?;
    insert_tx(conn, &cc, "2026-02-04", -12_800, "expense", Some(&cat_food), "Whole Foods", "Weekly groceries", true, &now)?;
    insert_tx(conn, &cc, "2026-02-11", -11_600, "expense", Some(&cat_food), "Trader Joe's", "Weekly groceries", true, &now)?;
    insert_tx(conn, &cc, "2026-02-14", -9_800, "expense", Some(&cat_food), "Nobu", "Valentine's dinner", true, &now)?;
    insert_tx(conn, &cc, "2026-02-18", -13_400, "expense", Some(&cat_food), "Whole Foods", "Weekly groceries", true, &now)?;
    insert_tx(conn, &cc, "2026-02-25", -12_100, "expense", Some(&cat_food), "Trader Joe's", "Weekly groceries", true, &now)?;
    insert_tx(conn, &cc, "2026-03-04", -13_700, "expense", Some(&cat_food), "Whole Foods", "Weekly groceries", false, &now)?;
    insert_tx(conn, &cc, "2026-03-11", -11_200, "expense", Some(&cat_food), "Trader Joe's", "Weekly groceries", false, &now)?;
    insert_tx(conn, &cc, "2026-03-15", -6_400, "expense", Some(&cat_food), "The French Laundry", "Birthday dinner", false, &now)?;
    insert_tx(conn, &cc, "2026-03-18", -12_900, "expense", Some(&cat_food), "Whole Foods", "Weekly groceries", false, &now)?;

    // Transportation on CC
    insert_tx(conn, &cc, "2026-01-05", -6_200, "expense", Some(&cat_transport), "Uber", "Rides", true, &now)?;
    insert_tx(conn, &cc, "2026-01-15", -5_500, "expense", Some(&cat_transport), "Shell", "Gas", true, &now)?;
    insert_tx(conn, &cc, "2026-02-05", -7_100, "expense", Some(&cat_transport), "Uber", "Rides", true, &now)?;
    insert_tx(conn, &cc, "2026-02-20", -5_500, "expense", Some(&cat_transport), "Shell", "Gas", true, &now)?;
    insert_tx(conn, &cc, "2026-03-05", -6_800, "expense", Some(&cat_transport), "Uber", "Rides", false, &now)?;
    insert_tx(conn, &cc, "2026-03-20", -5_500, "expense", Some(&cat_transport), "Chevron", "Gas", false, &now)?;

    // Subscriptions on CC
    insert_tx(conn, &cc, "2026-01-02", -1_599, "expense", Some(&cat_subs), "Netflix", "Monthly", true, &now)?;
    insert_tx(conn, &cc, "2026-01-02", -1_099, "expense", Some(&cat_subs), "Spotify", "Monthly", true, &now)?;
    insert_tx(conn, &cc, "2026-01-02", -2_999, "expense", Some(&cat_subs), "iCloud+", "Monthly", true, &now)?;
    insert_tx(conn, &cc, "2026-01-05", -1_400, "expense", Some(&cat_subs), "New York Times", "Monthly", true, &now)?;
    insert_tx(conn, &cc, "2026-02-02", -1_599, "expense", Some(&cat_subs), "Netflix", "Monthly", true, &now)?;
    insert_tx(conn, &cc, "2026-02-02", -1_099, "expense", Some(&cat_subs), "Spotify", "Monthly", true, &now)?;
    insert_tx(conn, &cc, "2026-02-02", -2_999, "expense", Some(&cat_subs), "iCloud+", "Monthly", true, &now)?;
    insert_tx(conn, &cc, "2026-02-05", -1_400, "expense", Some(&cat_subs), "New York Times", "Monthly", true, &now)?;
    insert_tx(conn, &cc, "2026-03-02", -1_599, "expense", Some(&cat_subs), "Netflix", "Monthly", false, &now)?;
    insert_tx(conn, &cc, "2026-03-02", -1_099, "expense", Some(&cat_subs), "Spotify", "Monthly", false, &now)?;
    insert_tx(conn, &cc, "2026-03-02", -2_999, "expense", Some(&cat_subs), "iCloud+", "Monthly", false, &now)?;
    insert_tx(conn, &cc, "2026-03-05", -1_400, "expense", Some(&cat_subs), "New York Times", "Monthly", false, &now)?;

    // Entertainment on CC
    insert_tx(conn, &cc, "2026-01-12", -8_000, "expense", Some(&cat_entertain), "AMC Theatres", "Movie night", true, &now)?;
    insert_tx(conn, &cc, "2026-01-22", -15_000, "expense", Some(&cat_entertain), "Ticketmaster", "Jazz concert tickets", true, &now)?;
    insert_tx(conn, &cc, "2026-02-08", -7_500, "expense", Some(&cat_entertain), "AMC Theatres", "Movie night", true, &now)?;
    insert_tx(conn, &cc, "2026-02-22", -22_000, "expense", Some(&cat_entertain), "Ticketmaster", "NBA tickets", true, &now)?;
    insert_tx(conn, &cc, "2026-03-14", -9_000, "expense", Some(&cat_entertain), "AMC Theatres", "Movie night x2", false, &now)?;

    // Shopping on CC
    insert_tx(conn, &cc, "2026-01-17", -24_900, "expense", Some(&cat_shopping), "Amazon", "Home supplies", true, &now)?;
    insert_tx(conn, &cc, "2026-02-10", -18_500, "expense", Some(&cat_shopping), "Target", "Household items", true, &now)?;
    insert_tx(conn, &cc, "2026-02-16", -34_000, "expense", Some(&cat_shopping), "Apple Store", "AirPods Pro", true, &now)?;
    insert_tx(conn, &cc, "2026-03-07", -22_300, "expense", Some(&cat_shopping), "Amazon", "Misc household", false, &now)?;

    // Healthcare
    insert_tx(conn, &cc, "2026-01-24", -15_000, "expense", Some(&cat_health), "Dr. Martinez", "Dental cleaning", true, &now)?;
    insert_tx(conn, &cc, "2026-02-12", -4_200, "expense", Some(&cat_health), "CVS Pharmacy", "Prescriptions", true, &now)?;
    insert_tx(conn, &cc, "2026-03-22", -20_000, "expense", Some(&cat_health), "Dr. Chen", "Annual physical", false, &now)?;

    // Personal
    insert_tx(conn, &cc, "2026-01-19", -8_500, "expense", Some(&cat_personal), "Great Clips", "Haircut", true, &now)?;
    insert_tx(conn, &cc, "2026-02-19", -8_500, "expense", Some(&cat_personal), "Great Clips", "Haircut", true, &now)?;
    insert_tx(conn, &cc, "2026-03-19", -8_500, "expense", Some(&cat_personal), "Great Clips", "Haircut", false, &now)?;

    // CC payments (transfer checking → CC)
    insert_transfer(conn, &checking, &cc, 180_000, "2026-02-05", "CC payment — January", &now)?;
    insert_transfer(conn, &checking, &cc, 195_000, "2026-03-05", "CC payment — February", &now)?;

    // Savings transfers
    insert_transfer(conn, &checking, &savings, 50_000, "2026-01-31", "Monthly savings transfer", &now)?;
    insert_transfer(conn, &checking, &savings, 50_000, "2026-02-28", "Monthly savings transfer", &now)?;
    insert_transfer(conn, &checking, &savings, 50_000, "2026-03-31", "Monthly savings transfer", &now)?;

    Ok(())
}
