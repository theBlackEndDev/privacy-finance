use rusqlite::Connection;
use crate::error::AppResult;
use crate::helpers::timestamps::now_utc;

pub fn run_migrations(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(SCHEMA)?;
    seed_defaults(conn)?;
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

CREATE INDEX IF NOT EXISTS idx_transactions_account ON transactions(account_id);
CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date);
CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category_id);
CREATE INDEX IF NOT EXISTS idx_transactions_kind ON transactions(kind);
CREATE INDEX IF NOT EXISTS idx_transactions_account_date ON transactions(account_id, date);
CREATE INDEX IF NOT EXISTS idx_transactions_payee ON transactions(payee);
CREATE INDEX IF NOT EXISTS idx_transactions_import_batch ON transactions(import_batch_id);
CREATE INDEX IF NOT EXISTS idx_transactions_transfer_group ON transactions(transfer_group_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_transactions_dedupe ON transactions(account_id, import_row_hash);
";

fn seed_defaults(conn: &Connection) -> AppResult<()> {
    // Only seed if categories table is empty
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM categories",
        [],
        |row| row.get(0),
    )?;
    if count > 0 {
        return Ok(());
    }

    let now = now_utc();

    // Income categories
    let income_cats = [
        ("Salary",       "#22c55e"),
        ("Freelance",    "#3b82f6"),
        ("Investments",  "#8b5cf6"),
        ("Other Income", "#06b6d4"),
    ];
    for (i, (name, color)) in income_cats.iter().enumerate() {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO categories (id, name, color, icon, type, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'folder', 'income', ?4, ?5, ?6)",
            rusqlite::params![id, name, color, i as i64, now, now],
        )?;
    }

    // Expense categories
    let expense_cats = [
        ("Housing",        "#ef4444"),
        ("Food",           "#f97316"),
        ("Transportation", "#eab308"),
        ("Utilities",      "#14b8a6"),
        ("Healthcare",     "#ec4899"),
        ("Entertainment",  "#8b5cf6"),
        ("Shopping",       "#f43f5e"),
        ("Education",      "#6366f1"),
        ("Subscriptions",  "#a855f7"),
        ("Personal",       "#0ea5e9"),
        ("Other",          "#6b7280"),
    ];
    for (i, (name, color)) in expense_cats.iter().enumerate() {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO categories (id, name, color, icon, type, sort_order, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'folder', 'expense', ?4, ?5, ?6)",
            rusqlite::params![id, name, color, i as i64, now, now],
        )?;
    }

    // Default account
    let account_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM accounts",
        [],
        |row| row.get(0),
    )?;
    if account_count == 0 {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO accounts (id, name, type, currency_code, opening_balance, sort_order, created_at, updated_at)
             VALUES (?1, 'Primary Checking', 'checking', 'USD', 0, 0, ?2, ?3)",
            rusqlite::params![id, now, now],
        )?;
    }

    Ok(())
}
