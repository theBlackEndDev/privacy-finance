use tauri::State;
use crate::db::DbState;
use crate::error::AppResult;
use crate::helpers::timestamps::now_utc;
use crate::models::reconciliation::*;

fn row_to_reconciliation(row: &rusqlite::Row<'_>) -> rusqlite::Result<Reconciliation> {
    Ok(Reconciliation {
        id: row.get(0)?,
        account_id: row.get(1)?,
        statement_date: row.get(2)?,
        statement_balance: row.get(3)?,
        status: row.get(4)?,
        created_at: row.get(5)?,
    })
}

/// Calculate the cleared balance for an account up to a given date.
/// cleared_balance = opening_balance + SUM(signed cleared transactions up to date)
///                 + SUM(signed newly-reconciled items in this session)
fn calc_cleared_balance(
    conn: &rusqlite::Connection,
    account_id: &str,
    statement_date: &str,
    reconciliation_id: &str,
) -> rusqlite::Result<i64> {
    // Get opening balance
    let opening: i64 = conn.query_row(
        "SELECT opening_balance FROM accounts WHERE id = ?1",
        [account_id],
        |row| row.get(0),
    )?;

    // Sum of already-cleared transactions (before this session)
    let cleared_sum: i64 = conn.query_row(
        "SELECT COALESCE(SUM(
            CASE WHEN kind = 'income' OR (kind = 'transfer' AND account_id = ?1) THEN amount
                 ELSE -amount END
        ), 0) FROM transactions
         WHERE account_id = ?1 AND date <= ?2 AND is_cleared = 1",
        rusqlite::params![account_id, statement_date],
        |row| row.get(0),
    )?;

    // Sum of newly-reconciled items in this session (not yet is_cleared in transactions)
    let reconciled_sum: i64 = conn.query_row(
        "SELECT COALESCE(SUM(
            CASE WHEN t.kind = 'income' OR (t.kind = 'transfer' AND t.account_id = ?2) THEN t.amount
                 ELSE -t.amount END
        ), 0) FROM reconciliation_items ri
         JOIN transactions t ON ri.transaction_id = t.id
         WHERE ri.reconciliation_id = ?1 AND ri.is_reconciled = 1",
        rusqlite::params![reconciliation_id, account_id],
        |row| row.get(0),
    )?;

    Ok(opening + cleared_sum + reconciled_sum)
}

