#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use crate::db::migrations::run_migrations;
    use crate::helpers::timestamps::now_utc;
    use uuid::Uuid;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    // ── Account tests ──────────────────────────────────────────────

    #[test]
    fn account_create_and_read() {
        let conn = setup();
        let id = Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO accounts (id, name, type, currency_code, opening_balance, sort_order, is_archived, created_at, updated_at)
             VALUES (?1, 'Checking', 'checking', 'USD', 0, 0, 0, ?2, ?3)",
            rusqlite::params![id, now, now],
        ).unwrap();

        let name: String = conn.query_row(
            "SELECT name FROM accounts WHERE id = ?1", [&id], |r| r.get(0)
        ).unwrap();
        assert_eq!(name, "Checking");
    }

    #[test]
    fn account_update() {
        let conn = setup();
        let id = Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO accounts (id, name, type, currency_code, opening_balance, sort_order, is_archived, created_at, updated_at)
             VALUES (?1, 'Old Name', 'checking', 'USD', 0, 0, 0, ?2, ?3)",
            rusqlite::params![id, now, now],
        ).unwrap();

        conn.execute(
            "UPDATE accounts SET name='New Name', updated_at=?1 WHERE id=?2",
            rusqlite::params![now, id],
        ).unwrap();

        let name: String = conn.query_row(
            "SELECT name FROM accounts WHERE id = ?1", [&id], |r| r.get(0)
        ).unwrap();
        assert_eq!(name, "New Name");
    }

    #[test]
    fn account_delete() {
        let conn = setup();
        let id = Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO accounts (id, name, type, currency_code, opening_balance, sort_order, is_archived, created_at, updated_at)
             VALUES (?1, 'ToDelete', 'checking', 'USD', 0, 0, 0, ?2, ?3)",
            rusqlite::params![id, now, now],
        ).unwrap();

        conn.execute("DELETE FROM accounts WHERE id = ?1", [&id]).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM accounts WHERE id = ?1", [&id], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn account_delete_blocked_with_transactions() {
        let conn = setup();
        let now = now_utc();

        // Get the seeded account id
        let acct_id: String = conn.query_row(
            "SELECT id FROM accounts LIMIT 1", [], |r| r.get(0)
        ).unwrap();

        // Insert a transaction referencing it
        let txn_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, created_at, updated_at)
             VALUES (?1, ?2, '2026-01-01', 100, 'expense', ?3, ?4)",
            rusqlite::params![txn_id, acct_id, now, now],
        ).unwrap();

        // Delete should fail due to RESTRICT
        let result = conn.execute("DELETE FROM accounts WHERE id = ?1", [&acct_id]);
        assert!(result.is_err(), "should be blocked by FK RESTRICT");
    }

    // ── Category tests ─────────────────────────────────────────────

    #[test]
    fn category_create_and_read() {
        let conn = setup();
        let id = Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO categories (id, name, color, icon, type, sort_order, is_archived, created_at, updated_at)
             VALUES (?1, 'Dining', '#f97316', 'fork', 'expense', 0, 0, ?2, ?3)",
            rusqlite::params![id, now, now],
        ).unwrap();

        let name: String = conn.query_row(
            "SELECT name FROM categories WHERE id = ?1", [&id], |r| r.get(0)
        ).unwrap();
        assert_eq!(name, "Dining");
    }

    #[test]
    fn category_update() {
        let conn = setup();
        let id = Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO categories (id, name, color, icon, type, sort_order, is_archived, created_at, updated_at)
             VALUES (?1, 'Old', '#ffffff', 'folder', 'expense', 0, 0, ?2, ?3)",
            rusqlite::params![id, now, now],
        ).unwrap();

        conn.execute(
            "UPDATE categories SET name='New', color='#000000', updated_at=?1 WHERE id=?2",
            rusqlite::params![now, id],
        ).unwrap();

        let (name, color): (String, String) = conn.query_row(
            "SELECT name, color FROM categories WHERE id = ?1", [&id],
            |r| Ok((r.get(0)?, r.get(1)?))
        ).unwrap();
        assert_eq!(name, "New");
        assert_eq!(color, "#000000");
    }

    #[test]
    fn category_delete() {
        let conn = setup();
        let id = Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO categories (id, name, color, icon, type, sort_order, is_archived, created_at, updated_at)
             VALUES (?1, 'ToDelete', '#ffffff', 'folder', 'expense', 0, 0, ?2, ?3)",
            rusqlite::params![id, now, now],
        ).unwrap();

        conn.execute("DELETE FROM categories WHERE id = ?1", [&id]).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM categories WHERE id = ?1", [&id], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn category_delete_sets_transactions_null() {
        let conn = setup();
        let now = now_utc();

        // Get the seeded account
        let acct_id: String = conn.query_row(
            "SELECT id FROM accounts LIMIT 1", [], |r| r.get(0)
        ).unwrap();

        // Create a custom category
        let cat_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO categories (id, name, color, icon, type, sort_order, is_archived, created_at, updated_at)
             VALUES (?1, 'TempCat', '#ffffff', 'folder', 'expense', 99, 0, ?2, ?3)",
            rusqlite::params![cat_id, now, now],
        ).unwrap();

        // Create a transaction referencing it
        let txn_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, category_id, created_at, updated_at)
             VALUES (?1, ?2, '2026-01-01', 500, 'expense', ?3, ?4, ?5)",
            rusqlite::params![txn_id, acct_id, cat_id, now, now],
        ).unwrap();

        // Delete category
        conn.execute("DELETE FROM categories WHERE id = ?1", [&cat_id]).unwrap();

        // Transaction should still exist with category_id = NULL
        let category_id: Option<String> = conn.query_row(
            "SELECT category_id FROM transactions WHERE id = ?1", [&txn_id],
            |r| r.get(0)
        ).unwrap();
        assert!(category_id.is_none(), "category_id must be NULL after category delete");
    }
}
