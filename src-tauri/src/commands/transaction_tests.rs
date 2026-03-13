#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use uuid::Uuid;
    use crate::db::migrations::run_migrations;
    use crate::helpers::timestamps::now_utc;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    fn seed_account(conn: &Connection) -> String {
        conn.query_row("SELECT id FROM accounts LIMIT 1", [], |r| r.get(0)).unwrap()
    }

    fn seed_category(conn: &Connection) -> String {
        conn.query_row("SELECT id FROM categories WHERE type='expense' LIMIT 1", [], |r| r.get(0)).unwrap()
    }

    fn insert_txn(conn: &Connection, account_id: &str, amount: i64, kind: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, payee, description, is_cleared, created_at, updated_at)
             VALUES (?1, ?2, '2026-01-15', ?3, ?4, 'Test Payee', '', 0, ?5, ?6)",
            rusqlite::params![id, account_id, amount, kind, now, now],
        ).unwrap();
        id
    }

    // ── CRUD tests ────────────────────────────────────────────────

    #[test]
    fn transaction_create_and_read() {
        let conn = setup();
        let acct = seed_account(&conn);
        let id = insert_txn(&conn, &acct, 5000, "expense");

        let amount: i64 = conn.query_row(
            "SELECT amount FROM transactions WHERE id = ?1", [&id], |r| r.get(0)
        ).unwrap();
        assert_eq!(amount, 5000);
    }

    #[test]
    fn transaction_amount_stored_as_cents() {
        let conn = setup();
        let acct = seed_account(&conn);
        let id = insert_txn(&conn, &acct, 4299, "expense"); // $42.99

        let amount: i64 = conn.query_row(
            "SELECT amount FROM transactions WHERE id = ?1", [&id], |r| r.get(0)
        ).unwrap();
        assert_eq!(amount, 4299, "amount must be stored as integer cents");
    }

    #[test]
    fn transaction_update() {
        let conn = setup();
        let acct = seed_account(&conn);
        let id = insert_txn(&conn, &acct, 1000, "expense");
        let now = now_utc();

        conn.execute(
            "UPDATE transactions SET amount=2000, updated_at=?1 WHERE id=?2",
            rusqlite::params![now, id],
        ).unwrap();

        let amount: i64 = conn.query_row(
            "SELECT amount FROM transactions WHERE id = ?1", [&id], |r| r.get(0)
        ).unwrap();
        assert_eq!(amount, 2000);
    }

    #[test]
    fn transaction_delete() {
        let conn = setup();
        let acct = seed_account(&conn);
        let id = insert_txn(&conn, &acct, 1000, "expense");
        conn.execute("DELETE FROM transactions WHERE id = ?1", [&id]).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE id = ?1", [&id], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn transaction_foreign_key_account_enforced() {
        let conn = setup();
        let now = now_utc();
        let result = conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, created_at, updated_at)
             VALUES ('t1', 'bad-account-id', '2026-01-01', 100, 'expense', ?1, ?2)",
            rusqlite::params![now, now],
        );
        assert!(result.is_err(), "invalid account_id must be rejected");
    }

    // ── Filter + pagination tests ─────────────────────────────────

    #[test]
    fn filter_by_kind() {
        let conn = setup();
        let acct = seed_account(&conn);
        insert_txn(&conn, &acct, 1000, "expense");
        insert_txn(&conn, &acct, 2000, "income");
        insert_txn(&conn, &acct, 3000, "expense");

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE kind='expense'", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn filter_by_account() {
        let conn = setup();
        let acct = seed_account(&conn);
        insert_txn(&conn, &acct, 1000, "expense");
        insert_txn(&conn, &acct, 2000, "expense");

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE account_id = ?1", [&acct], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn filter_by_date_range() {
        let conn = setup();
        let acct = seed_account(&conn);
        let now = now_utc();

        // Insert transactions on different dates
        for (date, amount) in [("2026-01-01", 100i64), ("2026-02-01", 200), ("2026-03-01", 300)] {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO transactions (id, account_id, date, amount, kind, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'expense', ?5, ?6)",
                rusqlite::params![id, acct, date, amount, now, now],
            ).unwrap();
        }

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE date >= '2026-01-15' AND date <= '2026-02-28'",
            [], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 1, "only Feb transaction in range");
    }

    #[test]
    fn pagination_offset() {
        let conn = setup();
        let acct = seed_account(&conn);
        for i in 0..10 {
            insert_txn(&conn, &acct, i * 100, "expense");
        }

        let page2: Vec<i64> = {
            let mut stmt = conn.prepare(
                "SELECT amount FROM transactions ORDER BY amount ASC LIMIT 5 OFFSET 5"
            ).unwrap();
            stmt.query_map([], |r| r.get(0)).unwrap()
                .map(|r| r.unwrap()).collect()
        };
        assert_eq!(page2.len(), 5, "page 2 should have 5 rows");
    }

    // ── Transfer tests ────────────────────────────────────────────

    #[test]
    fn transfer_creates_two_linked_transactions() {
        let conn = setup();
        let now = now_utc();

        // Create a second account
        let acct1 = seed_account(&conn);
        let acct2 = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO accounts (id, name, type, currency_code, opening_balance, sort_order, is_archived, created_at, updated_at)
             VALUES (?1, 'Savings', 'savings', 'USD', 0, 1, 0, ?2, ?3)",
            rusqlite::params![acct2, now, now],
        ).unwrap();

        let group_id = Uuid::new_v4().to_string();
        let from_id = Uuid::new_v4().to_string();
        let to_id = Uuid::new_v4().to_string();

        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, payee, description, is_cleared, transfer_group_id, created_at, updated_at)
             VALUES (?1, ?2, '2026-01-15', 10000, 'transfer', 'Transfer to account', '', 0, ?3, ?4, ?5)",
            rusqlite::params![from_id, acct1, group_id, now, now],
        ).unwrap();
        conn.execute(
            "INSERT INTO transactions (id, account_id, date, amount, kind, payee, description, is_cleared, transfer_group_id, created_at, updated_at)
             VALUES (?1, ?2, '2026-01-15', 10000, 'transfer', 'Transfer from account', '', 0, ?3, ?4, ?5)",
            rusqlite::params![to_id, acct2, group_id, now, now],
        ).unwrap();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE transfer_group_id = ?1", [&group_id], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 2, "transfer must create exactly 2 linked transactions");
    }

    #[test]
    fn transfer_amounts_match() {
        let conn = setup();
        let now = now_utc();
        let acct1 = seed_account(&conn);
        let acct2 = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO accounts (id, name, type, currency_code, opening_balance, sort_order, is_archived, created_at, updated_at)
             VALUES (?1, 'Savings2', 'savings', 'USD', 0, 2, 0, ?2, ?3)",
            rusqlite::params![acct2, now, now],
        ).unwrap();

        let group_id = Uuid::new_v4().to_string();
        for (tid, aid) in [(&Uuid::new_v4().to_string(), &acct1), (&Uuid::new_v4().to_string(), &acct2)] {
            conn.execute(
                "INSERT INTO transactions (id, account_id, date, amount, kind, payee, description, is_cleared, transfer_group_id, created_at, updated_at)
                 VALUES (?1, ?2, '2026-01-15', 5000, 'transfer', '', '', 0, ?3, ?4, ?5)",
                rusqlite::params![tid, aid, group_id, now, now],
            ).unwrap();
        }

        let amounts: Vec<i64> = {
            let mut stmt = conn.prepare(
                "SELECT amount FROM transactions WHERE transfer_group_id = ?1"
            ).unwrap();
            stmt.query_map([&group_id], |r| r.get(0)).unwrap()
                .map(|r| r.unwrap()).collect()
        };
        assert!(amounts.iter().all(|&a| a == 5000), "both transfer legs must have the same amount");
    }
}