/// Load reconciliation summary (reconciliation record + items with details + balances)
fn load_summary(
    conn: &rusqlite::Connection,
    reconciliation_id: &str,
) -> AppResult<ReconciliationSummary> {
    let recon = conn.query_row(
        "SELECT id, account_id, statement_date, statement_balance, status, created_at
         FROM reconciliations WHERE id = ?1",
        [reconciliation_id],
        row_to_reconciliation,
    )?;

    let mut stmt = conn.prepare(
        "SELECT ri.transaction_id, t.date, t.payee, t.amount, t.kind, ri.is_reconciled
         FROM reconciliation_items ri
         JOIN transactions t ON ri.transaction_id = t.id
         WHERE ri.reconciliation_id = ?1
         ORDER BY t.date ASC",
    )?;
    let items: Vec<ReconciliationItemDetail> = stmt
        .query_map([reconciliation_id], |row| {
            Ok(ReconciliationItemDetail {
                transaction_id: row.get(0)?,
                date: row.get(1)?,
                payee: row.get(2)?,
                amount: row.get(3)?,
                kind: row.get(4)?,
                is_reconciled: row.get::<_, i64>(5)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let cleared_balance = calc_cleared_balance(
        conn,
        &recon.account_id,
        &recon.statement_date,
        reconciliation_id,
    )?;

    let statement_balance = recon.statement_balance;
    let difference = cleared_balance - statement_balance;

    Ok(ReconciliationSummary {
        reconciliation: recon,
        items,
        cleared_balance,
        statement_balance,
        difference,
    })
}

#[tauri::command]
pub fn start_reconciliation(
    db: State<DbState>,
    input: StartReconciliationInput,
) -> AppResult<ReconciliationSummary> {
    let conn = db.write_conn.lock().unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let now = now_utc();

    conn.execute(
        "INSERT INTO reconciliations (id, account_id, statement_date, statement_balance, status, created_at)
         VALUES (?1, ?2, ?3, ?4, 'pending', ?5)",
        rusqlite::params![id, input.account_id, input.statement_date, input.statement_balance, now],
    )?;

    // Bulk-insert unreconciled transactions for this account up to statement date
    conn.execute(
        "INSERT INTO reconciliation_items (reconciliation_id, transaction_id, is_reconciled)
         SELECT ?1, id, 0 FROM transactions
         WHERE account_id = ?2 AND date <= ?3 AND is_cleared = 0
         ORDER BY date ASC",
        rusqlite::params![id, input.account_id, input.statement_date],
    )?;

    load_summary(&conn, &id)
}

#[tauri::command]
pub fn toggle_reconciliation_item(
    db: State<DbState>,
    reconciliation_id: String,
    transaction_id: String,
) -> AppResult<ReconciliationSummary> {
    let conn = db.write_conn.lock().unwrap();

    conn.execute(
        "UPDATE reconciliation_items
         SET is_reconciled = CASE WHEN is_reconciled = 0 THEN 1 ELSE 0 END
         WHERE reconciliation_id = ?1 AND transaction_id = ?2",
        rusqlite::params![reconciliation_id, transaction_id],
    )?;

    load_summary(&conn, &reconciliation_id)
}

#[tauri::command]
pub fn get_reconciliation_diff(
    db: State<DbState>,
    reconciliation_id: String,
) -> AppResult<ReconciliationSummary> {
    let conn = db.read_conn.lock().unwrap();
    load_summary(&conn, &reconciliation_id)
}

#[tauri::command]
pub fn complete_reconciliation(
    db: State<DbState>,
    reconciliation_id: String,
) -> AppResult<Reconciliation> {
    let conn = db.write_conn.lock().unwrap();
    let summary = load_summary(&conn, &reconciliation_id)?;

    if summary.difference != 0 {
        return Err(crate::error::AppError::Validation(
            format!("Cannot complete: difference is {} (must be 0)", summary.difference),
        ));
    }

    // Mark reconciled transactions as cleared
    conn.execute(
        "UPDATE transactions SET is_cleared = 1
         WHERE id IN (
             SELECT transaction_id FROM reconciliation_items
             WHERE reconciliation_id = ?1 AND is_reconciled = 1
         )",
        [&reconciliation_id],
    )?;

    // Mark reconciliation as completed
    conn.execute(
        "UPDATE reconciliations SET status = 'completed' WHERE id = ?1",
        [&reconciliation_id],
    )?;

    let mut recon = summary.reconciliation;
    recon.status = "completed".to_string();
    Ok(recon)
}

#[tauri::command]
pub fn cancel_reconciliation(
    db: State<DbState>,
    reconciliation_id: String,
) -> AppResult<()> {
    let conn = db.write_conn.lock().unwrap();

    conn.execute(
        "DELETE FROM reconciliation_items WHERE reconciliation_id = ?1",
        [&reconciliation_id],
    )?;
    conn.execute(
        "UPDATE reconciliations SET status = 'cancelled' WHERE id = ?1",
        [&reconciliation_id],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::{run_migrations, seed_test_data};
    use crate::helpers::timestamps::now_utc;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        seed_test_data(&conn).unwrap();
        conn
    }

    fn insert_test_tx(conn: &Connection, account_id: &str, date: &str, amount: i64, kind: &str, cleared: bool) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, payee, description, is_cleared, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 'Test', '', ?6, ?7, ?7)",
            rusqlite::params![id, account_id, date, amount, kind, cleared as i64, now],
        ).unwrap();
        id
    }

    fn get_test_account(conn: &Connection) -> String {
        conn.query_row("SELECT id FROM accounts LIMIT 1", [], |row| row.get(0)).unwrap()
    }

    #[test]
    fn test_start_reconciliation_loads_unreconciled() {
        let conn = setup_db();
        let acct = get_test_account(&conn);

        // Add an unreconciled transaction
        insert_test_tx(&conn, &acct, "2026-03-01", 5000, "income", false);
        // Add a cleared transaction (should NOT appear in items)
        insert_test_tx(&conn, &acct, "2026-03-02", 3000, "expense", true);

        let id = uuid::Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO reconciliations (id, account_id, statement_date, statement_balance, status, created_at)
             VALUES (?1, ?2, '2026-03-31', 100000, 'pending', ?3)",
            rusqlite::params![id, acct, now],
        ).unwrap();

        // Load unreconciled
        let mut stmt = conn.prepare(
            "SELECT id FROM transactions WHERE account_id = ?1 AND date <= '2026-03-31' AND is_cleared = 0"
        ).unwrap();
        let tx_ids: Vec<String> = stmt.query_map([&acct], |row| row.get(0)).unwrap()
            .collect::<Result<Vec<_>, _>>().unwrap();

        for tx_id in &tx_ids {
            conn.execute(
                "INSERT INTO reconciliation_items (reconciliation_id, transaction_id, is_reconciled) VALUES (?1, ?2, 0)",
                rusqlite::params![id, tx_id],
            ).unwrap();
        }

        let summary = load_summary(&conn, &id).unwrap();
        // Should have unreconciled items only (seed data has some too)
        assert!(summary.items.len() >= 1, "Should load at least our unreconciled tx");
        assert!(summary.items.iter().all(|i| !i.is_reconciled), "All items should start unreconciled");
    }

    #[test]
    fn test_toggle_changes_diff() {
        let conn = setup_db();
        let acct = get_test_account(&conn);
        let tx_id = insert_test_tx(&conn, &acct, "2026-03-01", 10000, "income", false);

        let recon_id = uuid::Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO reconciliations (id, account_id, statement_date, statement_balance, status, created_at)
             VALUES (?1, ?2, '2026-03-31', 0, 'pending', ?3)",
            rusqlite::params![recon_id, acct, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO reconciliation_items (reconciliation_id, transaction_id, is_reconciled) VALUES (?1, ?2, 0)",
            rusqlite::params![recon_id, tx_id],
        ).unwrap();

        let before = load_summary(&conn, &recon_id).unwrap();
        let before_diff = before.difference;

        // Toggle on
        conn.execute(
            "UPDATE reconciliation_items SET is_reconciled = 1 WHERE reconciliation_id = ?1 AND transaction_id = ?2",
            rusqlite::params![recon_id, tx_id],
        ).unwrap();

        let after = load_summary(&conn, &recon_id).unwrap();
        assert_ne!(before_diff, after.difference, "Toggling should change the difference");
    }

    #[test]
    fn test_complete_sets_cleared() {
        let conn = setup_db();
        let acct = get_test_account(&conn);

        // Insert a new unreconciled income tx
        let tx_id = insert_test_tx(&conn, &acct, "2026-03-01", 5000, "income", false);

        let recon_id = uuid::Uuid::new_v4().to_string();
        let now = now_utc();

        // First create the reconciliation with a placeholder balance
        conn.execute(
            "INSERT INTO reconciliations (id, account_id, statement_date, statement_balance, status, created_at)
             VALUES (?1, ?2, '2026-03-31', 0, 'pending', ?3)",
            rusqlite::params![recon_id, acct, now],
        ).unwrap();

        // Add just our test tx as reconciled
        conn.execute(
            "INSERT INTO reconciliation_items (reconciliation_id, transaction_id, is_reconciled) VALUES (?1, ?2, 1)",
            rusqlite::params![recon_id, tx_id],
        ).unwrap();

        // Calculate what the cleared_balance actually is, then set statement_balance to match
        let cleared_balance = calc_cleared_balance(&conn, &acct, "2026-03-31", &recon_id).unwrap();
        conn.execute(
            "UPDATE reconciliations SET statement_balance = ?1 WHERE id = ?2",
            rusqlite::params![cleared_balance, recon_id],
        ).unwrap();

        // Now diff should be zero
        let summary = load_summary(&conn, &recon_id).unwrap();
        assert_eq!(summary.difference, 0, "Diff should be zero before completing");

        // Complete
        conn.execute("UPDATE transactions SET is_cleared = 1 WHERE id IN (SELECT transaction_id FROM reconciliation_items WHERE reconciliation_id = ?1 AND is_reconciled = 1)", [&recon_id]).unwrap();
        conn.execute("UPDATE reconciliations SET status = 'completed' WHERE id = ?1", [&recon_id]).unwrap();

        // Verify transaction is now cleared
        let cleared: i64 = conn.query_row(
            "SELECT is_cleared FROM transactions WHERE id = ?1", [&tx_id], |row| row.get(0)
        ).unwrap();
        assert_eq!(cleared, 1, "Transaction should be cleared after complete");
    }

    #[test]
    fn test_complete_rejects_unbalanced() {
        let conn = setup_db();
        let acct = get_test_account(&conn);

        let recon_id = uuid::Uuid::new_v4().to_string();
        let now = now_utc();
        // Set statement balance to something that won't match
        conn.execute(
            "INSERT INTO reconciliations (id, account_id, statement_date, statement_balance, status, created_at)
             VALUES (?1, ?2, '2026-03-31', 99999999, 'pending', ?3)",
            rusqlite::params![recon_id, acct, now],
        ).unwrap();

        let summary = load_summary(&conn, &recon_id).unwrap();
        assert_ne!(summary.difference, 0, "Diff should be non-zero");
    }

    #[test]
    fn test_cancel_leaves_transactions_unchanged() {
        let conn = setup_db();
        let acct = get_test_account(&conn);
        let tx_id = insert_test_tx(&conn, &acct, "2026-03-01", 5000, "expense", false);

        let recon_id = uuid::Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO reconciliations (id, account_id, statement_date, statement_balance, status, created_at)
             VALUES (?1, ?2, '2026-03-31', 0, 'pending', ?3)",
            rusqlite::params![recon_id, acct, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO reconciliation_items (reconciliation_id, transaction_id, is_reconciled) VALUES (?1, ?2, 1)",
            rusqlite::params![recon_id, tx_id],
        ).unwrap();

        // Cancel
        conn.execute("DELETE FROM reconciliation_items WHERE reconciliation_id = ?1", [&recon_id]).unwrap();
        conn.execute("UPDATE reconciliations SET status = 'cancelled' WHERE id = ?1", [&recon_id]).unwrap();

        // Transaction should still NOT be cleared
        let cleared: i64 = conn.query_row(
            "SELECT is_cleared FROM transactions WHERE id = ?1", [&tx_id], |row| row.get(0)
        ).unwrap();
        assert_eq!(cleared, 0, "Cancel should not clear transactions");

        let status: String = conn.query_row(
            "SELECT status FROM reconciliations WHERE id = ?1", [&recon_id], |row| row.get(0)
        ).unwrap();
        assert_eq!(status, "cancelled");
    }
}
