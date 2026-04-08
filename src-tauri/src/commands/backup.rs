use serde::Serialize;
use tauri::State;
use crate::db::DbState;
use crate::error::{AppError, AppResult};

// ── BackupInfo ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub file_size_bytes: u64,
    pub account_count: i64,
    pub category_count: i64,
    pub transaction_count: i64,
    pub recurring_count: i64,
    pub date_range_from: Option<String>,
    pub date_range_to: Option<String>,
}

/// Validate that a path does not contain directory traversal sequences.
fn validate_path(path: &str) -> AppResult<()> {
    if path.contains("..") {
        return Err(AppError::Validation("Path must not contain '..' traversal".into()));
    }
    Ok(())
}

// ── create_backup ───────────────────────────────────────────────────

#[tauri::command]
pub fn create_backup(
    path: String,
    state: State<DbState>,
) -> AppResult<()> {
    validate_path(&path)?;
    let conn = state.read_conn.lock().unwrap();

    // Verify source integrity before backup
    let integrity: String = conn.query_row(
        "PRAGMA integrity_check", [], |row| row.get(0),
    )?;
    if integrity != "ok" {
        return Err(AppError::Validation(
            format!("Source database integrity check failed: {}", integrity),
        ));
    }

    // Use SQLite backup API for safe, consistent copy
    let mut dst = rusqlite::Connection::open(&path)?;
    let backup = rusqlite::backup::Backup::new(&conn, &mut dst)?;
    backup.run_to_completion(100, std::time::Duration::from_millis(10), None)?;

    Ok(())
}

// ── restore_backup ──────────────────────────────────────────────────

#[tauri::command]
pub fn restore_backup(
    path: String,
    state: State<DbState>,
) -> AppResult<()> {
    validate_path(&path)?;
    // First validate the backup file
    let src = rusqlite::Connection::open(&path)?;
    let integrity: String = src.query_row(
        "PRAGMA integrity_check", [], |row| row.get(0),
    )?;
    if integrity != "ok" {
        return Err(AppError::Validation(
            format!("Backup integrity check failed: {}", integrity),
        ));
    }

    // Lock the write connection and restore via SQLite backup API
    let mut write_conn = state.write_conn.lock().unwrap();

    // Use backup API to copy from backup source into the live write connection
    let backup = rusqlite::backup::Backup::new(&src, &mut *write_conn)?;
    backup.run_to_completion(100, std::time::Duration::from_millis(10), None)?;

    Ok(())
}

// ── get_backup_info ─────────────────────────────────────────────────

#[tauri::command]
pub fn get_backup_info(path: String) -> AppResult<BackupInfo> {
    validate_path(&path)?;
    let metadata = std::fs::metadata(&path)?;
    let file_size_bytes = metadata.len();

    // Open read-write so integrity_check can validate FTS5 indexes
    let conn = rusqlite::Connection::open(&path)?;

    // Validate it's a real SQLite database
    let integrity: String = conn.query_row(
        "PRAGMA integrity_check", [], |row| row.get(0),
    )?;
    if integrity != "ok" {
        return Err(AppError::Validation(
            format!("Backup file integrity check failed: {}", integrity),
        ));
    }

    let account_count = count_table(&conn, "accounts")?;
    let category_count = count_table(&conn, "categories")?;
    let transaction_count = count_table(&conn, "transactions")?;
    let recurring_count = count_table_optional(&conn, "recurring_transactions")?;

    let (date_range_from, date_range_to): (Option<String>, Option<String>) = conn.query_row(
        "SELECT MIN(date), MAX(date) FROM transactions",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).unwrap_or((None, None));

    Ok(BackupInfo {
        file_size_bytes,
        account_count,
        category_count,
        transaction_count,
        recurring_count,
        date_range_from,
        date_range_to,
    })
}

// ── Helpers ─────────────────────────────────────────────────────────

fn count_table(conn: &rusqlite::Connection, table: &str) -> AppResult<i64> {
    let sql = format!("SELECT COUNT(*) FROM {}", table);
    let count: i64 = conn.query_row(&sql, [], |row| row.get(0))?;
    Ok(count)
}

