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

    #[test]
    fn fix_transfer_signs_repairs_old_rows() {
        let conn = in_memory_db();
        conn.execute(
            "INSERT INTO accounts (id, name, type, created_at, updated_at)
             VALUES ('extra-acct', 'Extra', 'checking', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        let ids: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT id FROM accounts ORDER BY sort_order, id LIMIT 4")
                .unwrap();
            stmt.query_map([], |r| r.get(0)).unwrap().map(|r| r.unwrap()).collect()
        };
        let (a, b, c, d) = (&ids[0], &ids[1], &ids[2], &ids[3]);
        let now = "2026-01-01T00:00:00Z";

        let insert = |id: &str, acct: &str, amount: i64, payee: &str, group: &str| {
            conn.execute(
                "INSERT INTO transactions (id, account_id, date, amount, kind, payee, description, is_cleared, transfer_group_id, created_at, updated_at)
                 VALUES (?1,?2,'2026-01-15',?3,'transfer',?4,'',0,?5,?6,?6)",
                rusqlite::params![id, acct, amount, payee, group, now],
            )
            .unwrap();
        };
        // Bad manual pair: both positive, outgoing leg marked by payee.
        insert("m-from", a, 5000, "Transfer to account", "g-manual");
        insert("m-to", b, 5000, "Transfer from account", "g-manual");
        // Bad recurring pair: both positive, outgoing leg is the recurring row's account.
        conn.execute(
            "INSERT INTO recurring_transactions (id, account_id, payee, description, amount, kind, transfer_to_account_id, frequency, start_date, next_date, created_at, updated_at)
             VALUES ('r1', ?1, 'Savings', '', 2500, 'transfer', ?2, 'monthly', '2026-01-15', '2026-02-15', ?3, ?3)",
            rusqlite::params![c, d, now],
        )
        .unwrap();
        // Insert the incoming leg first so row order cannot decide the outcome.
        insert("r-to", d, 2500, "Savings", "g-rec");
        insert("r-from", c, 2500, "Savings", "g-rec");
        // Already-correct pair must stay untouched.
        insert("ok-from", a, -700, "Transfer to account", "g-ok");
        insert("ok-to", b, 700, "Transfer from account", "g-ok");

        let amount = |id: &str| -> i64 {
            conn.query_row("SELECT amount FROM transactions WHERE id = ?1", [id], |r| r.get(0))
                .unwrap()
        };
        let check = |label: &str| {
            assert_eq!(amount("m-from"), -5000, "{label}: manual outgoing");
            assert_eq!(amount("m-to"), 5000, "{label}: manual incoming");
            assert_eq!(amount("r-from"), -2500, "{label}: recurring outgoing");
            assert_eq!(amount("r-to"), 2500, "{label}: recurring incoming");
            assert_eq!(amount("ok-from"), -700, "{label}: correct outgoing");
            assert_eq!(amount("ok-to"), 700, "{label}: correct incoming");
        };

        run_migrations(&conn).unwrap();
        check("first run");
        run_migrations(&conn).unwrap();
        check("second run");
    }
}
