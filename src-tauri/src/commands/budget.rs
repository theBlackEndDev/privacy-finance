use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::timestamps::now_utc;
use crate::models::budget::{
    Budget, BudgetCategoryRow, BudgetSummary, BudgetWithItems, UpdateBudgetItemInput,
};
use crate::models::transaction::Transaction;
use tauri::State;

// ── Helpers ──────────────────────────────────────────────────────────

fn prev_year_month(year: i32, month: i32) -> (i32, i32) {
    if month == 1 { (year - 1, 12) } else { (year, month - 1) }
}

fn month_like_pattern(year: i32, month: i32) -> String {
    format!("{:04}-{:02}%", year, month)
}

/// Split-aware spent per category for one month (`?1` = date LIKE pattern):
/// non-split expenses by parent category UNION ALL split rows by split category.
const SPENT_BY_CATEGORY_SQL: &str =
    "SELECT category_id, SUM(amount) AS spent
     FROM (
         SELECT t.category_id, t.amount
         FROM transactions t
         WHERE t.kind = 'expense' AND t.date LIKE ?1 AND t.is_split = 0
         UNION ALL
         SELECT ts.category_id, ts.amount
         FROM transactions t
         JOIN transaction_splits ts ON ts.transaction_id = t.id
         WHERE t.kind = 'expense' AND t.date LIKE ?1 AND t.is_split = 1
     )
     GROUP BY category_id";