fn count_table_optional(conn: &rusqlite::Connection, table: &str) -> AppResult<i64> {
    let sql = format!("SELECT COUNT(*) FROM {}", table);
    match conn.query_row(&sql, [], |row: &rusqlite::Row| row.get::<_, i64>(0)) {
        Ok(count) => Ok(count),
        Err(_) => Ok(0), // table doesn't exist in older backups
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::db::migrations::{run_migrations, seed_test_data};

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        run_migrations(&conn).unwrap();
        seed_test_data(&conn).unwrap();
        conn
    }

    #[test]
    fn test_backup_creates_valid_copy() {
        let conn = setup_db();
        let dir = tempfile::tempdir().unwrap();
        let backup_path = dir.path().join("backup.db");

        // Use backup API directly (mirroring create_backup logic)
        let mut dst = Connection::open(&backup_path).unwrap();
        let backup = rusqlite::backup::Backup::new(&conn, &mut dst).unwrap();
        backup.run_to_completion(100, std::time::Duration::from_millis(10), None).unwrap();
        drop(backup);
        drop(dst);

        // Verify the backup is valid
        let backup_conn = Connection::open(&backup_path).unwrap();
        let integrity: String = backup_conn.query_row(
            "PRAGMA integrity_check", [], |row| row.get(0),
        ).unwrap();
        assert_eq!(integrity, "ok");

        // Verify data was copied
        let count: i64 = backup_conn.query_row(
            "SELECT COUNT(*) FROM accounts", [], |row| row.get(0),
        ).unwrap();
        assert!(count > 0, "Backup should contain accounts");
    }

    #[test]
    fn test_backup_info() {
        let conn = setup_db();
        let dir = tempfile::tempdir().unwrap();
        let backup_path = dir.path().join("info_test.db");

        let mut dst = Connection::open(&backup_path).unwrap();
        let backup = rusqlite::backup::Backup::new(&conn, &mut dst).unwrap();
        backup.run_to_completion(100, std::time::Duration::from_millis(10), None).unwrap();
        drop(backup);
        drop(dst);

        let info = get_backup_info(backup_path.to_str().unwrap().to_string()).unwrap();
        assert!(info.account_count > 0);
        assert!(info.category_count > 0);
        assert!(info.file_size_bytes > 0);
    }

    #[test]
    fn test_restore_backup() {
        let mut conn = setup_db();

        // Create a backup
        let dir = tempfile::tempdir().unwrap();
        let backup_path = dir.path().join("restore_test.db");
        let mut dst = Connection::open(&backup_path).unwrap();
        let backup = rusqlite::backup::Backup::new(&conn, &mut dst).unwrap();
        backup.run_to_completion(100, std::time::Duration::from_millis(10), None).unwrap();
        drop(backup);
        drop(dst);

        // Modify the original DB
        let now = "2026-01-01T00:00:00Z";
        let account_id: String = conn.query_row(
            "SELECT id FROM accounts LIMIT 1", [], |row| row.get(0),
        ).unwrap();
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, payee, description, is_cleared, created_at, updated_at)
             VALUES ('extra-tx', ?1, '2026-06-01', 9999, 'expense', 'Extra', 'Should be gone after restore', 0, ?2, ?2)",
            rusqlite::params![account_id, now],
        ).unwrap();
        let count_before: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions", [], |row| row.get(0),
        ).unwrap();

        // Restore from backup (need &mut for destination)
        {
            let src = Connection::open(&backup_path).unwrap();
            let restore = rusqlite::backup::Backup::new(&src, &mut conn).unwrap();
            restore.run_to_completion(100, std::time::Duration::from_millis(10), None).unwrap();
        }

        let count_after: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions", [], |row| row.get(0),
        ).unwrap();
        assert!(count_after < count_before, "Restore should remove extra transaction");

        // Verify the extra transaction is gone
        let extra_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE id = 'extra-tx'", [], |row| row.get(0),
        ).unwrap();
        assert_eq!(extra_count, 0, "Extra transaction should be gone after restore");
    }

    #[test]
    fn test_integrity_check_on_corrupt_file() {
        let dir = tempfile::tempdir().unwrap();
        let corrupt_path = dir.path().join("corrupt.db");
        std::fs::write(&corrupt_path, b"not a database").unwrap();

        let result = get_backup_info(corrupt_path.to_str().unwrap().to_string());
        assert!(result.is_err(), "Should fail on corrupt file");
    }
}
