#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use crate::db::migrations::{run_migrations, seed_test_data};

    fn in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory db");
        run_migrations(&conn).expect("migrations");
        seed_test_data(&conn).expect("seed test data");
        conn
    }

    #[test]
    fn all_tables_exist() {
        let conn = in_memory_db();
        let tables: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
                .unwrap();
            stmt.query_map([], |row| row.get(0))
                .unwrap()
                .map(|r| r.unwrap())
                .collect()
        };
        assert!(tables.contains(&"accounts".to_string()));
        assert!(tables.contains(&"categories".to_string()));
        assert!(tables.contains(&"transactions".to_string()));
        assert!(tables.contains(&"import_batches".to_string()));
        assert!(tables.contains(&"import_profiles".to_string()));
        assert!(tables.contains(&"settings".to_string()));
    }

    #[test]
    fn seed_categories_present() {
        let conn = in_memory_db();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
            .unwrap();
        // 4 income + 11 expense = 15
        assert_eq!(count, 15);
    }

    #[test]
    fn seed_income_categories() {
        let conn = in_memory_db();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM categories WHERE type='income'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 4);
    }

    #[test]
    fn seed_expense_categories() {
        let conn = in_memory_db();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM categories WHERE type='expense'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 11);
    }

    #[test]
    fn seed_default_account() {
        let conn = in_memory_db();
        let name: String = conn
            .query_row(
                "SELECT name FROM accounts WHERE type='checking' LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(name, "Primary Checking");
    }

    #[test]
    fn migrations_idempotent() {
        let conn = in_memory_db();
        // Running migrations a second time should not error or duplicate seed data
        run_migrations(&conn).expect("second migration run");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 15, "seed data must not be duplicated on re-run");
    }

    #[test]
    fn timestamps_are_rfc3339() {
        let conn = in_memory_db();
        let ts: String = conn
            .query_row(
                "SELECT created_at FROM accounts LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        // RFC3339 contains 'T' separator and ends with 'Z' or offset
        assert!(ts.contains('T'), "created_at must be RFC3339: {ts}");
    }

    #[test]
    fn foreign_keys_enforced() {
        let conn = in_memory_db();
        // Inserting a transaction with a non-existent account_id must fail
        let result = conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, created_at, updated_at)
             VALUES ('t1', 'nonexistent-account', '2026-01-01', 100, 'expense', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        );
        assert!(result.is_err(), "foreign key violation must be rejected");
    }
}
