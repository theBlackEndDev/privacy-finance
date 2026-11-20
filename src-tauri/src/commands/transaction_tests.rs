#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use uuid::Uuid;
    use crate::db::migrations::{run_migrations, seed_test_data};
    use crate::helpers::timestamps::now_utc;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();
        seed_test_data(&conn).unwrap();
        conn
    }

    fn seed_account(conn: &Connection) -> String {
        conn.query_row("SELECT id FROM accounts LIMIT 1", [], |r| r.get(0)).unwrap()
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

        // Count only the test payee rows to isolate from seed data
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE kind='expense' AND payee='Test Payee'",
            [], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn filter_by_account() {
        let conn = setup();
        let acct = seed_account(&conn);
        let before: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE account_id = ?1", [&acct], |r| r.get(0)
        ).unwrap();

        insert_txn(&conn, &acct, 1000, "expense");
        insert_txn(&conn, &acct, 2000, "expense");

        let after: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE account_id = ?1", [&acct], |r| r.get(0)
        ).unwrap();
        assert_eq!(after - before, 2);
    }

    #[test]
    fn filter_by_date_range() {
        let conn = setup();
        let acct = seed_account(&conn);
        let now = now_utc();

        // Use dates well outside the 2026-01..2026-03 seed range
        for (date, amount) in [("2027-01-01", 100i64), ("2027-02-01", 200), ("2027-03-01", 300)] {
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO transactions (id, account_id, date, amount, kind, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, 'expense', ?5, ?6)",
                rusqlite::params![id, acct, date, amount, now, now],
            ).unwrap();
        }

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE date >= '2027-01-15' AND date <= '2027-02-28'",
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

    fn make_account(conn: &Connection, name: &str, sort: i64) -> String {
        let id = Uuid::new_v4().to_string();
        let now = now_utc();
        conn.execute(
            "INSERT INTO accounts (id, name, type, currency_code, opening_balance, sort_order, is_archived, created_at, updated_at)
             VALUES (?1, ?2, 'savings', 'USD', 0, ?3, 0, ?4, ?4)",
            rusqlite::params![id, name, sort, now],
        ).unwrap();
        id
    }

    fn do_transfer(conn: &Connection, from: &str, to: &str, amount: i64) -> String {
        use crate::commands::transactions::create_transfer_inner;
        use crate::models::transaction::CreateTransferInput;
        let (from_leg, _) = create_transfer_inner(conn, CreateTransferInput {
            from_account_id: from.to_string(),
            to_account_id: to.to_string(),
            amount,
            date: "2026-01-15".to_string(),
            description: None,
        }).unwrap();
        from_leg.transfer_group_id.unwrap()
    }

    #[test]
    fn transfer_creates_two_linked_transactions() {
        let conn = setup();
        let acct1 = seed_account(&conn);
        let acct2 = make_account(&conn, "Savings", 1);

        let group_id = do_transfer(&conn, &acct1, &acct2, 10000);

        let legs: Vec<(String, i64)> = {
            let mut stmt = conn.prepare(
                "SELECT account_id, amount FROM transactions WHERE transfer_group_id = ?1 ORDER BY amount"
            ).unwrap();
            stmt.query_map([&group_id], |r| Ok((r.get(0)?, r.get(1)?))).unwrap()
                .map(|r| r.unwrap()).collect()
        };
        assert_eq!(legs.len(), 2, "transfer must create exactly 2 linked transactions");
        assert_eq!(legs[0], (acct1, -10000), "outgoing leg is negative");
        assert_eq!(legs[1], (acct2, 10000), "incoming leg is positive");
    }

    #[test]
    fn transfer_amounts_match() {
        let conn = setup();
        let acct1 = seed_account(&conn);
        let acct2 = make_account(&conn, "Savings2", 2);

        let group_id = do_transfer(&conn, &acct1, &acct2, 5000);

        let amounts: Vec<i64> = {
            let mut stmt = conn.prepare(
                "SELECT amount FROM transactions WHERE transfer_group_id = ?1 ORDER BY amount"
            ).unwrap();
            stmt.query_map([&group_id], |r| r.get(0)).unwrap()
                .map(|r| r.unwrap()).collect()
        };
        assert_eq!(amounts, vec![-5000, 5000], "legs must be equal magnitude, opposite sign");
        assert_eq!(amounts.iter().sum::<i64>(), 0);
    }

    // Same CASE expression as `get_account_balances` in dashboard.rs.
    fn balance(conn: &Connection, account_id: &str) -> i64 {
        conn.query_row(
            "SELECT a.opening_balance + COALESCE(SUM(
                 CASE
                     WHEN t.kind = 'income' THEN t.amount
                     WHEN t.kind = 'expense' THEN -t.amount
                     ELSE t.amount
                 END
             ), 0)
             FROM accounts a
             LEFT JOIN transactions t ON t.account_id = a.id
             WHERE a.id = ?1
             GROUP BY a.id",
            [account_id],
            |r| r.get(0),
        ).unwrap()
    }

    #[test]
    fn create_transfer_moves_money() {
        use crate::commands::transactions::create_transfer_inner;
        use crate::models::transaction::CreateTransferInput;

        let conn = setup();
        let now = now_utc();
        let from = seed_account(&conn);
        let to = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO accounts (id, name, type, currency_code, opening_balance, sort_order, is_archived, created_at, updated_at)
             VALUES (?1, 'Savings2', 'savings', 'USD', 0, 2, 0, ?2, ?3)",
            rusqlite::params![to, now, now],
        ).unwrap();

        let from_before = balance(&conn, &from);
        let to_before = balance(&conn, &to);

        let input = |amount| CreateTransferInput {
            from_account_id: from.clone(),
            to_account_id: to.clone(),
            amount,
            date: "2026-01-15".to_string(),
            description: None,
        };

        let (from_leg, to_leg) = create_transfer_inner(&conn, input(7500)).unwrap();
        assert_eq!(from_leg.amount, -7500);
        assert_eq!(to_leg.amount, 7500);
        assert_eq!(balance(&conn, &from), from_before - 7500, "source must drop");
        assert_eq!(balance(&conn, &to), to_before + 7500, "destination must rise");

        // Negative input is rejected, not sign-flipped; zero likewise.
        assert!(create_transfer_inner(&conn, input(-7500)).is_err());
        assert!(create_transfer_inner(&conn, input(0)).is_err());
        assert_eq!(balance(&conn, &from), from_before - 7500);
    }

    fn setup_transfer(conn: &Connection) -> (String, String, String) {
        use crate::commands::transactions::create_transfer_inner;
        use crate::models::transaction::CreateTransferInput;

        let now = now_utc();
        let from = seed_account(conn);
        let to = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO accounts (id, name, type, currency_code, opening_balance, sort_order, is_archived, created_at, updated_at)
             VALUES (?1, 'Savings2', 'savings', 'USD', 0, 2, 0, ?2, ?3)",
            rusqlite::params![to, now, now],
        ).unwrap();
        let (from_leg, to_leg) = create_transfer_inner(conn, CreateTransferInput {
            from_account_id: from,
            to_account_id: to,
            amount: 7500,
            date: "2026-01-15".to_string(),
            description: None,
        }).unwrap();
        (from_leg.id, to_leg.id, from_leg.transfer_group_id.unwrap())
    }

    #[test]
    fn delete_transfer_removes_both_legs() {
        use crate::commands::transactions::delete_transaction_inner;

        let conn = setup();
        let (from_id, _to_id, group) = setup_transfer(&conn);
        let plain = insert_txn(&conn, &seed_account(&conn), 1000, "expense");

        delete_transaction_inner(&conn, &from_id).unwrap();

        let left: i64 = conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE transfer_group_id = ?1", [&group], |r| r.get(0)
        ).unwrap();
        assert_eq!(left, 0, "both legs must be deleted");

        // Unrelated non-transfer rows are untouched, and a plain delete removes just one row.
        delete_transaction_inner(&conn, &plain).unwrap();
        assert!(delete_transaction_inner(&conn, &plain).is_err());
    }

    #[test]
    fn update_transfer_updates_both_legs() {
        use crate::commands::transactions::update_transaction_inner;
        use crate::models::transaction::UpdateTransactionInput;

        let conn = setup();
        let (from_id, to_id, _group) = setup_transfer(&conn);

        let updated = update_transaction_inner(&conn, from_id.clone(), UpdateTransactionInput {
            date: Some("2026-02-01".to_string()),
            amount: Some(-9000),
            kind: None,
            category_id: None,
            payee: None,
            description: None,
            is_cleared: None,
        }).unwrap();
        assert_eq!(updated.amount, -9000);

        let row = |id: &str| -> (String, i64) {
            conn.query_row("SELECT date, amount FROM transactions WHERE id = ?1", [id],
                |r| Ok((r.get(0)?, r.get(1)?))).unwrap()
        };
        assert_eq!(row(&from_id), ("2026-02-01".to_string(), -9000));
        assert_eq!(row(&to_id), ("2026-02-01".to_string(), 9000), "peer gets date and mirrored amount");
    }
}
