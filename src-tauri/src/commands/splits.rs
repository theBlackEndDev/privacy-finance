use tauri::State;
use uuid::Uuid;
use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::helpers::timestamps::now_utc;
use crate::models::split::{CreateSplitInput, TransactionSplit};

const SPLIT_COLS: &str = "
    ts.id, ts.transaction_id, ts.category_id,
    ts.amount, ts.description, ts.created_at,
    c.name AS category_name, c.color AS category_color
";

fn row_to_split(row: &rusqlite::Row<'_>) -> rusqlite::Result<TransactionSplit> {
    Ok(TransactionSplit {
        id: row.get(0)?,
        transaction_id: row.get(1)?,
        category_id: row.get(2)?,
        amount: row.get(3)?,
        description: row.get(4)?,
        created_at: row.get(5)?,
        category_name: row.get(6)?,
        category_color: row.get(7)?,
    })
}

fn validate_split_sum(
    conn: &rusqlite::Connection,
    transaction_id: &str,
    splits: &[CreateSplitInput],
) -> AppResult<()> {
    let parent_amount: i64 = conn
        .query_row(
            "SELECT amount FROM transactions WHERE id = ?1",
            [transaction_id],
            |row| row.get(0),
        )
        .map_err(|_| AppError::NotFound(format!("transaction {transaction_id}")))?;

    let split_sum: i64 = splits.iter().map(|s| s.amount).sum();
    if split_sum != parent_amount {
        return Err(AppError::Validation(format!(
            "Split sum ({split_sum}) does not equal parent amount ({parent_amount})"
        )));
    }
    Ok(())
}

fn insert_split_rows(
    conn: &rusqlite::Connection,
    transaction_id: &str,
    splits: &[CreateSplitInput],
    now: &str,
) -> AppResult<()> {
    for split in splits {
        let id = Uuid::new_v4().to_string();
        if let Err(e) = conn.execute(
            "INSERT INTO transaction_splits (id, transaction_id, category_id, amount, description, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, transaction_id, split.category_id, split.amount, split.description, now],
        ) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(AppError::Db(e));
        }
    }
    Ok(())
}

fn fetch_splits(
    conn: &rusqlite::Connection,
    transaction_id: &str,
) -> AppResult<Vec<TransactionSplit>> {
    let sql = format!(
        "SELECT {SPLIT_COLS} FROM transaction_splits ts
         LEFT JOIN categories c ON ts.category_id = c.id
         WHERE ts.transaction_id = ?1
         ORDER BY ts.created_at ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let result = stmt
        .query_map([transaction_id], row_to_split)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(result)
}

#[tauri::command]
pub fn create_splits(
    transaction_id: String,
    splits: Vec<CreateSplitInput>,
    state: State<DbState>,
) -> AppResult<Vec<TransactionSplit>> {
    if splits.is_empty() {
        return Err(AppError::Validation("At least one split is required".to_string()));
    }

    let conn = state.write_conn.lock().unwrap();
    validate_split_sum(&conn, &transaction_id, &splits)?;

    let now = now_utc();
    conn.execute("BEGIN", [])?;

    insert_split_rows(&conn, &transaction_id, &splits, &now)?;

    // Mark parent as split
    if let Err(e) = conn.execute(
        "UPDATE transactions SET is_split = 1, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, transaction_id],
    ) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(AppError::Db(e));
    }

    conn.execute("COMMIT", [])?;
    fetch_splits(&conn, &transaction_id)
}

#[tauri::command]
pub fn get_splits(
    transaction_id: String,
    state: State<DbState>,
) -> AppResult<Vec<TransactionSplit>> {
    let conn = state.read_conn.lock().unwrap();
    fetch_splits(&conn, &transaction_id)
}

#[tauri::command]
pub fn update_splits(
    transaction_id: String,
    splits: Vec<CreateSplitInput>,
    state: State<DbState>,
) -> AppResult<Vec<TransactionSplit>> {
    if splits.is_empty() {
        return Err(AppError::Validation("At least one split is required".to_string()));
    }

    let conn = state.write_conn.lock().unwrap();
    validate_split_sum(&conn, &transaction_id, &splits)?;

    let now = now_utc();
    conn.execute("BEGIN", [])?;

    // Delete existing splits
    if let Err(e) = conn.execute(
        "DELETE FROM transaction_splits WHERE transaction_id = ?1",
        [&transaction_id],
    ) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(AppError::Db(e));
    }

    insert_split_rows(&conn, &transaction_id, &splits, &now)?;

    // Update parent timestamp
    if let Err(e) = conn.execute(
        "UPDATE transactions SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, transaction_id],
    ) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(AppError::Db(e));
    }

    conn.execute("COMMIT", [])?;
    fetch_splits(&conn, &transaction_id)
}

