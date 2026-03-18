use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::timestamps::now_utc;
use crate::models::budget::{
    Budget, BudgetCategoryRow, BudgetSummary, BudgetWithItems, UpdateBudgetItemInput,
};
use tauri::State;

// ── Helpers ──────────────────────────────────────────────────────────

fn prev_year_month(year: i32, month: i32) -> (i32, i32) {
    if month == 1 { (year - 1, 12) } else { (year, month - 1) }
}

fn month_like_pattern(year: i32, month: i32) -> String {
    format!("{:04}-{:02}%", year, month)
}

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
    let now = now_utc();
    let budget_id = uuid::Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO budgets (id, year, month, notes, created_at, updated_at)
         VALUES (?1, ?2, ?3, '', ?4, ?4)",
        rusqlite::params![budget_id, year, month, now],
    )?;

    seed_budget_items(&conn, &budget_id, &now)?;

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

    let mut stmt = conn.prepare(
        "SELECT
            bi.id,
            bi.category_id,
            c.name,
            c.color,
            bi.budgeted,
            bi.rollover,
            COALESCE(t_sum.spent, 0) AS spent,
            c.group_id,
            cg.name
         FROM budget_items bi
         JOIN categories c ON c.id = bi.category_id
         LEFT JOIN category_groups cg ON c.group_id = cg.id
         LEFT JOIN (
             SELECT category_id, SUM(amount) AS spent
             FROM transactions
             WHERE kind = 'expense' AND date LIKE ?1
             GROUP BY category_id
         ) t_sum ON t_sum.category_id = bi.category_id
         WHERE bi.budget_id = ?2
         ORDER BY COALESCE(cg.sort_order, 999), c.sort_order",
    )?;

    let items: Vec<BudgetCategoryRow> = stmt
        .query_map(rusqlite::params![month_like_pattern(year, month), budget.id], |row| {
            let budgeted: i64 = row.get(4)?;
            let spent: i64 = row.get(6)?;
            Ok(BudgetCategoryRow {
                item_id: row.get(0)?,
                category_id: row.get(1)?,
                category_name: row.get(2)?,
                category_color: row.get(3)?,
                budgeted,
                spent,
                available: budgeted - spent,
                rollover: row.get::<_, i64>(5)? != 0,
                group_id: row.get(7)?,
                group_name: row.get(8)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(Some(BudgetWithItems { budget, items }))
}

/// Update a single budget item's budgeted amount and/or rollover flag.
#[tauri::command]
pub fn update_budget_item(
    item_id: String,
    input: UpdateBudgetItemInput,
    state: State<'_, DbState>,
) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    let now = now_utc();

    match (input.budgeted, input.rollover) {
        (Some(b), Some(r)) => {
            conn.execute(
                "UPDATE budget_items SET budgeted = ?1, rollover = ?2, updated_at = ?3 WHERE id = ?4",
                rusqlite::params![b, r as i64, now, item_id],
            )?;
        }
        (Some(b), None) => {
            conn.execute(
                "UPDATE budget_items SET budgeted = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![b, now, item_id],
            )?;
        }
        (None, Some(r)) => {
            conn.execute(
                "UPDATE budget_items SET rollover = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![r as i64, now, item_id],
            )?;
        }
        (None, None) => {}
    }

    Ok(())
}

/// Copy the previous month's budget allocations to the specified month.
/// Creates the budget if it doesn't exist, then copies budgeted amounts.
#[tauri::command]
pub fn copy_budget(year: i32, month: i32, state: State<'_, DbState>) -> AppResult<Budget> {
    let (prev_year, prev_month) = prev_year_month(year, month);

    let conn = state.write_conn.lock().unwrap();

    // Get previous month's budget
    let prev_budget_id: Option<String> = conn
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
    let existing_id: Option<String> = conn
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
            conn.execute(
                "INSERT INTO budgets (id, year, month, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, '', ?4, ?4)",
                rusqlite::params![id, year, month, now],
            )?;

            seed_budget_items(&conn, &id, &now)?;

            id
        }
    };

    // Copy allocations from previous month in a single UPDATE...FROM
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
        rusqlite::params![now, prev_budget_id, budget_id],
    )?;

    let budget = conn.query_row(
        "SELECT id, year, month, notes, created_at, updated_at FROM budgets WHERE id = ?1",
        [&budget_id],
        row_to_budget,
    )?;

    Ok(budget)
}

/// Get budget summary for a given month: totals and "to be budgeted".
#[tauri::command]
pub fn get_budget_summary(year: i32, month: i32, state: State<'_, DbState>) -> AppResult<BudgetSummary> {
    let conn = state.read_conn.lock().unwrap();
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
        total_available: total_budgeted - total_spent,
        total_income,
        to_be_budgeted: total_income - total_budgeted,
    })
}

/// Calculate rollover from previous month.
/// For each category with rollover=1 in the previous month, carry forward (budgeted - spent).
/// Adds the rollover amount to the current month's budgeted value.
#[tauri::command]
pub fn calculate_rollover(year: i32, month: i32, state: State<'_, DbState>) -> AppResult<i32> {
    let (prev_year, prev_month) = prev_year_month(year, month);

    let conn = state.write_conn.lock().unwrap();
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

    let mut stmt = conn.prepare(
        "SELECT bi.category_id, bi.budgeted,
                COALESCE(t_sum.spent, 0) AS spent
         FROM budget_items bi
         LEFT JOIN (
             SELECT category_id, SUM(amount) AS spent
             FROM transactions
             WHERE kind = 'expense' AND date LIKE ?1
             GROUP BY category_id
         ) t_sum ON t_sum.category_id = bi.category_id
         WHERE bi.budget_id = ?2 AND bi.rollover = 1",
    )?;

    let rollovers: Vec<(String, i64)> = stmt
        .query_map(rusqlite::params![month_like_pattern(prev_year, prev_month), prev_budget_id], |row| {
            let budgeted: i64 = row.get(1)?;
            let spent: i64 = row.get(2)?;
            let rollover_amount = budgeted - spent; // positive = surplus, negative = overspent
            Ok((row.get::<_, String>(0)?, rollover_amount))
        })?
        .collect::<Result<_, _>>()?;

    let mut count = 0;
    for (cat_id, rollover_amount) in &rollovers {
        // Add rollover to current month's budgeted amount
        let updated = conn.execute(
            "UPDATE budget_items SET budgeted = budgeted + ?1, updated_at = ?2
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
    use crate::db::migrations::run_migrations;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();
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
}
