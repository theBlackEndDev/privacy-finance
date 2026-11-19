use rusqlite::Connection;
use crate::error::AppResult;
use crate::helpers::timestamps::now_utc;

/// Delete all user data from every table and re-seed defaults.
/// Tables are deleted in FK-safe order (children first).
pub fn reset_all_data(conn: &Connection) -> AppResult<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute_batch("
        DELETE FROM reconciliation_items;
        DELETE FROM reconciliations;
        DELETE FROM transaction_tags;
        DELETE FROM tags;
        DELETE FROM transaction_splits;
        DELETE FROM budget_items;
        DELETE FROM budgets;
        DELETE FROM categorization_rules;
        DELETE FROM recurring_transactions;
        DELETE FROM transactions;
        DELETE FROM import_batches;
        DELETE FROM import_profiles;
        DELETE FROM categories;
        DELETE FROM category_groups;
        DELETE FROM accounts;
        DELETE FROM exchange_rates;
        DELETE FROM settings;
    ")?;
    seed_defaults(&tx)?;
    seed_category_groups(&tx)?;
    tx.commit()?;
    Ok(())
}

pub fn run_migrations(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(SCHEMA)?;
    // Agent gateway was never implemented; drop its tables (children first).
    conn.execute_batch("
        DROP TABLE IF EXISTS agent_audit_log;
        DROP TABLE IF EXISTS agent_scope_grants;
        DROP TABLE IF EXISTS agent_keys;
        DROP TABLE IF EXISTS agents;
    ")?;
    seed_defaults(conn)?;
    fix_negative_expense_amounts(conn)?;
    populate_fts(conn)?;
    add_is_split_column(conn)?;
    add_category_group_id_column(conn)?;
    seed_category_groups(conn)?;
    add_target_amount_column(conn)?;
    fix_transfer_signs(conn)?;
    add_rollover_amount_column(conn)?;
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

/// Idempotent: add group_id column to categories if it doesn't exist.
fn add_category_group_id_column(conn: &Connection) -> AppResult<()> {
    let has_column: bool = conn
        .prepare("PRAGMA table_info(categories)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .any(|name| name.as_deref() == Ok("group_id"));
    if !has_column {
        conn.execute_batch("ALTER TABLE categories ADD COLUMN group_id TEXT REFERENCES category_groups(id) ON DELETE SET NULL")?;
    }
    Ok(())
}

/// Idempotent: seed default category groups and assign categories to them.
fn seed_category_groups(conn: &Connection) -> AppResult<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM category_groups", [], |row| row.get(0))?;
    if count > 0 {
        return Ok(());
    }

    // Check if group_id column exists (it should after add_category_group_id_column)
    let has_column: bool = conn
        .prepare("PRAGMA table_info(categories)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .any(|name| name.as_deref() == Ok("group_id"));
    if !has_column {
        return Ok(());
    }

    let now = now_utc();

    let grp_essentials = uuid::Uuid::new_v4().to_string();
    let grp_lifestyle  = uuid::Uuid::new_v4().to_string();
    let grp_savings    = uuid::Uuid::new_v4().to_string();
    let grp_income     = uuid::Uuid::new_v4().to_string();

    for (id, name, sort) in [
        (&grp_essentials, "Essentials",      0),
        (&grp_lifestyle,  "Lifestyle",       1),
        (&grp_savings,    "Savings & Goals", 2),
        (&grp_income,     "Income",          3),
    ] {
        conn.execute(
            "INSERT INTO category_groups (id, name, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
            rusqlite::params![id, name, sort, now],
        )?;
    }

    // Assign default categories to groups
    // Essentials: Housing, Food, Transportation, Utilities, Healthcare
    for name in ["Housing", "Food", "Transportation", "Utilities", "Healthcare"] {
        conn.execute(
            "UPDATE categories SET group_id = ?1 WHERE name = ?2",
            rusqlite::params![grp_essentials, name],
        )?;
    }

    // Lifestyle: Entertainment, Shopping, Subscriptions, Personal, Education, Other
    for name in ["Entertainment", "Shopping", "Subscriptions", "Personal", "Education", "Other"] {
        conn.execute(
            "UPDATE categories SET group_id = ?1 WHERE name = ?2",
            rusqlite::params![grp_lifestyle, name],
        )?;
    }

    // Income: Salary, Freelance, Investments, Other Income
    for name in ["Salary", "Freelance", "Investments", "Other Income"] {
        conn.execute(
            "UPDATE categories SET group_id = ?1 WHERE name = ?2",
            rusqlite::params![grp_income, name],
        )?;
    }

    Ok(())
}

/// Idempotent: add target_amount column to budget_items if it doesn't exist.
fn add_target_amount_column(conn: &Connection) -> AppResult<()> {
    let has_column: bool = conn
        .prepare("PRAGMA table_info(budget_items)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .any(|name| name.as_deref() == Ok("target_amount"));
    if !has_column {
        conn.execute_batch("ALTER TABLE budget_items ADD COLUMN target_amount INTEGER")?;
    }
    Ok(())
}

/// Idempotent: add rollover_amount column to budget_items if it doesn't exist.
fn add_rollover_amount_column(conn: &Connection) -> AppResult<()> {
    let has_column: bool = conn
        .prepare("PRAGMA table_info(budget_items)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .any(|name| name.as_deref() == Ok("rollover_amount"));
    if !has_column {
        conn.execute_batch("ALTER TABLE budget_items ADD COLUMN rollover_amount INTEGER NOT NULL DEFAULT 0")?;
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
#[cfg(test)]
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

/// Idempotent: older code stored both legs of a transfer as positive amounts.
/// Only transfer groups where both legs are positive are touched; in those the
/// outgoing leg is negated. The outgoing leg is the one whose payee is
/// 'Transfer to account' (manual transfers), or, for recurring transfers, the
/// one on the recurring row's `account_id` whose peer is on its
/// `transfer_to_account_id`. A recurring pair that matches in both directions
/// is ambiguous and is left alone.
fn fix_transfer_signs(conn: &Connection) -> AppResult<()> {
    const BAD_GROUPS: &str = "SELECT transfer_group_id FROM transactions
         WHERE kind = 'transfer' AND transfer_group_id IS NOT NULL
         GROUP BY transfer_group_id
         HAVING COUNT(*) = 2 AND MIN(amount) > 0";

    let tx = conn.unchecked_transaction()?;
    tx.execute(
        &format!(
            "UPDATE transactions SET amount = -amount
             WHERE kind = 'transfer' AND amount > 0
               AND payee = 'Transfer to account'
               AND transfer_group_id IN ({BAD_GROUPS})"
        ),
        [],
    )?;

    // Runs after the manual fix, so groups repaired above are no longer "bad".
    tx.execute(
        &format!(
            "UPDATE transactions SET amount = -amount
             WHERE kind = 'transfer' AND amount > 0
               AND transfer_group_id IN ({BAD_GROUPS})
               AND EXISTS (
                   SELECT 1 FROM transactions o
                   JOIN recurring_transactions r
                     ON r.kind = 'transfer'
                    AND r.account_id = transactions.account_id
                    AND r.transfer_to_account_id = o.account_id
                   WHERE o.transfer_group_id = transactions.transfer_group_id
                     AND o.id != transactions.id)
               AND NOT EXISTS (
                   SELECT 1 FROM transactions o
                   JOIN recurring_transactions r
                     ON r.kind = 'transfer'
                    AND r.account_id = o.account_id
                    AND r.transfer_to_account_id = transactions.account_id
                   WHERE o.transfer_group_id = transactions.transfer_group_id
                     AND o.id != transactions.id)"
        ),
        [],
    )?;
    tx.commit()?;
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

-- Category Groups
CREATE TABLE IF NOT EXISTS category_groups (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL UNIQUE,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

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

#[cfg(test)]
fn insert_account(conn: &Connection, name: &str, kind: &str, opening_balance: i64, sort: i64, now: &str) -> AppResult<String> {
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO accounts (id, name, type, currency_code, opening_balance, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'USD', ?4, ?5, ?6, ?6)",
        rusqlite::params![id, name, kind, opening_balance, sort, now],
    )?;
    Ok(id)
}

#[cfg(test)]
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

#[cfg(test)]
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

    let _cat_salary     = insert_category(conn, "Salary",        "#22c55e", "income",  0, &now)?;
    let _cat_freelance  = insert_category(conn, "Freelance",     "#3b82f6", "income",  1, &now)?;
    let _cat_investments = insert_category(conn, "Investments",   "#8b5cf6", "income",  2, &now)?;
    let _cat_oi         = insert_category(conn, "Other Income",  "#06b6d4", "income",  3, &now)?;

    let _cat_housing    = insert_category(conn, "Housing",        "#ef4444", "expense",  0, &now)?;
    let _cat_food       = insert_category(conn, "Food",           "#f97316", "expense",  1, &now)?;
    let _cat_transport  = insert_category(conn, "Transportation", "#eab308", "expense",  2, &now)?;
    let _cat_utilities  = insert_category(conn, "Utilities",      "#14b8a6", "expense",  3, &now)?;
    let _cat_health     = insert_category(conn, "Healthcare",     "#ec4899", "expense",  4, &now)?;
    let _cat_entertain  = insert_category(conn, "Entertainment",  "#8b5cf6", "expense",  5, &now)?;
    let _cat_shopping   = insert_category(conn, "Shopping",       "#f43f5e", "expense",  6, &now)?;
    let _cat_education  = insert_category(conn, "Education",      "#6366f1", "expense",  7, &now)?;
    let _cat_subs       = insert_category(conn, "Subscriptions",  "#a855f7", "expense",  8, &now)?;
    let _cat_personal   = insert_category(conn, "Personal",       "#0ea5e9", "expense",  9, &now)?;
    let _cat_other      = insert_category(conn, "Other",          "#6b7280", "expense", 10, &now)?;

    // No seed accounts or transactions in production — the onboarding wizard
    // guides users through creating their first account on fresh install.

    Ok(())
}

/// Seed sample accounts + transactions for unit tests.
/// Not called in production — only from test setup functions.
#[cfg(test)]
pub fn seed_test_data(conn: &Connection) -> AppResult<()> {
    let now = now_utc();

    // Reuse seed categories (already created by run_migrations → seed_defaults)
    let cat_salary: String = conn.query_row(
        "SELECT id FROM categories WHERE name = 'Salary'", [], |r| r.get(0)
    )?;
    let cat_food: String = conn.query_row(
        "SELECT id FROM categories WHERE name = 'Food'", [], |r| r.get(0)
    )?;
    let cat_housing: String = conn.query_row(
        "SELECT id FROM categories WHERE name = 'Housing'", [], |r| r.get(0)
    )?;

    let checking = insert_account(conn, "Primary Checking", "checking", 250_000, 0, &now)?;
    insert_account(conn, "High-Yield Savings", "savings", 1_200_000, 1, &now)?;
    let cc       = insert_account(conn, "Visa Rewards Card", "credit_card", 0, 2, &now)?;

    // Minimal set of transactions for testing
    insert_tx(conn, &checking, "2026-01-15", 425_000, "income", Some(&cat_salary), "Acme Corp", "Salary — Jan", true, &now)?;
    insert_tx(conn, &checking, "2026-02-14", 425_000, "income", Some(&cat_salary), "Acme Corp", "Salary — Feb", true, &now)?;
    insert_tx(conn, &checking, "2026-03-14", 425_000, "income", Some(&cat_salary), "Acme Corp", "Salary — Mar", true, &now)?;
    insert_tx(conn, &cc, "2026-01-03", -12_400, "expense", Some(&cat_food), "Whole Foods", "Groceries", true, &now)?;
    insert_tx(conn, &cc, "2026-02-04", -12_800, "expense", Some(&cat_food), "Trader Joe's", "Groceries", true, &now)?;
    insert_tx(conn, &cc, "2026-03-04", -13_700, "expense", Some(&cat_food), "Whole Foods", "Groceries", false, &now)?;
    insert_tx(conn, &checking, "2026-01-01", -185_000, "expense", Some(&cat_housing), "Sunset Apartments", "Rent", true, &now)?;
    insert_tx(conn, &checking, "2026-02-01", -185_000, "expense", Some(&cat_housing), "Sunset Apartments", "Rent", true, &now)?;
    insert_transfer(conn, &checking, &cc, 180_000, "2026-02-05", "CC payment", &now)?;

    Ok(())
}
