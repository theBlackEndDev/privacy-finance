use std::collections::HashMap;
use chrono::NaiveDate;
use tauri::State;
use crate::db::DbState;
use crate::error::AppResult;
use crate::models::report::*;

const MONTHLY_INCOME_EXPENSE_SQL: &str =
    "SELECT
        strftime('%Y-%m', date) AS month,
        COALESCE(SUM(CASE WHEN kind = 'income' THEN amount ELSE 0 END), 0) AS income,
        COALESCE(SUM(CASE WHEN kind = 'expense' THEN amount ELSE 0 END), 0) AS expense
     FROM transactions
     WHERE date >= ?1 AND date <= ?2
       AND kind != 'transfer'
     GROUP BY month
     ORDER BY month ASC";

/// Split-aware category expense subquery: non-split by parent category UNION ALL split by split category.
fn category_expense_sql(date_filter: &str) -> String {
    format!(
        "SELECT category_id, category_name, category_color, SUM(total) AS total
         FROM (
             SELECT
                 t.category_id,
                 COALESCE(c.name, 'Uncategorized') AS category_name,
                 COALESCE(c.color, '#6b7280') AS category_color,
                 t.amount AS total
             FROM transactions t
             LEFT JOIN categories c ON t.category_id = c.id
             WHERE {filter}
               AND t.kind = 'expense'
               AND t.is_split = 0
             UNION ALL
             SELECT
                 ts.category_id,
                 COALESCE(c.name, 'Uncategorized') AS category_name,
                 COALESCE(c.color, '#6b7280') AS category_color,
                 ts.amount AS total
             FROM transactions t
             JOIN transaction_splits ts ON t.id = ts.transaction_id
             LEFT JOIN categories c ON ts.category_id = c.id
             WHERE {filter}
               AND t.kind = 'expense'
               AND t.is_split = 1
         )
         GROUP BY category_id
         ORDER BY total DESC",
        filter = date_filter
    )
}

