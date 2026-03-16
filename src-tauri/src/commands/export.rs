use std::io::Write;
use tauri::State;
use crate::db::DbState;
use crate::error::AppResult;

// ── Export filters ──────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFilters {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub account_id: Option<String>,
}

// ── export_transactions_csv ─────────────────────────────────────────

#[tauri::command]
pub fn export_transactions_csv(
    path: String,
    filters: ExportFilters,
    state: State<DbState>,
) -> AppResult<u64> {
    let conn = state.read_conn.lock().unwrap();
    let (where_clause, params) = build_filter_clause(&filters);

    let sql = format!(
        "SELECT t.date, a.name AS account_name, t.payee, t.description,
                COALESCE(c.name, '') AS category_name,
                t.amount, t.kind,
                CASE WHEN t.is_cleared THEN 'Yes' ELSE 'No' END AS cleared
         FROM transactions t
         JOIN accounts a ON a.id = t.account_id
         LEFT JOIN categories c ON c.id = t.category_id
         {where_clause}
         ORDER BY t.date DESC, t.created_at DESC"
    );

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params
        .iter()
        .map(|s| s as &dyn rusqlite::types::ToSql)
        .collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row: &rusqlite::Row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, String>(7)?,
        ))
    })?;

    let file = std::fs::File::create(&path)?;
    let mut wtr = csv::Writer::from_writer(file);
    wtr.write_record(["Date", "Account", "Payee", "Description", "Category", "Amount", "Kind", "Cleared"])?;

    let mut count: u64 = 0;
    for row in rows {
        let (date, account, payee, desc, category, amount, kind, cleared) = row?;
        let amount_str = format_amount(amount);
        wtr.write_record([&date, &account, &payee, &desc, &category, &amount_str, &kind, &cleared])?;
        count += 1;
    }
    wtr.flush()?;
    Ok(count)
}

// ── export_transactions_json ────────────────────────────────────────

#[tauri::command]
pub fn export_transactions_json(
    path: String,
    filters: ExportFilters,
    state: State<DbState>,
) -> AppResult<u64> {
    let conn = state.read_conn.lock().unwrap();
    let (where_clause, params) = build_filter_clause(&filters);

    let sql = format!(
        "SELECT t.id, t.account_id, t.date, t.amount, t.kind,
                t.category_id, t.payee, t.description, t.is_cleared,
                t.transfer_group_id, t.import_batch_id,
                t.created_at, t.updated_at
         FROM transactions t
         {where_clause}
         ORDER BY t.date DESC, t.created_at DESC"
    );

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params
        .iter()
        .map(|s| s as &dyn rusqlite::types::ToSql)
        .collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row: &rusqlite::Row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "accountId": row.get::<_, String>(1)?,
            "date": row.get::<_, String>(2)?,
            "amount": row.get::<_, i64>(3)?,
            "kind": row.get::<_, String>(4)?,
            "categoryId": row.get::<_, Option<String>>(5)?,
            "payee": row.get::<_, String>(6)?,
            "description": row.get::<_, String>(7)?,
            "isCleared": row.get::<_, i64>(8)? != 0,
            "transferGroupId": row.get::<_, Option<String>>(9)?,
            "importBatchId": row.get::<_, Option<String>>(10)?,
            "createdAt": row.get::<_, String>(11)?,
            "updatedAt": row.get::<_, String>(12)?,
        }))
    })?;

    let file = std::fs::File::create(&path)?;
    let mut wtr = std::io::BufWriter::new(file);
    wtr.write_all(b"[\n")?;
    let mut count: u64 = 0;
    for row in rows {
        if count > 0 { wtr.write_all(b",\n")?; }
        let val = row?;
        serde_json::to_writer(&mut wtr, &val)
            .map_err(|e| crate::error::AppError::Validation(e.to_string()))?;
        count += 1;
    }
    wtr.write_all(b"\n]")?;
    wtr.flush()?;
    Ok(count)
}

// ── export_all_data_json ────────────────────────────────────────────

