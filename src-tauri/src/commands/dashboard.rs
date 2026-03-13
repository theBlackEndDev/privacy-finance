use crate::db::DbState;
use crate::error::AppResult;
use crate::models::dashboard::{AccountBalance, CategorySummary, DashboardData, MonthSummary};
use tauri::State;

/// Returns income/expense/net for a given month (YYYY-MM) plus spending by category.
/// Excludes transfer transactions.
#[tauri::command]
pub fn get_dashboard_data(month: String, state: State<'_, DbState>) -> AppResult<DashboardData> {
    let conn = state.read_conn.lock().unwrap();

    let pattern = format!("{}%", month);

    let mut stmt = conn.prepare(
        "SELECT
            COALESCE(SUM(CASE WHEN amount > 0 THEN amount ELSE 0 END), 0) AS income,
            COALESCE(SUM(CASE WHEN amount < 0 THEN amount ELSE 0 END), 0) AS expense
         FROM transactions
         WHERE date LIKE ?1
           AND kind != 'transfer'",
    )?;
    let (income, expense): (i64, i64) =
        stmt.query_row([&pattern], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let net = income + expense;

    let mut cat_stmt = conn.prepare(
        "SELECT
            t.category_id,
            COALESCE(c.name, 'Uncategorized') AS category_name,
            COALESCE(c.color, '#6b7280') AS category_color,
            SUM(t.amount) AS total
         FROM transactions t
         LEFT JOIN categories c ON t.category_id = c.id
         WHERE t.date LIKE ?1
           AND t.amount < 0
           AND t.kind != 'transfer'
         GROUP BY t.category_id
         ORDER BY total ASC",
    )?;
    let by_category: Vec<CategorySummary> = cat_stmt
        .query_map([&pattern], |row| {
            Ok(CategorySummary {
                category_id: row.get(0)?,
                category_name: row.get(1)?,
                category_color: row.get(2)?,
                total: row.get(3)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(DashboardData {
        income,
        expense,
        net,
        by_category,
    })
}

/// Returns monthly income/expense/net for the last `months` months.
#[tauri::command]
pub fn get_monthly_trend(months: u32, state: State<'_, DbState>) -> AppResult<Vec<MonthSummary>> {
    let conn = state.read_conn.lock().unwrap();
    let limit = months.min(60) as i64;

    let mut stmt = conn.prepare(
        "SELECT
            strftime('%Y-%m', date) AS month,
            COALESCE(SUM(CASE WHEN amount > 0 THEN amount ELSE 0 END), 0) AS income,
            COALESCE(SUM(CASE WHEN amount < 0 THEN amount ELSE 0 END), 0) AS expense,
            COALESCE(SUM(amount), 0) AS net
         FROM transactions
         WHERE kind != 'transfer'
         GROUP BY month
         ORDER BY month DESC
         LIMIT ?1",
    )?;
    let rows: Vec<MonthSummary> = stmt
        .query_map([limit], |row| {
            Ok(MonthSummary {
                month: row.get(0)?,
                income: row.get(1)?,
                expense: row.get(2)?,
                net: row.get(3)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    let mut result = rows;
    result.reverse();
    Ok(result)
}

/// Returns current balance for each account (opening_balance + sum of transactions).
#[tauri::command]
pub fn get_account_balances(state: State<'_, DbState>) -> AppResult<Vec<AccountBalance>> {
    let conn = state.read_conn.lock().unwrap();

    let mut stmt = conn.prepare(
        "SELECT
            a.id,
            a.name,
            a.type,
            a.opening_balance + COALESCE(SUM(t.amount), 0) AS balance
         FROM accounts a
         LEFT JOIN transactions t ON t.account_id = a.id
         GROUP BY a.id
         ORDER BY a.name ASC",
    )?;
    let rows: Vec<AccountBalance> = stmt
        .query_map([], |row| {
            Ok(AccountBalance {
                account_id: row.get(0)?,
                account_name: row.get(1)?,
                account_type: row.get(2)?,
                balance: row.get(3)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(rows)
}