#[tauri::command]
pub fn delete_splits(
    transaction_id: String,
    state: State<DbState>,
) -> AppResult<()> {
    let conn = state.write_conn.lock().unwrap();
    let now = now_utc();

    conn.execute("BEGIN", [])?;

    if let Err(e) = conn.execute(
        "DELETE FROM transaction_splits WHERE transaction_id = ?1",
        [&transaction_id],
    ) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(AppError::Db(e));
    }

    // Un-mark parent as split
    if let Err(e) = conn.execute(
        "UPDATE transactions SET is_split = 0, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, transaction_id],
    ) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(AppError::Db(e));
    }

    conn.execute("COMMIT", [])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::db::migrations::{run_migrations, seed_test_data};

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();
        seed_test_data(&conn).unwrap();
        conn
    }

    fn get_account_id(conn: &rusqlite::Connection) -> String {
        conn.query_row("SELECT id FROM accounts LIMIT 1", [], |r| r.get(0)).unwrap()
    }

    fn create_test_transaction(conn: &rusqlite::Connection, amount: i64) -> String {
        let id = Uuid::new_v4().to_string();
        let now = now_utc();
        let account_id = get_account_id(conn);
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, payee, description, is_cleared, created_at, updated_at)
             VALUES (?1, ?2, '2026-01-15', ?3, 'expense', 'Test', '', 0, ?4, ?4)",
            rusqlite::params![id, account_id, amount, now],
        ).unwrap();
        id
    }

    fn create_test_category(conn: &rusqlite::Connection, name: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO categories (id, name, color, icon, type, sort_order, created_at, updated_at)
             VALUES (?1, ?2, '#ff0000', 'folder', 'expense', 0, ?3, ?3)",
            rusqlite::params![id, name, now],
        ).unwrap();
        id
    }

    #[test]
    fn test_create_splits_valid_sum() {
        let conn = setup();
        let txn_id = create_test_transaction(&conn, 10000); // $100.00
        let cat1 = create_test_category(&conn, "SplitFood");
        let cat2 = create_test_category(&conn, "SplitTransport");

        let splits = vec![
            CreateSplitInput { category_id: cat1.clone(), amount: 7000, description: "Groceries".to_string() },
            CreateSplitInput { category_id: cat2.clone(), amount: 3000, description: "Bus".to_string() },
        ];

        // Insert splits manually (no Tauri state in tests)
        let now = now_utc();
        conn.execute("BEGIN", []).unwrap();
        for split in &splits {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO transaction_splits (id, transaction_id, category_id, amount, description, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![id, txn_id, split.category_id, split.amount, split.description, now],
            ).unwrap();
        }
        conn.execute("UPDATE transactions SET is_split = 1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, txn_id]).unwrap();
        conn.execute("COMMIT", []).unwrap();

        // Verify splits exist
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM transaction_splits WHERE transaction_id = ?1"
        ).unwrap();
        let count: i64 = stmt.query_row([&txn_id], |r| r.get(0)).unwrap();
        assert_eq!(count, 2);

        // Verify parent is marked as split
        let is_split: i64 = conn.query_row(
            "SELECT is_split FROM transactions WHERE id = ?1",
            [&txn_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(is_split, 1);
    }

    #[test]
    fn test_split_sum_validation() {
        let conn = setup();
        let txn_id = create_test_transaction(&conn, 10000);
        let _cat1 = create_test_category(&conn, "SplitValFood");

        // Sum doesn't match parent
        let split_sum: i64 = 5000;
        let parent_amount: i64 = conn.query_row(
            "SELECT amount FROM transactions WHERE id = ?1",
            [&txn_id], |r| r.get(0),
        ).unwrap();
        assert_ne!(split_sum, parent_amount, "Sum should not equal parent for this test");
    }

    #[test]
    fn test_delete_splits_restores_parent() {
        let conn = setup();
        let txn_id = create_test_transaction(&conn, 10000);
        let cat1 = create_test_category(&conn, "SplitDelFood");
        let now = now_utc();

        // Create a split
        let split_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO transaction_splits (id, transaction_id, category_id, amount, description, created_at)
             VALUES (?1, ?2, ?3, 10000, 'All food', ?4)",
            rusqlite::params![split_id, txn_id, cat1, now],
        ).unwrap();
        conn.execute("UPDATE transactions SET is_split = 1 WHERE id = ?1", [&txn_id]).unwrap();

        // Delete splits
        conn.execute("DELETE FROM transaction_splits WHERE transaction_id = ?1", [&txn_id]).unwrap();
        conn.execute("UPDATE transactions SET is_split = 0, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![now, txn_id]).unwrap();

        // Verify parent is no longer split
        let is_split: i64 = conn.query_row(
            "SELECT is_split FROM transactions WHERE id = ?1",
            [&txn_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(is_split, 0);

        // Verify no splits remain
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transaction_splits WHERE transaction_id = ?1",
            [&txn_id], |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 0);
    }
}