#[tauri::command]
pub fn export_all_data_json(
    path: String,
    state: State<DbState>,
) -> AppResult<()> {
    let conn = state.read_conn.lock().unwrap();
    let file = std::fs::File::create(&path)?;
    let mut wtr = std::io::BufWriter::new(file);

    write!(wtr, "{{\n  \"exportedAt\": \"{}\",\n", chrono::Utc::now().to_rfc3339())?;

    let tables: &[(&str, &str)] = &[
        ("accounts", "SELECT * FROM accounts ORDER BY sort_order"),
        ("categories", "SELECT * FROM categories ORDER BY sort_order"),
        ("transactions", "SELECT * FROM transactions ORDER BY date DESC"),
        ("recurringTransactions", "SELECT * FROM recurring_transactions ORDER BY created_at"),
    ];

    for (i, (key, sql)) in tables.iter().enumerate() {
        write!(wtr, "  \"{}\": ", key)?;
        stream_table_json(&conn, sql, &mut wtr)?;
        if i < tables.len() - 1 {
            wtr.write_all(b",\n")?;
        } else {
            wtr.write_all(b"\n")?;
        }
    }

    wtr.write_all(b"}\n")?;
    wtr.flush()?;
    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────────

fn build_filter_clause(filters: &ExportFilters) -> (String, Vec<String>) {
    let mut conditions = Vec::new();
    let mut params = Vec::new();

    if let Some(ref from) = filters.date_from {
        params.push(from.clone());
        conditions.push(format!("t.date >= ?{}", params.len()));
    }
    if let Some(ref to) = filters.date_to {
        params.push(to.clone());
        conditions.push(format!("t.date <= ?{}", params.len()));
    }
    if let Some(ref account_id) = filters.account_id {
        params.push(account_id.clone());
        conditions.push(format!("t.account_id = ?{}", params.len()));
    }

    let clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    (clause, params)
}

fn format_amount(cents: i64) -> String {
    let dollars = cents as f64 / 100.0;
    format!("{:.2}", dollars)
}

fn row_to_json(
    row: &rusqlite::Row,
    col_names: &[String],
) -> rusqlite::Result<serde_json::Value> {
    let mut map = serde_json::Map::with_capacity(col_names.len());
    for (i, name) in col_names.iter().enumerate() {
        let val = match row.get_ref(i) {
            Ok(rusqlite::types::ValueRef::Null) => serde_json::Value::Null,
            Ok(rusqlite::types::ValueRef::Integer(n)) => serde_json::json!(n),
            Ok(rusqlite::types::ValueRef::Real(f)) => serde_json::json!(f),
            Ok(rusqlite::types::ValueRef::Text(s)) => {
                serde_json::Value::String(String::from_utf8_lossy(s).to_string())
            }
            Ok(rusqlite::types::ValueRef::Blob(b)) => {
                serde_json::Value::String(format!("<blob {} bytes>", b.len()))
            }
            Err(_) => serde_json::Value::Null,
        };
        map.insert(name.clone(), val);
    }
    Ok(serde_json::Value::Object(map))
}

fn stream_table_json(
    conn: &rusqlite::Connection,
    sql: &str,
    wtr: &mut impl Write,
) -> AppResult<()> {
    let mut stmt = conn.prepare(sql)?;
    let col_count = stmt.column_count();
    let col_names: Vec<String> = (0..col_count)
        .map(|i| stmt.column_name(i).unwrap_or("").to_string())
        .collect();

    let rows = stmt.query_map([], |row| row_to_json(row, &col_names))?;
    wtr.write_all(b"[")?;
    let mut first = true;
    for row in rows {
        if !first { wtr.write_all(b",")?; }
        first = false;
        let val = row?;
        serde_json::to_writer(&mut *wtr, &val)
            .map_err(|e| crate::error::AppError::Validation(e.to_string()))?;
    }
    wtr.write_all(b"]")?;
    Ok(())
}

#[cfg(test)]
fn query_table_json(
    conn: &rusqlite::Connection,
    sql: &str,
) -> AppResult<Vec<serde_json::Value>> {
    let mut buf = Vec::new();
    stream_table_json(conn, sql, &mut buf)?;
    let parsed: Vec<serde_json::Value> = serde_json::from_slice(&buf)
        .map_err(|e| crate::error::AppError::Validation(e.to_string()))?;
    Ok(parsed)
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::db::migrations::run_migrations;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    fn get_account_id(conn: &Connection) -> String {
        conn.query_row("SELECT id FROM accounts LIMIT 1", [], |row| row.get(0)).unwrap()
    }

    fn insert_test_tx(conn: &Connection, account_id: &str, date: &str, payee: &str, amount: i64) {
        let now = "2026-01-01T00:00:00Z";
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, payee, description, is_cleared, created_at, updated_at)
             VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, 'expense', ?4, 'test', 0, ?5, ?5)",
            rusqlite::params![account_id, date, amount, payee, now],
        ).unwrap();
    }

    #[test]
    fn test_csv_export_format() {
        let conn = setup_db();
        let account_id = get_account_id(&conn);
        insert_test_tx(&conn, &account_id, "2026-01-15", "Netflix", 1599);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.csv");

        let filters = ExportFilters { date_from: None, date_to: None, account_id: None };
        let (where_clause, params) = build_filter_clause(&filters);
        let sql = format!(
            "SELECT t.date, a.name AS account_name, t.payee, t.description,
                    COALESCE(c.name, '') AS category_name,
                    t.amount, t.kind,
                    CASE WHEN t.is_cleared THEN 'Yes' ELSE 'No' END AS cleared
             FROM transactions t
             JOIN accounts a ON a.id = t.account_id
             LEFT JOIN categories c ON c.id = t.category_id
             {where_clause}
             ORDER BY t.date DESC, t.created_at DESC"
        );
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
        let mut stmt = conn.prepare(&sql).unwrap();
        let rows = stmt.query_map(param_refs.as_slice(), |row: &rusqlite::Row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        }).unwrap();

        let file = std::fs::File::create(path.to_str().unwrap()).unwrap();
        let mut wtr = csv::Writer::from_writer(file);
        wtr.write_record(["Date", "Account", "Payee", "Description", "Category", "Amount", "Kind", "Cleared"]).unwrap();
        for row in rows {
            let (date, account, payee, desc, category, amount, kind, cleared) = row.unwrap();
            wtr.write_record([&date, &account, &payee, &desc, &category, &format_amount(amount), &kind, &cleared]).unwrap();
        }
        wtr.flush().unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert!(lines[0].contains("Date"));
        assert!(lines[0].contains("Payee"));
        assert!(lines[0].contains("Amount"));
        // Find the Netflix line
        let netflix_line = lines.iter().find(|l| l.contains("Netflix"));
        assert!(netflix_line.is_some(), "Netflix transaction should be in CSV");
        assert!(netflix_line.unwrap().contains("15.99"));
    }

    #[test]
    fn test_json_export_valid() {
        let conn = setup_db();
        let account_id = get_account_id(&conn);
        insert_test_tx(&conn, &account_id, "2026-02-01", "Grocery", 4500);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.json");

        // Export using direct query (matching command logic without State)
        let filters = ExportFilters { date_from: None, date_to: None, account_id: None };
        let (where_clause, params) = build_filter_clause(&filters);
        let sql = format!(
            "SELECT t.id, t.account_id, t.date, t.amount, t.kind,
                    t.category_id, t.payee, t.description, t.is_cleared,
                    t.transfer_group_id, t.import_batch_id,
                    t.created_at, t.updated_at
             FROM transactions t {where_clause}
             ORDER BY t.date DESC, t.created_at DESC"
        );
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
        let mut stmt = conn.prepare(&sql).unwrap();
        let rows = stmt.query_map(param_refs.as_slice(), |row: &rusqlite::Row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "date": row.get::<_, String>(2)?,
                "amount": row.get::<_, i64>(3)?,
                "payee": row.get::<_, String>(6)?,
            }))
        }).unwrap();
        let txs: Vec<serde_json::Value> = rows.filter_map(|r| r.ok()).collect();
        let json = serde_json::to_string_pretty(&txs).unwrap();
        std::fs::write(&path, &json).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_array());
        let arr = parsed.as_array().unwrap();
        let grocery = arr.iter().find(|v| v["payee"] == "Grocery");
        assert!(grocery.is_some(), "Grocery transaction should be in JSON");
        assert_eq!(grocery.unwrap()["amount"], 4500);
    }

    #[test]
    fn test_csv_export_date_filter() {
        let conn = setup_db();
        let account_id = get_account_id(&conn);
        insert_test_tx(&conn, &account_id, "2026-01-10", "Before", 100);
        insert_test_tx(&conn, &account_id, "2026-01-20", "During", 200);
        insert_test_tx(&conn, &account_id, "2026-02-05", "After", 300);

        let filters = ExportFilters {
            date_from: Some("2026-01-15".to_string()),
            date_to: Some("2026-01-31".to_string()),
            account_id: None,
        };
        let (where_clause, params) = build_filter_clause(&filters);

        let sql = format!(
            "SELECT t.payee FROM transactions t {where_clause}"
        );
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
        let mut stmt = conn.prepare(&sql).unwrap();
        let payees: Vec<String> = stmt.query_map(param_refs.as_slice(), |row: &rusqlite::Row| {
            row.get(0)
        }).unwrap().filter_map(|r| r.ok()).collect();

        assert!(payees.contains(&"During".to_string()));
        assert!(!payees.contains(&"Before".to_string()));
        assert!(!payees.contains(&"After".to_string()));
    }

    #[test]
    fn test_full_data_export() {
        let conn = setup_db();
        let accounts = query_table_json(&conn, "SELECT * FROM accounts").unwrap();
        let categories = query_table_json(&conn, "SELECT * FROM categories").unwrap();
        let transactions = query_table_json(&conn, "SELECT * FROM transactions").unwrap();

        assert!(!accounts.is_empty(), "Should have seed accounts");
        assert!(!categories.is_empty(), "Should have seed categories");
        assert!(!transactions.is_empty(), "Should have seed transactions");
    }

    #[test]
    fn test_format_amount() {
        assert_eq!(format_amount(1599), "15.99");
        assert_eq!(format_amount(0), "0.00");
        assert_eq!(format_amount(100), "1.00");
        assert_eq!(format_amount(-2550), "-25.50");
    }
}