fn query_monthly_income_expense(
    conn: &rusqlite::Connection,
    start: &str,
    end: &str,
) -> rusqlite::Result<Vec<(String, i64, i64)>> {
    let mut stmt = conn.prepare(MONTHLY_INCOME_EXPENSE_SQL)?;
    let rows = stmt
        .query_map(rusqlite::params![start, end], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn parse_date(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap_or(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
}

#[tauri::command]
pub fn get_income_expense_report(
    range: ReportDateRange,
    state: State<DbState>,
) -> AppResult<IncomeExpenseReport> {
    let conn = state.read_conn.lock().unwrap();

    let rows = query_monthly_income_expense(&conn, &range.start_date, &range.end_date)?;
    let months: Vec<IncomeExpenseMonth> = rows.into_iter().map(|(month, income, expense)| {
        IncomeExpenseMonth { month, income, expense, net: income - expense }
    }).collect();

    // Previous period: same duration shifted back
    let start = parse_date(&range.start_date);
    let end = parse_date(&range.end_date);
    let days = (end - start).num_days().max(0);
    let prev_end = start - chrono::Duration::days(1);
    let prev_start = start - chrono::Duration::days(days + 1);

    let (previous_income, previous_expense): (i64, i64) = conn
        .query_row(
            "SELECT
                COALESCE(SUM(CASE WHEN kind = 'income' THEN amount ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN kind = 'expense' THEN amount ELSE 0 END), 0)
             FROM transactions
             WHERE date >= ?1 AND date <= ?2
               AND kind != 'transfer'",
            rusqlite::params![prev_start.format("%Y-%m-%d").to_string(), prev_end.format("%Y-%m-%d").to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

    Ok(IncomeExpenseReport {
        months,
        previous_income,
        previous_expense,
    })
}

#[tauri::command]
pub fn get_category_spending_report(
    range: ReportDateRange,
    state: State<DbState>,
) -> AppResult<Vec<CategorySpendingRow>> {
    let conn = state.read_conn.lock().unwrap();

    let sql = category_expense_sql("t.date >= ?1 AND t.date <= ?2");
    let mut stmt = conn.prepare(&sql)?;

    let rows: Vec<(Option<String>, String, String, i64)> = stmt
        .query_map(rusqlite::params![range.start_date, range.end_date], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<Result<_, _>>()?;

    let grand_total: i64 = rows.iter().map(|r| r.3).sum();

    let result = rows.into_iter().map(|(cat_id, name, color, total)| {
        CategorySpendingRow {
            category_id: cat_id,
            category_name: name,
            category_color: color,
            total,
            percentage: if grand_total > 0 { (total as f64 / grand_total as f64) * 100.0 } else { 0.0 },
        }
    }).collect();

    Ok(result)
}

#[tauri::command]
pub fn get_cash_flow_report(
    range: ReportDateRange,
    state: State<DbState>,
) -> AppResult<Vec<CashFlowPoint>> {
    let conn = state.read_conn.lock().unwrap();

    let rows = query_monthly_income_expense(&conn, &range.start_date, &range.end_date)?;

    let mut cumulative: i64 = 0;
    let points = rows.into_iter().map(|(month, income, expense)| {
        let net = income - expense;
        cumulative += net;
        CashFlowPoint {
            date: month,
            income,
            expense,
            net,
            cumulative,
        }
    }).collect();

    Ok(points)
}

#[tauri::command]
pub fn get_net_worth_report(
    range: ReportDateRange,
    state: State<DbState>,
) -> AppResult<Vec<NetWorthPoint>> {
    let conn = state.read_conn.lock().unwrap();

    // Get all non-archived accounts with opening balances
    let mut acct_stmt = conn.prepare(
        "SELECT id, name, opening_balance FROM accounts WHERE is_archived = 0 ORDER BY name ASC"
    )?;
    let accounts: Vec<(String, String, i64)> = acct_stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<Result<_, _>>()?;

    if accounts.is_empty() {
        return Ok(vec![]);
    }

    // Single query: get cumulative transaction delta per account per month (up to end of range)
    let mut txn_stmt = conn.prepare(
        "SELECT
            account_id,
            strftime('%Y-%m', date) AS month,
            COALESCE(SUM(
                CASE
                    WHEN kind = 'income' THEN amount
                    WHEN kind = 'expense' THEN -amount
                    ELSE amount
                END
            ), 0) AS delta
         FROM transactions
         WHERE date <= ?1
         GROUP BY account_id, month
         ORDER BY account_id, month ASC"
    )?;
    // Map: (account_id, month) -> delta for that month
    let mut monthly_deltas: HashMap<(String, String), i64> = HashMap::new();
    let mut all_months: Vec<String> = Vec::new();
    txn_stmt.query_map(rusqlite::params![range.end_date], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?))
    })?.for_each(|r| {
        if let Ok((acct_id, month, delta)) = r {
            if !all_months.contains(&month) {
                all_months.push(month.clone());
            }
            monthly_deltas.insert((acct_id, month), delta);
        }
    });

    all_months.sort();

    // Filter to months within range
    let start_month = &range.start_date[..7]; // YYYY-MM
    let end_month = &range.end_date[..7];
    let months_in_range: Vec<&String> = all_months.iter()
        .filter(|m| m.as_str() >= start_month && m.as_str() <= end_month)
        .collect();

    // Build running balances per account across all months
    let mut running: HashMap<String, i64> = accounts.iter()
        .map(|(id, _, opening)| (id.clone(), *opening))
        .collect();

    let mut points = Vec::new();
    for month in &all_months {
        // Accumulate this month's deltas
        for (acct_id, _, _) in &accounts {
            if let Some(delta) = monthly_deltas.get(&(acct_id.clone(), month.clone())) {
                *running.entry(acct_id.clone()).or_insert(0) += delta;
            }
        }

        // Only emit points within the requested range
        if months_in_range.contains(&month) {
            let mut account_balances = Vec::new();
            let mut total: i64 = 0;
            for (acct_id, acct_name, _) in &accounts {
                let balance = *running.get(acct_id).unwrap_or(&0);
                total += balance;
                account_balances.push(NetWorthAccount {
                    account_id: acct_id.clone(),
                    account_name: acct_name.clone(),
                    balance,
                });
            }
            points.push(NetWorthPoint {
                month: month.clone(),
                total,
                accounts: account_balances,
            });
        }
    }

    Ok(points)
}

#[tauri::command]
pub fn get_tag_spending_report(
    range: ReportDateRange,
    state: State<DbState>,
) -> AppResult<Vec<TagSpendingRow>> {
    let conn = state.read_conn.lock().unwrap();

    let mut stmt = conn.prepare(
        "SELECT tg.id, tg.name, tg.color, COALESCE(SUM(t.amount), 0) AS total
         FROM transaction_tags tt
         JOIN tags tg ON tt.tag_id = tg.id
         JOIN transactions t ON tt.transaction_id = t.id
         WHERE t.date >= ?1 AND t.date <= ?2
           AND t.kind = 'expense'
         GROUP BY tg.id
         ORDER BY total DESC"
    )?;
    let rows = stmt
        .query_map(rusqlite::params![range.start_date, range.end_date], |row| {
            Ok(TagSpendingRow {
                tag_id: row.get(0)?,
                tag_name: row.get(1)?,
                tag_color: row.get(2)?,
                total: row.get(3)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(rows)
}

#[tauri::command]
pub fn get_payee_spending_report(
    range: ReportDateRange,
    limit: Option<u32>,
    state: State<DbState>,
) -> AppResult<Vec<PayeeSpendingRow>> {
    let conn = state.read_conn.lock().unwrap();
    let max = limit.unwrap_or(20) as i64;

    let mut stmt = conn.prepare(
        "SELECT payee, COALESCE(SUM(amount), 0) AS total, COUNT(*) AS cnt
         FROM transactions
         WHERE date >= ?1 AND date <= ?2
           AND kind = 'expense'
           AND payee != ''
         GROUP BY payee
         ORDER BY total DESC
         LIMIT ?3"
    )?;
    let rows = stmt
        .query_map(rusqlite::params![range.start_date, range.end_date, max], |row| {
            Ok(PayeeSpendingRow {
                payee: row.get(0)?,
                total: row.get(1)?,
                count: row.get(2)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::db::migrations::{run_migrations, seed_test_data};
    use uuid::Uuid;
    use crate::helpers::timestamps::now_utc;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();
        seed_test_data(&conn).unwrap();
        conn
    }

    fn create_test_account(conn: &Connection) -> String {
        let id = Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO accounts (id, name, type, currency_code, opening_balance, sort_order, created_at, updated_at)
             VALUES (?1, 'Test Account', 'checking', 'USD', 0, 99, ?2, ?2)",
            rusqlite::params![id, now],
        ).unwrap();
        id
    }

    fn create_test_category(conn: &Connection, name: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO categories (id, name, color, icon, type, sort_order, created_at, updated_at)
             VALUES (?1, ?2, '#ff0000', 'folder', 'expense', 0, ?3, ?3)",
            rusqlite::params![id, name, now],
        ).unwrap();
        id
    }

    fn insert_transaction(conn: &Connection, account_id: &str, date: &str, amount: i64, kind: &str, payee: &str, category_id: Option<&str>) -> String {
        let id = Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, payee, description, is_cleared, created_at, updated_at, category_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, '', 0, ?7, ?7, ?8)",
            rusqlite::params![id, account_id, date, amount, kind, payee, now, category_id],
        ).unwrap();
        id
    }

    #[test]
    fn test_income_expense_report() {
        let conn = setup();
        let acct = create_test_account(&conn);

        insert_transaction(&conn, &acct, "2030-01-15", 100000, "income", "Salary", None);
        insert_transaction(&conn, &acct, "2030-01-20", 30000, "expense", "Groceries", None);
        insert_transaction(&conn, &acct, "2030-02-10", 100000, "income", "Salary", None);
        insert_transaction(&conn, &acct, "2030-02-15", 45000, "expense", "Rent", None);

        let rows = query_monthly_income_expense(&conn, "2030-01-01", "2030-02-28").unwrap();
        let months: Vec<IncomeExpenseMonth> = rows.into_iter().map(|(month, income, expense)| {
            IncomeExpenseMonth { month, income, expense, net: income - expense }
        }).collect();

        assert_eq!(months.len(), 2);
        assert_eq!(months[0].income, 100000);
        assert_eq!(months[0].expense, 30000);
        assert_eq!(months[0].net, 70000);
        assert_eq!(months[1].income, 100000);
        assert_eq!(months[1].expense, 45000);
    }

    #[test]
    fn test_category_spending_percentages_sum_to_100() {
        let conn = setup();
        let acct = create_test_account(&conn);
        let cat1 = create_test_category(&conn, "RptFood");
        let cat2 = create_test_category(&conn, "RptTransport");
        let cat3 = create_test_category(&conn, "RptEntertain");

        insert_transaction(&conn, &acct, "2030-01-10", 5000, "expense", "Store", Some(&cat1));
        insert_transaction(&conn, &acct, "2030-01-12", 3000, "expense", "Bus", Some(&cat2));
        insert_transaction(&conn, &acct, "2030-01-14", 2000, "expense", "Movie", Some(&cat3));

        let sql = category_expense_sql("t.date >= ?1 AND t.date <= ?2");
        let mut stmt = conn.prepare(&sql).unwrap();

        let rows: Vec<(Option<String>, String, String, i64)> = stmt.query_map(
            rusqlite::params!["2030-01-01", "2030-01-31"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        ).unwrap().collect::<Result<_, _>>().unwrap();

        let grand_total: i64 = rows.iter().map(|r| r.3).sum();
        let percentages: Vec<f64> = rows.iter().map(|r| {
            if grand_total > 0 { (r.3 as f64 / grand_total as f64) * 100.0 } else { 0.0 }
        }).collect();

        let sum: f64 = percentages.iter().sum();
        assert!((sum - 100.0).abs() < 0.01, "Percentages should sum to 100, got {}", sum);
        assert_eq!(grand_total, 10000);
    }

    #[test]
    fn test_net_worth_calculation() {
        let conn = setup();
        let acct = create_test_account(&conn);

        insert_transaction(&conn, &acct, "2030-01-15", 100000, "income", "Salary", None);
        insert_transaction(&conn, &acct, "2030-01-20", 30000, "expense", "Groceries", None);

        let expected: i64 = 100000 - 30000;

        let balance: i64 = conn.query_row(
            "SELECT COALESCE(SUM(
                CASE WHEN kind = 'income' THEN amount
                     WHEN kind = 'expense' THEN -amount
                     ELSE amount END
            ), 0) FROM transactions WHERE account_id = ?1 AND date <= '2030-01-31'",
            [&acct], |r| r.get(0)
        ).unwrap();

        assert_eq!(balance, expected);
    }

    #[test]
    fn test_cash_flow_cumulative() {
        let conn = setup();
        let acct = create_test_account(&conn);

        insert_transaction(&conn, &acct, "2030-01-15", 100000, "income", "Salary", None);
        insert_transaction(&conn, &acct, "2030-01-20", 30000, "expense", "Groceries", None);
        insert_transaction(&conn, &acct, "2030-02-15", 100000, "income", "Salary", None);
        insert_transaction(&conn, &acct, "2030-02-20", 50000, "expense", "Rent", None);

        let rows = query_monthly_income_expense(&conn, "2030-01-01", "2030-02-28").unwrap();

        let mut cumulative: i64 = 0;
        let points: Vec<CashFlowPoint> = rows.into_iter().map(|(month, income, expense)| {
            let net = income - expense;
            cumulative += net;
            CashFlowPoint { date: month, income, expense, net, cumulative }
        }).collect();

        assert_eq!(points.len(), 2);
        assert_eq!(points[0].cumulative, 70000);
        assert_eq!(points[1].cumulative, 120000);
    }
}