fn row_to_budget(row: &rusqlite::Row) -> rusqlite::Result<Budget> {
    Ok(Budget {
        id: row.get(0)?,
        year: row.get(1)?,
        month: row.get(2)?,
        notes: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

/// Create zero-budgeted items for all active expense categories in a budget.
fn seed_budget_items(conn: &rusqlite::Connection, budget_id: &str, now: &str) -> AppResult<()> {
    let mut cat_stmt = conn.prepare(
        "SELECT id FROM categories WHERE type = 'expense' AND is_archived = 0 ORDER BY sort_order",
    )?;
    let cat_ids: Vec<String> = cat_stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?;

    for cat_id in &cat_ids {
        let item_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO budget_items (id, budget_id, category_id, budgeted, rollover, created_at, updated_at)
             VALUES (?1, ?2, ?3, 0, 0, ?4, ?4)",
            rusqlite::params![item_id, budget_id, cat_id, now],
        )?;
    }
    Ok(())
}

// ── Commands ─────────────────────────────────────────────────────────

/// Create a budget for the given year/month.
/// Auto-creates budget_items for all active expense categories with budgeted = 0.
/// Returns the created budget.
#[tauri::command]
pub fn create_budget(year: i32, month: i32, state: State<'_, DbState>) -> AppResult<Budget> {
    if !(1..=12).contains(&month) {
        return Err(AppError::Validation("Month must be between 1 and 12".into()));
    }

    let conn = state.write_conn.lock().unwrap();
    let tx = conn.unchecked_transaction()?;
    let now = now_utc();
    let budget_id = uuid::Uuid::new_v4().to_string();

    tx.execute(
        "INSERT INTO budgets (id, year, month, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, '', ?4, ?4)",
        rusqlite::params![budget_id, year, month, now],
    )?;

    seed_budget_items(&tx, &budget_id, &now)?;

    tx.commit()?;

    Ok(Budget {
        id: budget_id,
        year,
        month,
        notes: String::new(),
        created_at: now.clone(),
        updated_at: now,
    })
}

/// Get a budget with all items for a given year/month.
/// Each item includes category info, spent (from transactions), and available (budgeted - spent).
#[tauri::command]
pub fn get_budget(year: i32, month: i32, state: State<'_, DbState>) -> AppResult<Option<BudgetWithItems>> {
    let conn = state.read_conn.lock().unwrap();
    get_budget_inner(&conn, year, month)
}

fn get_budget_inner(conn: &rusqlite::Connection, year: i32, month: i32) -> AppResult<Option<BudgetWithItems>> {
    let budget: Option<Budget> = conn
        .query_row(
            "SELECT id, year, month, notes, created_at, updated_at
             FROM budgets WHERE year = ?1 AND month = ?2",
            rusqlite::params![year, month],
            row_to_budget,
        )
        .ok();

    let budget = match budget {
        Some(b) => b,
        None => return Ok(None),
    };

    let mut stmt = conn.prepare(&format!(
        "SELECT
            bi.id,
            bi.category_id,
            c.name,
            c.color,
            bi.budgeted,
            bi.rollover,
            COALESCE(t_sum.spent, 0) AS spent,
            c.group_id,
            cg.name,
            bi.target_amount,
            bi.rollover_amount
         FROM budget_items bi
         JOIN categories c ON c.id = bi.category_id
         LEFT JOIN category_groups cg ON c.group_id = cg.id
         LEFT JOIN ({SPENT_BY_CATEGORY_SQL}) t_sum ON t_sum.category_id = bi.category_id
         WHERE bi.budget_id = ?2
         ORDER BY COALESCE(cg.sort_order, 999), c.sort_order"
    ))?;

    let items: Vec<BudgetCategoryRow> = stmt
        .query_map(rusqlite::params![month_like_pattern(year, month), budget.id], |row| {
            let budgeted: i64 = row.get(4)?;
            let spent: i64 = row.get(6)?;
            let rollover_amount: i64 = row.get(10)?;
            Ok(BudgetCategoryRow {
                item_id: row.get(0)?,
                category_id: row.get(1)?,
                category_name: row.get(2)?,
                category_color: row.get(3)?,
                budgeted,
                spent,
                available: budgeted + rollover_amount - spent,
                rollover: row.get::<_, i64>(5)? != 0,
                rollover_amount,
                group_id: row.get(7)?,
                group_name: row.get(8)?,
                target_amount: row.get(9)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(Some(BudgetWithItems { budget, items }))
}

/// Update a single budget item's budgeted amount, rollover flag, and/or target.
#[tauri::command]
pub fn update_budget_item(
    item_id: String,
    input: UpdateBudgetItemInput,
    state: State<'_, DbState>,
) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    let now = now_utc();

    let mut sets: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    let mut idx = 1;

    if let Some(b) = input.budgeted {
        sets.push(format!("budgeted = ?{idx}"));
        params.push(Box::new(b));
        idx += 1;
    }
    if let Some(r) = input.rollover {
        sets.push(format!("rollover = ?{idx}"));
        params.push(Box::new(r as i64));
        idx += 1;
    }
    if let Some(ta) = input.target_amount {
        sets.push(format!("target_amount = ?{idx}"));
        params.push(Box::new(ta));
        idx += 1;
    }

    if sets.is_empty() {
        return Ok(());
    }

    sets.push(format!("updated_at = ?{idx}"));
    params.push(Box::new(now));
    idx += 1;

    let sql = format!(
        "UPDATE budget_items SET {} WHERE id = ?{idx}",
        sets.join(", ")
    );
    params.push(Box::new(item_id));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    conn.execute(&sql, param_refs.as_slice())?;

    Ok(())
}

/// Get transactions for a specific category in a given budget month.
#[tauri::command]
pub fn get_category_activity(
    category_id: String,
    year: i32,
    month: i32,
    state: State<'_, DbState>,
) -> AppResult<Vec<Transaction>> {
    use crate::commands::transactions::{row_to_transaction, SELECT};

    let conn = state.read_conn.lock().unwrap();
    let pattern = month_like_pattern(year, month);

    let sql = format!(
        "{SELECT} WHERE t.date LIKE ?2
           AND ((t.is_split = 0 AND t.category_id = ?1)
                OR (t.is_split = 1 AND EXISTS (
                    SELECT 1 FROM transaction_splits ts
                    WHERE ts.transaction_id = t.id AND ts.category_id = ?1)))
         ORDER BY t.date DESC, t.created_at DESC"
    );
    let mut stmt = conn.prepare(&sql)?;

    let txns = stmt
        .query_map(rusqlite::params![category_id, pattern], row_to_transaction)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(txns)
}

/// Copy the previous month's budget allocations to the specified month.
/// Creates the budget if it doesn't exist, then copies budgeted amounts.
#[tauri::command]
pub fn copy_budget(year: i32, month: i32, state: State<'_, DbState>) -> AppResult<Budget> {
    let (prev_year, prev_month) = prev_year_month(year, month);

    let conn = state.write_conn.lock().unwrap();
    let tx = conn.unchecked_transaction()?;

    // Get previous month's budget
    let prev_budget_id: Option<String> = tx
        .query_row(
            "SELECT id FROM budgets WHERE year = ?1 AND month = ?2",
            rusqlite::params![prev_year, prev_month],
            |row| row.get(0),
        )
        .ok();

    let prev_budget_id = match prev_budget_id {
        Some(id) => id,
        None => return Err(AppError::Validation("No budget found for previous month".into())),
    };

    let now = now_utc();

    // Check if current month budget exists, create if not
    let existing_id: Option<String> = tx
        .query_row(
            "SELECT id FROM budgets WHERE year = ?1 AND month = ?2",
            rusqlite::params![year, month],
            |row| row.get(0),
        )
        .ok();

    let budget_id = match existing_id {
        Some(id) => id,
        None => {
            let id = uuid::Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO budgets (id, year, month, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, '', ?4, ?4)",
                rusqlite::params![id, year, month, now],
            )?;

            seed_budget_items(&tx, &id, &now)?;

            id
        }
    };

    // Copy allocations from previous month in a single UPDATE...FROM
    tx.execute(
        "UPDATE budget_items SET
            budgeted = src.budgeted,
            rollover = src.rollover,
            updated_at = ?1
         FROM (
             SELECT category_id, budgeted, rollover
             FROM budget_items
             WHERE budget_id = ?2
         ) AS src
         WHERE budget_items.budget_id = ?3
           AND budget_items.category_id = src.category_id",
        rusqlite::params![now, prev_budget_id, budget_id],
    )?;

    let budget = tx.query_row(
        "SELECT id, year, month, notes, created_at, updated_at FROM budgets WHERE id = ?1",
        [&budget_id],
        row_to_budget,
    )?;

    tx.commit()?;

    Ok(budget)
}

/// Get budget summary for a given month: totals and "to be budgeted".
#[tauri::command]
pub fn get_budget_summary(year: i32, month: i32, state: State<'_, DbState>) -> AppResult<BudgetSummary> {
    let conn = state.read_conn.lock().unwrap();
    get_budget_summary_inner(&conn, year, month)
}

fn get_budget_summary_inner(conn: &rusqlite::Connection, year: i32, month: i32) -> AppResult<BudgetSummary> {
    let like_pattern = month_like_pattern(year, month);

    // Total budgeted
    let total_budgeted: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(bi.budgeted), 0)
             FROM budget_items bi
             JOIN budgets b ON b.id = bi.budget_id
             WHERE b.year = ?1 AND b.month = ?2",
            rusqlite::params![year, month],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Total rollover carried into this month (matches get_budget's per-row available)
    let total_rollover: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(bi.rollover_amount), 0)
             FROM budget_items bi
             JOIN budgets b ON b.id = bi.budget_id
             WHERE b.year = ?1 AND b.month = ?2",
            rusqlite::params![year, month],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Total spent (expenses in this month, excluding transfers)
    let total_spent: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions
             WHERE kind = 'expense' AND date LIKE ?1",
            rusqlite::params![&like_pattern],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Total income for this month (excluding transfers)
    let total_income: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions
             WHERE kind = 'income' AND date LIKE ?1",
            rusqlite::params![&like_pattern],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(BudgetSummary {
        total_budgeted,
        total_spent,
        total_available: total_budgeted + total_rollover - total_spent,
        total_income,
        to_be_budgeted: total_income - total_budgeted,
    })
}

/// Calculate rollover from previous month.
/// For each category with rollover=1 in the previous month, carry forward the previous
/// month's available (budgeted + rollover_amount - spent).
/// Sets (never adds to) the current month's `rollover_amount`, so calling it twice is safe.
#[tauri::command]
pub fn calculate_rollover(year: i32, month: i32, state: State<'_, DbState>) -> AppResult<i32> {
    let conn = state.write_conn.lock().unwrap();
    let tx = conn.unchecked_transaction()?;
    let count = apply_rollover(&tx, year, month)?;
    tx.commit()?;
    Ok(count)
}

fn apply_rollover(conn: &rusqlite::Connection, year: i32, month: i32) -> AppResult<i32> {
    let (prev_year, prev_month) = prev_year_month(year, month);
    let now = now_utc();

    // Get previous month's budget items with rollover enabled
    let prev_budget_id: Option<String> = conn
        .query_row(
            "SELECT id FROM budgets WHERE year = ?1 AND month = ?2",
            rusqlite::params![prev_year, prev_month],
            |row| row.get(0),
        )
        .ok();

    let prev_budget_id = match prev_budget_id {
        Some(id) => id,
        None => return Ok(0), // No previous budget, nothing to roll over
    };

    // Current month budget must exist
    let cur_budget_id: String = conn
        .query_row(
            "SELECT id FROM budgets WHERE year = ?1 AND month = ?2",
            rusqlite::params![year, month],
            |row| row.get(0),
        )
        .map_err(|_| AppError::Validation("Current month budget must exist before calculating rollover".into()))?;

    let mut stmt = conn.prepare(&format!(
        "SELECT bi.category_id, bi.budgeted + bi.rollover_amount,
                COALESCE(t_sum.spent, 0) AS spent
         FROM budget_items bi
         LEFT JOIN ({SPENT_BY_CATEGORY_SQL}) t_sum ON t_sum.category_id = bi.category_id
         WHERE bi.budget_id = ?2 AND bi.rollover = 1"
    ))?;

    let rollovers: Vec<(String, i64)> = stmt
        .query_map(rusqlite::params![month_like_pattern(prev_year, prev_month), prev_budget_id], |row| {
            let available_before: i64 = row.get(1)?;
            let spent: i64 = row.get(2)?;
            let rollover_amount = available_before - spent; // positive = surplus, negative = overspent
            Ok((row.get::<_, String>(0)?, rollover_amount))
        })?
        .collect::<Result<_, _>>()?;

    // Clear stale values first so categories no longer rolling over drop back to 0.
    conn.execute(
        "UPDATE budget_items SET rollover_amount = 0, updated_at = ?1
         WHERE budget_id = ?2 AND rollover_amount != 0",
        rusqlite::params![now, cur_budget_id],
    )?;

    let mut count = 0;
    for (cat_id, rollover_amount) in &rollovers {
        // Set (not add) the current month's rollover amount
        let updated = conn.execute(
            "UPDATE budget_items SET rollover_amount = ?1, updated_at = ?2
             WHERE budget_id = ?3 AND category_id = ?4",
            rusqlite::params![rollover_amount, now, cur_budget_id, cat_id],
        )?;
        if updated > 0 {
            count += 1;
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use rusqlite::{Connection, params};
    use crate::db::migrations::{run_migrations, seed_test_data};

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();
        seed_test_data(&conn).unwrap();
        conn
    }

    /// Returns (food_cat_id, transport_cat_id, salary_cat_id, account_id) from seeded data.
    fn get_seeded_ids(conn: &Connection) -> (String, String, String, String) {
        let food_id: String = conn.query_row(
            "SELECT id FROM categories WHERE name = 'Food'", [], |row| row.get(0),
        ).unwrap();
        let transport_id: String = conn.query_row(
            "SELECT id FROM categories WHERE name = 'Transportation'", [], |row| row.get(0),
        ).unwrap();
        let salary_id: String = conn.query_row(
            "SELECT id FROM categories WHERE name = 'Salary'", [], |row| row.get(0),
        ).unwrap();
        let account_id: String = conn.query_row(
            "SELECT id FROM accounts LIMIT 1", [], |row| row.get(0),
        ).unwrap();
        (food_id, transport_id, salary_id, account_id)
    }

    #[test]
    fn budget_summary_includes_rollover() {
        let conn = setup_db();
        let (food_id, transport_id, _, account_id) = get_seeded_ids(&conn);
        let now = "2026-06-01T00:00:00Z";

        let budget_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO budgets (id, year, month, notes, created_at, updated_at)
             VALUES (?1, 2026, 6, '', ?2, ?2)",
            params![budget_id, now],
        ).unwrap();
        // food: 50000 budgeted + 20000 rollover; transport: 30000 budgeted - 5000 rollover
        for (cat, budgeted, rollover) in [(&food_id, 50000, 20000), (&transport_id, 30000, -5000)] {
            conn.execute(
                "INSERT INTO budget_items (id, budget_id, category_id, budgeted, rollover, rollover_amount, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 1, ?5, ?6, ?6)",
                params![uuid::Uuid::new_v4().to_string(), budget_id, cat, budgeted, rollover, now],
            ).unwrap();
        }
        for (cat, amount) in [(&food_id, 15000), (&transport_id, 4000)] {
            conn.execute(
                "INSERT INTO transactions (id, account_id, date, amount, kind, category_id, payee, description, is_cleared, created_at, updated_at)
                 VALUES (?1, ?2, '2026-06-05', ?3, 'expense', ?4, 'Shop', '', 1, ?5, ?5)",
                params![uuid::Uuid::new_v4().to_string(), account_id, amount, cat, now],
            ).unwrap();
        }

        let items = super::get_budget_inner(&conn, 2026, 6).unwrap().unwrap().items;
        let summary = super::get_budget_summary_inner(&conn, 2026, 6).unwrap();

        let row_available: i64 = items.iter().map(|i| i.available).sum();
        assert_eq!(row_available, 50000 + 20000 - 15000 + 30000 - 5000 - 4000);
        assert_eq!(summary.total_available, row_available);
        assert_eq!(summary.total_budgeted, 80000);
        assert_eq!(summary.total_spent, 19000);
    }

    #[test]
    fn test_create_budget_auto_creates_items() {
        let conn = setup_db();

        // Count active expense categories from seed data
        let expense_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM categories WHERE type = 'expense' AND is_archived = 0",
            [], |row| row.get(0),
        ).unwrap();
        assert!(expense_count > 0);

        // Create budget for a month with no seed transactions (June 2026)
        let budget_id = uuid::Uuid::new_v4().to_string();
        let now = "2026-06-01T00:00:00Z";
        conn.execute(
            "INSERT INTO budgets (id, year, month, notes, created_at, updated_at)
             VALUES (?1, 2026, 6, '', ?2, ?2)",
            params![budget_id, now],
        ).unwrap();

        // Auto-create items for expense categories
        let mut cat_stmt = conn.prepare(
            "SELECT id FROM categories WHERE type = 'expense' AND is_archived = 0 ORDER BY sort_order",
        ).unwrap();
        let cat_ids: Vec<String> = cat_stmt
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        for cat_id in &cat_ids {
            let item_id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO budget_items (id, budget_id, category_id, budgeted, rollover, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 0, 0, ?4, ?4)",
                params![item_id, budget_id, cat_id, now],
            ).unwrap();
        }

        // Verify items created matches expense category count
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM budget_items WHERE budget_id = ?1",
            [&budget_id], |row| row.get(0),
        ).unwrap();
        assert_eq!(count, expense_count);

        // Verify no income category items
        let income_items: i64 = conn.query_row(
            "SELECT COUNT(*) FROM budget_items bi
             JOIN categories c ON c.id = bi.category_id
             WHERE bi.budget_id = ?1 AND c.type = 'income'",
            [&budget_id], |row| row.get(0),
        ).unwrap();
        assert_eq!(income_items, 0);
    }

    #[test]
    fn test_budget_spent_calculation() {
        let conn = setup_db();
        let (food_id, _, _, account_id) = get_seeded_ids(&conn);
        let now = "2026-06-01T00:00:00Z";

        // Create budget with food item budgeted at $500
        let budget_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO budgets (id, year, month, notes, created_at, updated_at)
             VALUES (?1, 2026, 6, '', ?2, ?2)",
            params![budget_id, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO budget_items (id, budget_id, category_id, budgeted, rollover, created_at, updated_at)
             VALUES (?1, ?2, ?3, 50000, 0, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), budget_id, food_id, now],
        ).unwrap();

        // Add expense transactions for June 2026
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, category_id, payee, description, is_cleared, created_at, updated_at)
             VALUES (?1, ?2, '2026-06-05', 15000, 'expense', ?3, 'Grocery', '', 1, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), account_id, food_id, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, category_id, payee, description, is_cleared, created_at, updated_at)
             VALUES (?1, ?2, '2026-06-12', 10000, 'expense', ?3, 'Restaurant', '', 1, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), account_id, food_id, now],
        ).unwrap();

        // Query spent for food in June
        let spent: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions
             WHERE category_id = ?1 AND kind = 'expense' AND date LIKE '2026-06%'",
            [&food_id], |row| row.get(0),
        ).unwrap();

        assert_eq!(spent, 25000); // $150 + $100 = $250
        let available = 50000 - spent;
        assert_eq!(available, 25000); // $500 - $250 = $250
    }

    #[test]
    fn test_rollover_positive() {
        let conn = setup_db();
        let (food_id, _, _, account_id) = get_seeded_ids(&conn);
        let now = "2026-06-01T00:00:00Z";

        // Create June budget with rollover enabled, $500 budgeted for food
        let jun_budget_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO budgets (id, year, month, notes, created_at, updated_at)
             VALUES (?1, 2026, 6, '', ?2, ?2)",
            params![jun_budget_id, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO budget_items (id, budget_id, category_id, budgeted, rollover, created_at, updated_at)
             VALUES (?1, ?2, ?3, 50000, 1, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), jun_budget_id, food_id, now],
        ).unwrap();

        // Spend only $300 in June (surplus of $200)
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, category_id, payee, description, is_cleared, created_at, updated_at)
             VALUES (?1, ?2, '2026-06-15', 30000, 'expense', ?3, 'Grocery', '', 1, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), account_id, food_id, now],
        ).unwrap();

        // Create July budget
        let jul_budget_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO budgets (id, year, month, notes, created_at, updated_at)
             VALUES (?1, 2026, 7, '', ?2, ?2)",
            params![jul_budget_id, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO budget_items (id, budget_id, category_id, budgeted, rollover, created_at, updated_at)
             VALUES (?1, ?2, ?3, 50000, 0, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), jul_budget_id, food_id, now],
        ).unwrap();

        // Calculate rollover: Jun had $500 budgeted, $300 spent → $200 surplus
        let rollover_amount: i64 = conn.query_row(
            "SELECT bi.budgeted - COALESCE((
                SELECT SUM(t.amount) FROM transactions t
                WHERE t.category_id = bi.category_id AND t.kind = 'expense' AND t.date LIKE '2026-06%'
            ), 0)
             FROM budget_items bi
             WHERE bi.budget_id = ?1 AND bi.category_id = ?2 AND bi.rollover = 1",
            params![jun_budget_id, food_id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(rollover_amount, 20000); // $200 surplus

        // Apply rollover to July
        conn.execute(
            "UPDATE budget_items SET budgeted = budgeted + ?1 WHERE budget_id = ?2 AND category_id = ?3",
            params![rollover_amount, jul_budget_id, food_id],
        ).unwrap();

        let july_budgeted: i64 = conn.query_row(
            "SELECT budgeted FROM budget_items WHERE budget_id = ?1 AND category_id = ?2",
            params![jul_budget_id, food_id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(july_budgeted, 70000); // $500 + $200 = $700
    }

    #[test]
    fn test_rollover_negative() {
        let conn = setup_db();
        let (food_id, _, _, account_id) = get_seeded_ids(&conn);
        let now = "2026-06-01T00:00:00Z";

        // Create June budget with rollover enabled, budget $500
        let jun_budget_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO budgets (id, year, month, notes, created_at, updated_at)
             VALUES (?1, 2026, 6, '', ?2, ?2)",
            params![jun_budget_id, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO budget_items (id, budget_id, category_id, budgeted, rollover, created_at, updated_at)
             VALUES (?1, ?2, ?3, 50000, 1, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), jun_budget_id, food_id, now],
        ).unwrap();

        // Overspend: $700 in June
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, category_id, payee, description, is_cleared, created_at, updated_at)
             VALUES (?1, ?2, '2026-06-10', 70000, 'expense', ?3, 'Big grocery', '', 1, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), account_id, food_id, now],
        ).unwrap();

        // Calculate rollover: $500 - $700 = -$200
        let rollover_amount: i64 = conn.query_row(
            "SELECT bi.budgeted - COALESCE((
                SELECT SUM(t.amount) FROM transactions t
                WHERE t.category_id = bi.category_id AND t.kind = 'expense' AND t.date LIKE '2026-06%'
            ), 0)
             FROM budget_items bi
             WHERE bi.budget_id = ?1 AND bi.category_id = ?2 AND bi.rollover = 1",
            params![jun_budget_id, food_id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(rollover_amount, -20000); // -$200 deficit
    }

    #[test]
    fn test_rollover_disabled() {
        let conn = setup_db();
        let (food_id, _, _, _) = get_seeded_ids(&conn);
        let now = "2026-06-01T00:00:00Z";

        // Create budget with rollover DISABLED
        let budget_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO budgets (id, year, month, notes, created_at, updated_at)
             VALUES (?1, 2026, 6, '', ?2, ?2)",
            params![budget_id, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO budget_items (id, budget_id, category_id, budgeted, rollover, created_at, updated_at)
             VALUES (?1, ?2, ?3, 50000, 0, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), budget_id, food_id, now],
        ).unwrap();

        // Query should return 0 rows with rollover = 1
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM budget_items WHERE budget_id = ?1 AND rollover = 1",
            [&budget_id], |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_copy_budget() {
        let conn = setup_db();
        let (food_id, transport_id, _, _) = get_seeded_ids(&conn);
        let now = "2026-06-01T00:00:00Z";

        // Create June budget with allocations
        let jun_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO budgets (id, year, month, notes, created_at, updated_at) VALUES (?1, 2026, 6, '', ?2, ?2)",
            params![jun_id, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO budget_items (id, budget_id, category_id, budgeted, rollover, created_at, updated_at)
             VALUES (?1, ?2, ?3, 50000, 1, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), jun_id, food_id, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO budget_items (id, budget_id, category_id, budgeted, rollover, created_at, updated_at)
             VALUES (?1, ?2, ?3, 20000, 0, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), jun_id, transport_id, now],
        ).unwrap();

        // Create July budget with items
        let jul_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO budgets (id, year, month, notes, created_at, updated_at) VALUES (?1, 2026, 7, '', ?2, ?2)",
            params![jul_id, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO budget_items (id, budget_id, category_id, budgeted, rollover, created_at, updated_at)
             VALUES (?1, ?2, ?3, 0, 0, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), jul_id, food_id, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO budget_items (id, budget_id, category_id, budgeted, rollover, created_at, updated_at)
             VALUES (?1, ?2, ?3, 0, 0, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), jul_id, transport_id, now],
        ).unwrap();

        // Copy Jun → Jul using UPDATE...FROM (matches production code)
        conn.execute(
            "UPDATE budget_items SET
                budgeted = src.budgeted,
                rollover = src.rollover,
                updated_at = ?1
             FROM (
                 SELECT category_id, budgeted, rollover
                 FROM budget_items
                 WHERE budget_id = ?2
             ) AS src
             WHERE budget_items.budget_id = ?3
               AND budget_items.category_id = src.category_id",
            params![now, jun_id, jul_id],
        ).unwrap();

        // Verify July allocations match June
        let jul_food: (i64, i64) = conn.query_row(
            "SELECT budgeted, rollover FROM budget_items WHERE budget_id = ?1 AND category_id = ?2",
            params![jul_id, food_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        ).unwrap();
        assert_eq!(jul_food, (50000, 1));

        let jul_transport: (i64, i64) = conn.query_row(
            "SELECT budgeted, rollover FROM budget_items WHERE budget_id = ?1 AND category_id = ?2",
            params![jul_id, transport_id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        ).unwrap();
        assert_eq!(jul_transport, (20000, 0));
    }

    #[test]
    fn test_budget_summary() {
        let conn = setup_db();
        let (food_id, transport_id, salary_id, account_id) = get_seeded_ids(&conn);
        let now = "2026-06-01T00:00:00Z";

        // Create budget with allocations for June
        let budget_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO budgets (id, year, month, notes, created_at, updated_at) VALUES (?1, 2026, 6, '', ?2, ?2)",
            params![budget_id, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO budget_items (id, budget_id, category_id, budgeted, rollover, created_at, updated_at)
             VALUES (?1, ?2, ?3, 50000, 0, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), budget_id, food_id, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO budget_items (id, budget_id, category_id, budgeted, rollover, created_at, updated_at)
             VALUES (?1, ?2, ?3, 20000, 0, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), budget_id, transport_id, now],
        ).unwrap();

        // Add income for June
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, category_id, payee, description, is_cleared, created_at, updated_at)
             VALUES (?1, ?2, '2026-06-01', 300000, 'income', ?3, 'Paycheck', '', 1, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), account_id, salary_id, now],
        ).unwrap();

        // Add expenses for June
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, category_id, payee, description, is_cleared, created_at, updated_at)
             VALUES (?1, ?2, '2026-06-05', 15000, 'expense', ?3, 'Grocery', '', 1, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), account_id, food_id, now],
        ).unwrap();

        // Verify summary
        let total_budgeted: i64 = conn.query_row(
            "SELECT COALESCE(SUM(bi.budgeted), 0)
             FROM budget_items bi JOIN budgets b ON b.id = bi.budget_id
             WHERE b.year = 2026 AND b.month = 6",
            [], |row| row.get(0),
        ).unwrap();
        assert_eq!(total_budgeted, 70000); // $500 + $200

        let total_income: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE kind = 'income' AND date LIKE '2026-06%'",
            [], |row| row.get(0),
        ).unwrap();
        assert_eq!(total_income, 300000); // $3000

        let to_be_budgeted = total_income - total_budgeted;
        assert_eq!(to_be_budgeted, 230000); // $3000 - $700 = $2300
    }

    #[test]
    fn rollover_twice_is_idempotent() {
        let conn = setup_db();
        let (food_id, _, _, account_id) = get_seeded_ids(&conn);
        let now = "2026-06-01T00:00:00Z";

        // June: $500 budgeted, rollover on, $300 spent -> $200 surplus
        let jun_id = uuid::Uuid::new_v4().to_string();
        let jul_id = uuid::Uuid::new_v4().to_string();
        for (id, month, rollover) in [(&jun_id, 6, 1), (&jul_id, 7, 0)] {
            conn.execute(
                "INSERT INTO budgets (id, year, month, notes, created_at, updated_at)
                 VALUES (?1, 2026, ?2, '', ?3, ?3)",
                params![id, month, now],
            ).unwrap();
            conn.execute(
                "INSERT INTO budget_items (id, budget_id, category_id, budgeted, rollover, created_at, updated_at)
                 VALUES (?1, ?2, ?3, 50000, ?4, ?5, ?5)",
                params![uuid::Uuid::new_v4().to_string(), id, food_id, rollover, now],
            ).unwrap();
        }
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, category_id, payee, description, is_cleared, created_at, updated_at)
             VALUES (?1, ?2, '2026-06-15', 30000, 'expense', ?3, 'Grocery', '', 1, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), account_id, food_id, now],
        ).unwrap();

        let read = || -> (i64, i64) {
            conn.query_row(
                "SELECT budgeted, rollover_amount FROM budget_items WHERE budget_id = ?1 AND category_id = ?2",
                params![jul_id, food_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            ).unwrap()
        };

        assert_eq!(super::apply_rollover(&conn, 2026, 7).unwrap(), 1);
        assert_eq!(read(), (50000, 20000));
        assert_eq!(super::apply_rollover(&conn, 2026, 7).unwrap(), 1);
        assert_eq!(read(), (50000, 20000));
    }

    #[test]
    fn budget_spent_includes_splits() {
        let conn = setup_db();
        let (food_id, transport_id, _, account_id) = get_seeded_ids(&conn);
        let now = "2026-06-01T00:00:00Z";

        // Plain $50 food expense
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, category_id, payee, description, is_cleared, created_at, updated_at)
             VALUES (?1, ?2, '2026-06-03', 5000, 'expense', ?3, 'Grocery', '', 1, ?4, ?4)",
            params![uuid::Uuid::new_v4().to_string(), account_id, food_id, now],
        ).unwrap();

        // Split $100 expense (parent categorised as food): $70 food + $30 transport
        let split_txn = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, category_id, payee, description, is_cleared, is_split, created_at, updated_at)
             VALUES (?1, ?2, '2026-06-10', 10000, 'expense', ?3, 'Mart', '', 1, 1, ?4, ?4)",
            params![split_txn, account_id, food_id, now],
        ).unwrap();
        for (cat, amount) in [(&food_id, 7000), (&transport_id, 3000)] {
            conn.execute(
                "INSERT INTO transaction_splits (id, transaction_id, category_id, amount, description, created_at)
                 VALUES (?1, ?2, ?3, ?4, '', ?5)",
                params![uuid::Uuid::new_v4().to_string(), split_txn, cat, amount, now],
            ).unwrap();
        }

        let mut stmt = conn.prepare(super::SPENT_BY_CATEGORY_SQL).unwrap();
        let spent: std::collections::HashMap<String, i64> = stmt
            .query_map(params!["2026-06%"], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        // Parent amount of the split txn must not be double counted under food.
        assert_eq!(spent.get(&food_id), Some(&12000));
        assert_eq!(spent.get(&transport_id), Some(&3000));
    }
}
